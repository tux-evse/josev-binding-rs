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
    iec_api: &'static str,
    meter_api: &'static str,
    meter_max_period_ms: i32
}

impl AfbApiControls for ApiUserData {
    // the API is created and ready. At this level user may subcall api(s) declare as dependencies
    fn start(&mut self, api: &AfbApi) -> Result<(), AfbError> {
        println!("== JOSEV binding starting");

        // Subscribe to IEC events
        AfbSubCall::call_sync(api, self.iec_api, "subscribe", true)?;

        // Subscribe to energy meter
        AfbSubCall::call_sync(api, self.meter_api, "tension", EnergyAction::SUBSCRIBE)?;
        AfbSubCall::call_sync(api, self.meter_api, "current", EnergyAction::SUBSCRIBE)?;

        // Subscribe to MQTT iso15118_state_info (for authorization)
        // FIXME ?
        let subscribed_messages = JsoncObj::array();
        subscribed_messages.append("iso15118_state_info")?;
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

fn iec_event_cb(_evt: &AfbEventMsg, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx: &mut SharedContext = ctx.get_mut::<SharedContext>()?;

    let msg = args.get::<&Iec6185Msg>(0)?;

    {
        let mut ctx = ctx.shared.write().unwrap();
        match msg {
            Iec6185Msg::Plugged(plugged) => {
                if *plugged && matches!(ctx.charging_state, josev::ControlPilotState::A1) {
                    ctx.charging_state = josev::ControlPilotState::B1;
                }
                if !*plugged {
                    ctx.charging_state = josev::ControlPilotState::A1;
                }
            }
            Iec6185Msg::PowerRqt(requested) => {
                if *requested && matches!(ctx.charging_state, josev::ControlPilotState::B1) {
                    ctx.charging_state = josev::ControlPilotState::C2;
                }
            }
            Iec6185Msg::RelayOn(closed) => {
                ctx.contactor_closed = *closed;
                ctx.contactor_status_event
                    .push(josev::CsContactorStatusUpdate {
                        evse_id: ctx.cs_parameters.parameters[0].evse_id.clone(),
                        status: {
                            if *closed {
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
        ctx.cp_status_event.push(josev::CpStatusUpdate {
            evse_id: ctx.cs_parameters.parameters[0].evse_id.clone(),
            connector_id: ctx.cs_parameters.parameters[0].connectors[0].id,
            state: ctx.charging_state,
            max_voltage: None,
            min_voltage: None,
            duty_cycle: None,
        });
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
                println!("tension {}", data.total);
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
    }

    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

fn mqtt_event_cb(_evt: &AfbEventMsg, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let msg = args.get::<JsoncObj>(0)?;
    let ctx = ctx.get_ref::<SharedContext>()?;

    // Always authorize the session
    if let Ok(session_status) = msg.get::<&'static str>("session_status") {
        if session_status == "Authorization" {
            let ctx = ctx.shared.read().unwrap();
            ctx.authorization_event.push(josev::AuthorizationUpdate {
                evse_id: msg.get::<&'static str>("evse_id")?.to_string(),
                token_type: josev::AuthorizationTokenType::KeyCode,
                status: josev::AuthorizationStatus::Accepted,
                id_token: Some("1234".to_owned()),
            });
        }
    }
    Ok(())
}

fn on_cable_check(
    request: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let arg: &josev::CableCheckRequest = args.get::<&josev::CableCheckRequest>(0)?;
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;

    let response = {
        let ctx = ctx.shared.read().unwrap();

        if ctx.cs_parameters.parameters[0].evse_id != arg.evse_id {
            // ignore requests for another EVSE ID
            return Ok(());
        }

        josev::CableCheckResponse {
            evse_id: arg.evse_id.clone(),
            cable_check_status: josev::CableCheckStatus::Finished,
            isolation_level: Some(josev::IsolationLevel::Valid),
        }
    };

    request.reply(response, 0);
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

fn on_power_electronics_setpoint(
    request: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let arg: &josev::PowerElectronicsSetpointRequest =
        args.get::<&josev::PowerElectronicsSetpointRequest>(0)?;
    let ctx: &mut SharedContext = ctx.get_mut::<SharedContext>()?;

    // Set present_voltage and present_current to the ones asked for
    {
        let mut ctx = ctx.shared.write().unwrap();
        let limits = &mut ctx.cs_status_and_limits.evses[0];
        limits
            .dc
            .as_mut()
            .map(|x| x.present_voltage = arg.dc.as_ref().unwrap().voltage);
    }

    // Always accept set point
    let response = josev::PowerElectronicsSetpointResponse {
        evse_id: arg.evse_id.clone(),
        status: josev::SetPointRequestStatus::Accepted,
    };
    request.reply(response, 0);
    Ok(())
}

fn on_status_and_limits(
    request: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    println!("***** CS STATUS AND LIMITS");
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
    println!("***** CS PARAMETERS");
    let ctx = ctx.get_ref::<SharedContext>()?;
    let ctx = ctx.shared.read().unwrap();
    request.reply(ctx.cs_parameters.clone(), 0);
    Ok(())
}

const JOSEV_API: &str = "josev";

pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    // Custom type converters
    am62x_registers()?;
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

    let iec_api = jconf.get::<&'static str>("iec_api")?;
    let meter_api = jconf.get::<&'static str>("meter_api")?;

    let cp_status_event = AfbEvent::new("cp_status");
    let authorization_event = AfbEvent::new("authorization");
    let contactor_status_event = AfbEvent::new("cs_constactor_status");

    let charging_state = josev::ControlPilotState::A1;

    let meter_max_period_ms: i32 = {
        if let Ok(value) = jconf.get("meter_max_period_ms") {
            value
        } else {
            1000
        }
    };

    let config = ApiUserData { iec_api, meter_api, meter_max_period_ms };

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

    api.require_api(iec_api);
    api.require_api(meter_api);

    let iec_handler = AfbEvtHandler::new("iec-evt")
        .set_pattern(to_static_str(format!("{}/*", iec_api)))
        .set_callback(iec_event_cb)
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

    let energy_evt_handler = AfbEvtHandler::new("energy-evt-handler")
        .set_pattern(to_static_str(format!("{}/*", meter_api)))
        .set_callback(energy_event_cb)
        .set_context(shared_context.clone())
        .finalize()?;

    //
    // Verbs called by Josev
    //

    // For DC charge
    let cable_check_verb = AfbVerb::new("cable_check")
        .set_callback(on_cable_check)
        .set_context(shared_context.clone())
        .finalize()?;

    let contactor_status_verb = AfbVerb::new("cs_contactor_status")
        .set_callback(on_contactor_status)
        .set_context(shared_context.clone())
        .finalize()?;

    let electronics_setpoint_verb = AfbVerb::new("power_electronics_setpoint")
        .set_callback(on_power_electronics_setpoint)
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

    api.add_evt_handler(iec_handler);
    api.add_event(cp_status_event);
    api.add_event(authorization_event);
    api.add_event(contactor_status_event);
    api.add_verb(subscribe_verb);
    api.add_evt_handler(mqtt_handler);
    api.add_evt_handler(energy_evt_handler);

    api.add_verb(cable_check_verb);
    api.add_verb(contactor_status_verb);
    api.add_verb(electronics_setpoint_verb);
    api.add_verb(status_and_limits_verb);
    api.add_verb(cs_parameters_verb);

    Ok(api.finalize()?)
}

AfbBindingRegister!(binding_init);
