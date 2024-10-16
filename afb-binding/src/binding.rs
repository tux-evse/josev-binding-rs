/*
 * Copyright (C) 2015-2024 IoT.bzh Company
 * Author: Hugo Mercier <hugo.mercier@iot.bzh>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 */

use std::sync::{Arc, RwLock};

use crate::josev;
use afbv4::prelude::*;
use std::time;
use typesv4::prelude::*;

#[derive(Clone)]
pub struct ApiUserData {
    charge_api: &'static str,
    auth_api: &'static str,
    meter_api: &'static str,
    meter_max_period_ms: i32,

    evse_id: String,
}

impl AfbApiControls for ApiUserData {
    // the API is created and ready. At this level user may subcall api(s) declare as dependencies
    fn start(&mut self, api: &AfbApi) -> Result<(), AfbError> {
        println!("== JOSEV binding starting");

        // Subscribe to IEC events
        AfbSubCall::call_sync(api, self.charge_api, "subscribe", true)?;

        // Subscribe to energy meter
        AfbSubCall::call_sync(api, self.meter_api, "tension", EnergyAction::SUBSCRIBE)?;
        AfbSubCall::call_sync(api, self.meter_api, "current", EnergyAction::SUBSCRIBE)?;

        // Reset authentication
        if let Err(err) = AfbSubCall::call_sync(api, self.auth_api, "logout", 0) {
            afb_log_msg!(
                Notice,
                api.get_apiv4(),
                "**logout failed**, probably already logged out"
            );
        }

        // Subscribe to MQTT iso15118_state_info (for authorization)
        let subscribed_messages = JsoncObj::array();
        subscribed_messages.append("iso15118_state_info")?;
        subscribed_messages.append("hlc_charging")?;
        AfbSubCall::call_sync(api, "from_mqtt", "subscribe_events", subscribed_messages)?;
        Ok(())
    }

    // mandatory unsed declaration
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

//
// The Context shared by all verbs and events
struct Context {
    cp_status_event: &'static AfbEvent,
    authorization_event: &'static AfbEvent,
    contactor_status_event: &'static AfbEvent,

    // current charging state
    charging_state: josev::ControlPilotState,

    // curent contactor state
    contactor_closed: bool,

    // static parameters of the charging station
    cs_parameters: josev::CsParametersResponse,

    // current electrical state of the station
    cs_status_and_limits: josev::CsStatusAndLimitsResponse,

    last_energy_update: Option<time::Instant>,
}

#[derive(Clone)]
struct SharedContext {
    config: ApiUserData,

    shared: Arc<RwLock<Context>>,
}

fn charge_event_cb(evt: &AfbEventMsg, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx: &mut SharedContext = ctx.get_mut::<SharedContext>()?;

    let msg: &ChargingMsg = args.get::<&ChargingMsg>(0)?;
    afb_log_msg!(Debug, evt.get_apiv4(), "Charge event received {:?}", msg);

    {
        let mut ctx = ctx.shared.write().unwrap();
        match msg {
            ChargingMsg::Plugged(plugged) => {
                match *plugged {
                    PlugState::PlugIn => {
                        ctx.charging_state = josev::ControlPilotState::B1;
                    }
                    PlugState::Lock => {
                        ctx.charging_state = josev::ControlPilotState::C2;
                    }
                    _ => {
                        ctx.charging_state = josev::ControlPilotState::A1;
                    }
                }

                ctx.cp_status_event.push(josev::CpStatusUpdate {
                    evse_id: ctx.cs_parameters.parameters[0].evse_id.clone(),
                    connector_id: ctx.cs_parameters.parameters[0].connectors[0].id,
                    state: ctx.charging_state,
                    max_voltage: None,
                    min_voltage: None,
                    duty_cycle: None,
                });
            }
            ChargingMsg::Power(power_state) => {
                match *power_state {
                    PowerRequest::Start | PowerRequest::Charging(_) => {
                        ctx.contactor_closed = true;
                    }
                    PowerRequest::Stop(_) => {
                        ctx.contactor_closed = false;
                    }
                    _ => {}
                }
                ctx.contactor_status_event
                    .push(josev::CsContactorStatusUpdate {
                        evse_id: ctx.cs_parameters.parameters[0].evse_id.clone(),
                        status: {
                            if ctx.contactor_closed {
                                josev::CsContactorStatusResponseStatus::Closed
                            } else {
                                josev::CsContactorStatusResponseStatus::Opened
                            }
                        },
                        info: None,
                    });
            }
            _ => {}
        }
    }

    Ok(())
}

fn energy_event_cb(
    _evt: &AfbEventMsg,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let ctx: &mut SharedContext = ctx.get_mut::<SharedContext>()?;

    {
        // Do not update energy values too fast
        let sctx = ctx.shared.read().unwrap();
        if let Some(instant) = sctx.last_energy_update {
            if instant.elapsed().as_millis() < ctx.config.meter_max_period_ms as u128 {
                return Ok(());
            }
        }
    }

    let data: &MeterDataSet = args.get::<&MeterDataSet>(0)?;
    {
        let mut ctx = ctx.shared.write().unwrap();
        match data.tag {
            MeterTagSet::Tension => {
                afb_log_msg!(Debug, _evt.get_apiv4(), "Received tension {}", data.total);
            }
            MeterTagSet::Current => {
                afb_log_msg!(Debug, _evt.get_apiv4(), "Received current {}", data.total);
            }
            _ => {}
        }

        ctx.last_energy_update = Some(time::Instant::now());
    }

    Ok(())
}

//
// Verb dedicated to the MQTT extension so that an MQTT update message
// is sent when we push to a set of events
fn on_subscribe(
    request: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<SharedContext>()?;

    {
        let ctx = ctx.shared.read().unwrap();
        ctx.cp_status_event.subscribe(request)?;
        ctx.authorization_event.subscribe(request)?;
        ctx.contactor_status_event.subscribe(request)?;
    }

    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

fn mqtt_event_cb(evt: &AfbEventMsg, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let msg = args.get::<JsoncObj>(0)?;
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;
    let config = &ctx.config;

    let evse_id = msg.get::<&'static str>("evse_id")?;

    // ignore other EVSE IDs
    if ctx.config.evse_id != evse_id {
        return Ok(());
    }

    // Always authorize the session
    if let Ok(session_status) = msg.get::<&'static str>("session_status") {
        if session_status == "Authorization" {
            let ctx = ctx.shared.read().unwrap();

            // Ask for authorization
            let auth_reply =
                AfbSubCall::call_sync(evt.get_apiv4(), config.auth_api, "login", false)?;
            let auth_state: &AuthState = auth_reply.get_onsuccess::<&AuthState>(0)?;
            if matches!(auth_state.auth, AuthMsg::Done) {
                ctx.authorization_event.push(josev::AuthorizationUpdate {
                    evse_id: evse_id.to_string(),
                    token_type: josev::AuthorizationTokenType::KeyCode,
                    status: josev::AuthorizationStatus::Accepted,
                    id_token: Some(auth_state.tagid.clone()),
                });
            }
        } else if session_status == "SessionStop" {
            // Open the contactor
            AfbSubCall::call_sync(
                evt.get_apiv4(),
                ctx.config.charge_api,
                "remote_power",
                false,
            )?;
        }
    }
    Ok(())
}

fn on_hlc_charging(evt: &AfbEventMsg, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let msg = args.get::<JsoncObj>(0)?;
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;
    let config = &ctx.config;

    // Always authorize the session
    if let Ok(evse_id) = msg.get::<&'static str>("evse_id") {
        if let Ok(status) = msg.get::<bool>("status") {
            if config.evse_id != evse_id {
                // ignore messages of other EVSE IDs
                return Ok(());
            }
            // Close the contactor
            AfbSubCall::call_sync(evt.get_apiv4(), config.charge_api, "remote_power", status)?;

            {
                let ctx = ctx.shared.write().unwrap();
                ctx.cp_status_event.push(josev::CpStatusUpdate {
                    evse_id: ctx.cs_parameters.parameters[0].evse_id.clone(),
                    connector_id: ctx.cs_parameters.parameters[0].connectors[0].id,
                    state: ctx.charging_state,
                    max_voltage: None,
                    min_voltage: None,
                    duty_cycle: None,
                });
            }
        }
    }
    Ok(())
}

fn on_contactor_status(
    request: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let arg = args.get::<&josev::CsContactorStatusRequest>(0)?;
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;

    let response = {
        let ctx = ctx.shared.read().unwrap();

        if ctx.cs_parameters.parameters[0].evse_id != arg.evse_id {
            // ignore requests for another EVSE ID
            return Ok(());
        }

        josev::CsContactorStatusResponse {
            evse_id: arg.evse_id.clone(),
            status: {
                if ctx.contactor_closed {
                    josev::CsContactorStatusResponseStatus::Closed
                } else {
                    josev::CsContactorStatusResponseStatus::Opened
                }
            },
            info: None,
        }
    };

    request.reply(response, 0);
    Ok(())
}

fn on_status_and_limits(
    request: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    afb_log_msg!(Debug, request.get_apiv4(), "CS STATUS AND LIMITS");
    let ctx = ctx.get_ref::<SharedContext>()?;
    let ctx = ctx.shared.read().unwrap();
    request.reply(ctx.cs_status_and_limits.clone(), 0);
    Ok(())
}

fn on_cs_parameters(
    request: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    afb_log_msg!(Debug, request.get_apiv4(), "CS PARAMETERS");
    let ctx = ctx.get_ref::<SharedContext>()?;
    let ctx = ctx.shared.read().unwrap();
    request.reply(ctx.cs_parameters.clone(), 0);
    Ok(())
}

const JOSEV_API: &str = "josev";

pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    // Custom type converters
    auth_registers()?;
    chmgr_registers()?;
    engy_registers()?;
    josev::josev_registers()?;

    let cs_parameters = jconf.get::<JsoncObj>("cs_parameters")?;
    let cs_parameters: josev::CsParametersResponse =
        serde_json::from_str(&cs_parameters.to_string())
            .or_else(|error| afb_error!(JOSEV_API, "'cs_parameters' malformed: {}", error))?;

    if cs_parameters.parameters.len() != 1 {
        return afb_error!(
            JOSEV_API,
            "One EVSE is mandatory, and only one is supported"
        );
    }
    if cs_parameters.parameters[0].connectors.len() != 1 {
        return afb_error!(
            JOSEV_API,
            "One connector is mandatory, and only one is supported"
        );
    }

    let cs_status_and_limits = jconf.get::<JsoncObj>("cs_status_and_limits")?;
    let cs_status_and_limits: josev::CsStatusAndLimitsResponse =
        serde_json::from_str(&cs_status_and_limits.to_string()).or_else(|error| {
            afb_error!(JOSEV_API, "'cs_status_and_limits' malformed: {}", error)
        })?;

    let charge_api = jconf.get::<&'static str>("charge_api")?;
    let meter_api = jconf.get::<&'static str>("meter_api")?;
    let auth_api = jconf.get::<&'static str>("auth_api")?;

    let cp_status_event = AfbEvent::new("cp_status");
    let authorization_event = AfbEvent::new("authorization");
    let contactor_status_event = AfbEvent::new("cs_contactor_status");

    let charging_state = josev::ControlPilotState::C2;

    let meter_max_period_ms: i32 = {
        if let Ok(value) = jconf.get("meter_max_period_ms") {
            value
        } else {
            1000
        }
    };

    // copy evse_id as immutable configuration
    let evse_id = cs_parameters.parameters[0].evse_id.clone();

    let config = ApiUserData {
        charge_api,
        auth_api,
        meter_api,
        meter_max_period_ms,
        evse_id,
    };

    let api = AfbApi::new(JOSEV_API).set_callback(Box::new(config.clone()));

    let shared_context = SharedContext {
        config,
        shared: Arc::new(RwLock::new(Context {
            cp_status_event,
            authorization_event,
            contactor_status_event,
            charging_state,
            cs_parameters,
            cs_status_and_limits,
            contactor_closed: false,
            last_energy_update: None,
        })),
    };

    api.require_api(charge_api);
    api.require_api(meter_api);
    api.require_api(auth_api);

    let charge_handler = AfbEvtHandler::new("charge-evt")
        .set_pattern(to_static_str(format!("{}/*", charge_api)))
        .set_callback(charge_event_cb)
        .set_context(shared_context.clone())
        .finalize()?;

    let subscribe_verb = AfbVerb::new("subscribe")
        .set_callback(on_subscribe)
        .set_context(shared_context.clone())
        .finalize()?;

    let mqtt_handler = AfbEvtHandler::new("mqtt-evt")
        .set_pattern(to_static_str(
            "from_mqtt/event/iso15118_state_info".to_owned(),
        ))
        .set_callback(mqtt_event_cb)
        .set_context(shared_context.clone())
        .finalize()?;

    let hlc_charging_handler = AfbEvtHandler::new("hlc-charging-evt")
        .set_pattern(to_static_str("from_mqtt/event/hlc_charging".to_owned()))
        .set_callback(on_hlc_charging)
        .set_context(shared_context.clone())
        .finalize()?;

    let energy_evt_handler = AfbEvtHandler::new("energy-evt-handler")
        .set_pattern(to_static_str(format!("{}/*", meter_api)))
        .set_callback(energy_event_cb)
        .set_context(shared_context.clone())
        .finalize()?;

    //
    // Verbs called by Josev
    //

    let contactor_status_verb = AfbVerb::new("cs_contactor_status")
        .set_callback(on_contactor_status)
        .set_context(shared_context.clone())
        .finalize()?;

    let status_and_limits_verb = AfbVerb::new("cs_status_and_limits")
        .set_callback(on_status_and_limits)
        .set_context(shared_context.clone())
        .finalize()?;

    let cs_parameters_verb = AfbVerb::new("cs_parameters")
        .set_callback(on_cs_parameters)
        .set_context(shared_context.clone())
        .finalize()?;

    api.add_evt_handler(charge_handler);
    api.add_event(cp_status_event);
    api.add_event(authorization_event);
    api.add_event(contactor_status_event);
    api.add_verb(subscribe_verb);
    api.add_evt_handler(mqtt_handler);
    api.add_evt_handler(energy_evt_handler);
    api.add_evt_handler(hlc_charging_handler);

    api.add_verb(contactor_status_verb);
    api.add_verb(status_and_limits_verb);
    api.add_verb(cs_parameters_verb);

    Ok(api.finalize()?)
}

AfbBindingRegister!(binding_init);
