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
use typesv4::prelude::*;

#[derive(Clone)]
pub struct ApiUserData {
    charge_api: &'static str,
    auth_api: &'static str,
    meter_api: &'static str,

    evse_id: String,
}

impl AfbApiControls for ApiUserData {
    // the API is created and ready. At this level user may subcall api(s) declare as dependencies
    fn start(&mut self, api: &AfbApi) -> Result<(), AfbError> {
        println!("== JOSEV binding starting");

        // Subscribe to IEC events
        AfbSubCall::call_sync(api, self.charge_api, "subscribe", true)?;

        // Reset authentication
        if let Err(_err) = AfbSubCall::call_sync(api, self.auth_api, "logout", 0) {
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
        subscribed_messages.append("transaction_status")?;
        subscribed_messages.append("slac_status")?;
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

    // current contactor state
    contactor_closed: bool,

    // static parameters of the charging station
    cs_parameters: josev::CsParametersResponse,

    // current electrical state of the station
    cs_status_and_limits: josev::CsStatusAndLimitsResponse,

    device_model: Option<josev::DeviceModelResponse>,

    // for debugging
    forced_charging_state: Option<josev::ControlPilotState>,
    forced_contactor_closed: Option<bool>,
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
        let mut b1_b2_transition = false;
        let mut ctx = ctx.shared.write().unwrap();
        match msg {
            ChargingMsg::Plugged(plugged) => {
                match *plugged {
                    PlugState::PlugIn => {
                        if matches!(ctx.charging_state, josev::ControlPilotState::A1) {
                            b1_b2_transition = true;
                        }
                        ctx.charging_state = josev::ControlPilotState::B2;
                    }
                    PlugState::Lock => {
                        ctx.charging_state = josev::ControlPilotState::C2;
                    }
                    _ => {
                        ctx.charging_state = josev::ControlPilotState::A1;
                    }
                }

                if ctx.forced_charging_state.is_none() {
                    if b1_b2_transition {
                        // Moving from A1 to B2 is sometimes too extreme,
                        // move first to B1 before moving to B2
                        ctx.cp_status_event.push(josev::CpStatusUpdate {
                            evse_id: ctx.cs_parameters.parameters[0].evse_id.clone(),
                            connector_id: ctx.cs_parameters.parameters[0].connectors[0].id,
                            state: josev::ControlPilotState::B1,
                            max_voltage: None,
                            min_voltage: None,
                            duty_cycle: None,
                        });
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
                if ctx.forced_contactor_closed.is_none() {
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
            }
            _ => {}
        }
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
            if let Ok(info) = msg.get::<JsoncObj>("info") {
                // The keyword "selectedPaymentOption" in python version
                // is different in the rust version ("selected_payment_option")
                // So this part will only work in python-iso15118
                if let Ok(selected_payment_option) =
                    info.get::<&'static str>("selectedPaymentOption")
                {
                    let iso_payment_option = match selected_payment_option.to_lowercase().as_str() {
                        "eim" => Some(ChargingMsg::Payment(PaymentOption::Eim)),
                        "pnc" => Some(ChargingMsg::Payment(PaymentOption::Pnc)),
                        _ => {
                            return afb_error!(
                                JOSEV_API,
                                "Invalid Payment Option: {}",
                                selected_payment_option
                            );
                        }
                    };

                    if let Some(option) = iso_payment_option {
                        AfbSubCall::call_sync(
                            evt.get_apiv4(),
                            config.charge_api,
                            "payment-option",
                            option,
                        )?;
                    }
                }
            }
            // Ask for authorization
            let auth_reply =
                AfbSubCall::call_sync(evt.get_apiv4(), config.auth_api, "login", false)?;
            let auth_state: &AuthState = auth_reply.get_onsuccess::<&AuthState>(0)?;
            if matches!(auth_state.auth, AuthMsg::Done) {
                if auth_state.ocpp_check {
                    // We ask josev for an authorization with this token.
                    // It will be forwarded to OCPP
                    AfbSubCall::call_sync(
                        evt.get_apiv4(),
                        "to_mqtt",
                        "authorization",
                        josev::AuthorizationRequest {
                            evse_id: Some(evse_id.to_string()),
                            id_token: Some(auth_state.tagid.clone()),
                            token_type: josev::AuthorizationTokenType::ISO14443,
                        },
                    )?;
                } else {
                    // Otherwise, no OCPP is involved and we accept the authorization
                    // by issuing an "update" Authorization message
                    ctx.authorization_event.push(josev::AuthorizationUpdate {
                        evse_id: evse_id.to_string(),
                        token_type: josev::AuthorizationTokenType::ISO14443,
                        status: josev::AuthorizationStatus::Accepted,
                        id_token: Some(auth_state.tagid.clone()),
                    });
                }
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
                    state: {
                        if let Some(forced) = ctx.forced_charging_state {
                            forced
                        } else {
                            ctx.charging_state
                        }
                    },
                    max_voltage: None,
                    min_voltage: None,
                    duty_cycle: None,
                });
            }
        }
    }
    Ok(())
}

fn on_transaction_status(
    evt: &AfbEventMsg,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let msg = args.get::<JsoncObj>(0)?;
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;
    let config = &ctx.config;

    if let Ok(evse_id) = msg.get::<&'static str>("evse_id") {
        if let Ok(status) = msg.get::<&'static str>("status") {
            if config.evse_id != evse_id {
                // ignore messages of other EVSE IDs
                return Ok(());
            }

            if status == "ended" {
                // Open the contactor
                AfbSubCall::call_sync(evt.get_apiv4(), config.charge_api, "remote_power", false)?;
            }
        }
    }
    Ok(())
}

fn on_slac_status(evt: &AfbEventMsg, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let msg: &josev::SlacStatusUpdate = args.get::<&josev::SlacStatusUpdate>(0)?;
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;
    let config = &ctx.config;

    if msg.evse_id != config.evse_id {
        // ignore messages of other EVSE IDs
        return Ok(());
    }

    let slac_status = match msg.status {
        josev::SlacStatusUpdateStatus::Unmatched => SlacStatus::UNMATCHED,
        josev::SlacStatusUpdateStatus::Matched => SlacStatus::MATCHED,
        josev::SlacStatusUpdateStatus::Matching => SlacStatus::MATCHING,
        josev::SlacStatusUpdateStatus::Failed => SlacStatus::UNMATCHED,
        josev::SlacStatusUpdateStatus::BasicCharging => SlacStatus::TIMEOUT,
    };
    AfbSubCall::call_sync(
        evt.get_apiv4(),
        config.charge_api,
        "set_slac_status",
        slac_status,
    )?;
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
                if let Some(closed) = ctx.forced_contactor_closed {
                    if closed {
                        josev::CsContactorStatusResponseStatus::Closed
                    } else {
                        josev::CsContactorStatusResponseStatus::Opened
                    }
                } else {
                    if ctx.contactor_closed {
                        josev::CsContactorStatusResponseStatus::Closed
                    } else {
                        josev::CsContactorStatusResponseStatus::Opened
                    }
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

fn on_device_model(
    request: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    afb_log_msg!(Debug, request.get_apiv4(), "DEVICE MODEL");
    let ctx = ctx.get_ref::<SharedContext>()?;
    let ctx = ctx.shared.read().unwrap();
    if let Some(device_model) = ctx.device_model.clone() {
        request.reply(device_model, 0);
    }
    Ok(())
}

fn on_stop_charging(
    request: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let arg = args.get::<&josev::StopChargingRequest>(0)?;
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;

    {
        let ctx = ctx.shared.read().unwrap();

        if &ctx.cs_parameters.parameters[0].evse_id != &arg.evse_id {
            // ignore requests for another EVSE ID
            return Ok(());
        }
    }

    // Open the contactor
    AfbSubCall::call_sync(
        request.get_apiv4(),
        ctx.config.charge_api,
        "remote_power",
        false,
    )?;

    request.reply(
        josev::StopChargingResponse {
            evse_id: arg.evse_id.clone(),
            status: josev::MessageStatus::Accepted,
        },
        0,
    );
    Ok(())
}

fn on_cp_pwm(request: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let arg: &josev::CpPwmRequest = args.get::<&josev::CpPwmRequest>(0)?;
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;

    {
        let ctx = ctx.shared.read().unwrap();

        if &ctx.cs_parameters.parameters[0].evse_id != &arg.evse_id {
            // ignore requests for another EVSE ID
            return Ok(());
        }
    }

    // CP PWM should be requested only for HLC (5% PWM)
    // which is already taken care of by the M4 firmware
    // So we always answer Valid here
    request.reply(
        josev::CpPwmResponse {
            evse_id: arg.evse_id.clone(),
            status: josev::CpPwmResponseStatus::Valid,
            info: None,
        },
        0,
    );
    Ok(())
}

fn on_meter_values(
    request: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    afb_log_msg!(Debug, request.get_apiv4(), "METER_VALUES");
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;
    let config = &ctx.config;

    let req: &josev::MeterValuesRequest = args.get::<&josev::MeterValuesRequest>(0)?;

    // read voltage, current and total energy
    let voltage = AfbSubCall::call_sync(
        request.get_apiv4(),
        config.meter_api,
        "tension",
        EnergyAction::READ,
    )?;
    let voltage: &MeterDataSet = voltage.get_onsuccess::<&MeterDataSet>(0)?;
    let current = AfbSubCall::call_sync(
        request.get_apiv4(),
        config.meter_api,
        "current",
        EnergyAction::READ,
    )?;
    let current: &MeterDataSet = current.get_onsuccess::<&MeterDataSet>(0)?;
    let energy = AfbSubCall::call_sync(
        request.get_apiv4(),
        config.meter_api,
        "energy",
        EnergyAction::READ,
    )?;
    let energy: &MeterDataSet = energy.get_onsuccess::<&MeterDataSet>(0)?;

    let response = josev::MeterValuesResponse {
        evse_id: req.evse_id.clone(),
        timestamp: req.timestamp.clone(),
        voltage: josev::MeterValuesUpdateVoltage {
            l1: voltage.l1 as f32 / 1000.0,
            l2: voltage.l2 as f32 / 1000.0,
            l3: voltage.l3 as f32 / 1000.0,
        },
        current: josev::MeterValuesUpdateCurrent {
            l1: current.l1 as f32 / 1000.0,
            l2: current.l2 as f32 / 1000.0,
            l3: current.l3 as f32 / 1000.0,
        },
        power_factor: 1.0,
        dc_current: None,
        dc_voltage: None,
        frequency: 50.0,
        total_active_energy_imported: energy.total as f32,
        total_active_energy_exported: None,
        total_reactive_energy_imported: 0.0,
        total_reactive_energy_exported: None,
        soc: None,
        signed_meter_values: None,
    };

    request.reply(response, 0);
    Ok(())
}

fn on_force_cp_state(
    request: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    afb_log_msg!(Debug, request.get_apiv4(), "METER_VALUES");
    let ctx: &SharedContext = ctx.get_ref::<SharedContext>()?;

    let arg: JsoncObj = args.get::<JsoncObj>(0)?;
    if let Some(cp) = arg.optional::<&str>("cp")? {
        let mut ctx = ctx.shared.write().unwrap();
        match cp {
            "A1" => ctx.forced_charging_state = Some(josev::ControlPilotState::A1),
            "A2" => ctx.forced_charging_state = Some(josev::ControlPilotState::A2),
            "B1" => ctx.forced_charging_state = Some(josev::ControlPilotState::B1),
            "B2" => ctx.forced_charging_state = Some(josev::ControlPilotState::B2),
            "C1" => ctx.forced_charging_state = Some(josev::ControlPilotState::C1),
            "C2" => ctx.forced_charging_state = Some(josev::ControlPilotState::C2),
            "E" => ctx.forced_charging_state = Some(josev::ControlPilotState::E),
            "F" => ctx.forced_charging_state = Some(josev::ControlPilotState::F),
            _ => {}
        }

        if let Some(charging_state) = ctx.forced_charging_state {
            ctx.cp_status_event.push(josev::CpStatusUpdate {
                evse_id: ctx.cs_parameters.parameters[0].evse_id.clone(),
                connector_id: ctx.cs_parameters.parameters[0].connectors[0].id,
                state: charging_state,
                max_voltage: None,
                min_voltage: None,
                duty_cycle: None,
            });
        }
    }
    if let Some(closed) = arg.optional::<bool>("closed_contactor")? {
        let mut ctx = ctx.shared.write().unwrap();
        ctx.forced_contactor_closed = Some(closed);
        ctx.contactor_status_event
            .push(josev::CsContactorStatusUpdate {
                evse_id: ctx.cs_parameters.parameters[0].evse_id.clone(),
                status: {
                    if closed {
                        josev::CsContactorStatusResponseStatus::Closed
                    } else {
                        josev::CsContactorStatusResponseStatus::Opened
                    }
                },
                info: None,
            });
    }

    request.reply(AFB_NO_DATA, 0);
    Ok(())
}
const JOSEV_API: &str = "josev";

pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    // Custom type converters
    auth_registers()?;
    chmgr_registers()?;
    engy_registers()?;
    slac_registers()?;
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

    let device_model = jconf.optional::<JsoncObj>("device_model")?;
    let device_model: Option<josev::DeviceModelResponse> = if device_model.is_some() {
        serde_json::from_str(&device_model.unwrap().to_string())
            .or_else(|error| afb_error!(JOSEV_API, "'device_model' malformed: {}", error))?
    } else {
        None
    };

    let charge_api = jconf.get::<&'static str>("charge_api")?;
    let meter_api = jconf.get::<&'static str>("meter_api")?;
    let auth_api = jconf.get::<&'static str>("auth_api")?;

    let cp_status_event = AfbEvent::new("cp_status");
    let authorization_event = AfbEvent::new("authorization");
    let contactor_status_event = AfbEvent::new("cs_contactor_status");

    let charging_state = josev::ControlPilotState::A1;

    // copy evse_id as immutable configuration
    let evse_id = cs_parameters.parameters[0].evse_id.clone();

    let config = ApiUserData {
        charge_api,
        auth_api,
        meter_api,
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
            device_model,
            contactor_closed: false,
            forced_charging_state: None,
            forced_contactor_closed: None,
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

    let transaction_status_handler = AfbEvtHandler::new("transaction-status-evt")
        .set_pattern(to_static_str(
            "from_mqtt/event/transaction_status".to_owned(),
        ))
        .set_callback(on_transaction_status)
        .set_context(shared_context.clone())
        .finalize()?;

    let slac_status_handler = AfbEvtHandler::new("slac-status-evt")
        .set_pattern(to_static_str("from_mqtt/event/slac_status".to_owned()))
        .set_callback(on_slac_status)
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

    let device_model_verb = AfbVerb::new("device_model")
        .set_callback(on_device_model)
        .set_context(shared_context.clone())
        .finalize()?;

    let meter_values_verb = AfbVerb::new("meter_values")
        .set_callback(on_meter_values)
        .set_context(shared_context.clone())
        .finalize()?;

    let stop_charging_verb = AfbVerb::new("stop_charging")
        .set_callback(on_stop_charging)
        .set_context(shared_context.clone())
        .finalize()?;

    let cp_pwm_verb = AfbVerb::new("cp_pwm")
        .set_callback(on_cp_pwm)
        .set_context(shared_context.clone())
        .finalize()?;

    // for debugging
    let force_cp_state_verb = AfbVerb::new("force_cp_state")
        .set_callback(on_force_cp_state)
        .set_context(shared_context.clone())
        .finalize()?;

    api.add_evt_handler(charge_handler);
    api.add_event(cp_status_event);
    api.add_event(authorization_event);
    api.add_event(contactor_status_event);
    api.add_verb(subscribe_verb);
    api.add_evt_handler(mqtt_handler);
    api.add_evt_handler(hlc_charging_handler);
    api.add_evt_handler(transaction_status_handler);
    api.add_evt_handler(slac_status_handler);

    api.add_verb(contactor_status_verb);
    api.add_verb(status_and_limits_verb);
    api.add_verb(cs_parameters_verb);
    api.add_verb(device_model_verb);
    api.add_verb(meter_values_verb);
    api.add_verb(stop_charging_verb);
    api.add_verb(cp_pwm_verb);

    api.add_verb(force_cp_state_verb);

    Ok(api.finalize()?)
}

AfbBindingRegister!(binding_init);
