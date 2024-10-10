use afbv4::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};

AfbDataConverter!(authorization_update, AuthorizationUpdate);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AuthorizationUpdate {
    pub evse_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    pub token_type: AuthorizationTokenType,
    pub status: AuthorizationStatus,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
pub enum AuthorizationTokenType {
    ISO14443,
    ISO15693,
    #[serde(rename = "key_code")]
    #[strum(serialize = "key_code")]
    KeyCode,
    #[serde(rename = "local")]
    #[strum(serialize = "local")]
    Local,
    #[serde(rename = "no_authorization")]
    #[strum(serialize = "no_authorizarion")]
    NoAuthorization,
    #[serde(rename = "mac_address")]
    #[strum(serialize = "mac_address")]
    MacAddress,
    #[serde(rename = "e_maid")]
    #[strum(serialize = "e_maid")]
    EMaid,
    #[serde(rename = "central")]
    #[strum(serialize = "central")]
    Central,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AuthorizationStatus {
    Accepted,
    Rejected,
    Deauthorized,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, IntoStaticStr)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CableCheck {
    Request(CableCheckRequest),
    Response(CableCheckResponse),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CableCheckAction {
    Start,
    Status,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CableCheckStatus {
    Ongoing,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum IsolationLevel {
    Valid,
    Invalid,
    Warning,
    Fault,
    NoIMD,
}

AfbDataConverter!(cable_check_request, CableCheckRequest);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CableCheckRequest {
    pub evse_id: String,
    pub cable_check_action: CableCheckAction,
}

AfbDataConverter!(cable_check_response, CableCheckResponse);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CableCheckResponse {
    pub evse_id: String,
    pub cable_check_status: CableCheckStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolation_level: Option<IsolationLevel>,
}

AfbDataConverter!(cp_status_update, CpStatusUpdate);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CpStatusUpdate {
    pub evse_id: String,
    pub connector_id: u32,
    pub state: ControlPilotState,
    pub max_voltage: Option<f32>,
    pub min_voltage: Option<f32>,
    pub duty_cycle: Option<f32>, // TODO: constraints min 0 max 100
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
pub enum ControlPilotState {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
    D1,
    D2,
    E,
    F,
}

AfbDataConverter!(cs_contactor_status_request, CsContactorStatusRequest);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsContactorStatusRequest {
    pub evse_id: String,
}

AfbDataConverter!(cs_contactor_status_response, CsContactorStatusResponse);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsContactorStatusResponse {
    pub evse_id: String,
    pub status: CsContactorStatusResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CsContactorStatusResponseStatus {
    Closed,
    Opened,
    Error,
}

AfbDataConverter!(cs_contactor_status_update, CsContactorStatusUpdate);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsContactorStatusUpdate {
    pub evse_id: String,
    pub status: CsContactorStatusResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AcSetpoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_active_power: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_reactive_power: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_current: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discharge_active_power: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discharge_reactive_power: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discharge_current: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DcSetpoint {
    pub voltage: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_current: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_power: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discharge_current: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discharge_power: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EvLimits {
    pub maximum_voltage: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_voltage: Option<f32>,
}

AfbDataConverter!(
    power_electronics_setpoint_request,
    PowerElectronicsSetpointRequest
);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PowerElectronicsSetpointRequest {
    pub evse_id: String,
    pub ev_limits: EvLimits,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_precharge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac: Option<AcSetpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<DcSetpoint>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SetPointRequestStatus {
    Accepted,
    Rejected,
}

AfbDataConverter!(
    power_electronics_setpoint_response,
    PowerElectronicsSetpointResponse
);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PowerElectronicsSetpointResponse {
    pub evse_id: String,
    pub status: SetPointRequestStatus,
}

AfbDataConverter!(
    cs_status_and_limits_response,
    CsStatusAndLimitsResponse
);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsStatusAndLimitsResponse {
    pub evses: Vec<CsStatusAndLimitsEvse>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsStatusAndLimitsEvse {
    pub evse_id: String,
    pub status_code: CsStatusAndLimitsStatusCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac: Option<CsStatusAndLimitsAc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac_bpt: Option<CsStatusAndLimitsAcBpt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<CsStatusAndLimitsDc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_bpt: Option<CsStatusAndLimitsDcBpt>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsStatusAndLimitsAc {
    pub max_current: AcMaxCurrent,
    pub nominal_voltage: f32,
    pub rcd_error: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsStatusAndLimitsAcBpt {
    pub evse_max_discharge_power: AcPowerLimit,
    pub evse_min_discharge_power: AcPowerLimit,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AcMaxCurrent {
    pub l1: f32,
    pub l2: f32,
    pub l3: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AcPowerLimit {
    pub l1: f32,
    pub l2: f32,
    pub l3: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsStatusAndLimitsDc {
    pub present_voltage: f32,
    pub present_current: f32,
    pub max_current: f32,
    pub min_current: f32,
    pub max_voltage: f32,
    pub min_voltage: f32,
    pub max_power: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_reg_tolerance: Option<f32>,
    pub peak_current_ripple: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_to_be_delivered: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolation_status: Option<CsStatusAndLimitsDcIsolation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_power_ramp_limit: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsStatusAndLimitsDcBpt {
    pub evse_max_discharge_power: f32,
    pub evse_min_discharge_power: f32,
    pub evse_max_discharge_current: f32,
    pub evse_min_discharge_current: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CsStatusAndLimitsDcIsolation {
    Invalid,
    Valid,
    Warning,
    Fault,
    NoImd,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CsStatusAndLimitsStatusCode {
    EvseNotReady,
    EvseReady,
    EvseShutdown,
    EvseUtilityInterruptEvent,
    EvseIsolationMonitoringActive,
    EvseEmergencyShutdown,
    EvseMalfunction,
}

AfbDataConverter!(
    cs_parameters_response,
    CsParametersResponse
);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsParametersResponse {
    pub sw_version: String,
    pub hw_version: String,
    pub number_of_evses: u32,
    pub parameters: Vec<CsParametersResponseEntry>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsParametersResponseEntry {
    pub evse_id: String,
    pub supports_eim: bool,
    pub network_interface: String,
    pub connectors: Vec<CsParametersConnector>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsParametersConnector {
    pub id: u32,
    pub services: CsParametersConnectorService,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsParametersConnectorService {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac: Option<CsParametersConnectorServiceAc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac_bpt: Option<CsParametersConnectorServiceAcBpt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<CsParametersConnectorServiceDc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_bpt: Option<CsParametersConnectorServiceDcBpt>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
pub enum CsConnectorType {
    #[serde(rename = "AC_single_phase_core")]
    #[strum(serialize = "AC_single_phase_core")]
    ACSinglePhaseCore,
    #[serde(rename = "AC_three_phase_core")]
    #[strum(serialize = "AC_three_phase_core")]
    ACThreePhaseCore,
    #[serde(rename = "DC_core")]
    #[strum(serialize = "DC_core")]
    DCCore,
    #[serde(rename = "DC_extended")]
    #[strum(serialize = "DC_extended")]
    DCExtended,
    #[serde(rename = "DC_combo_core")]
    #[strum(serialize = "DC_combo_core")]
    DCComboCore,
    #[serde(rename = "DC_unique")]
    #[strum(serialize = "DC_unique")]
    DCUnique,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsParametersConnectorServiceCommon {
    pub connector_type: CsConnectorType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_mode: Option<CsParametersControlMode>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsParametersConnectorServiceAc {
    #[serde(flatten)]
    pub common: CsParametersConnectorServiceCommon,
    pub nominal_voltage: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsParametersConnectorServiceAcBpt {
    #[serde(flatten)]
    pub common: CsParametersConnectorServiceCommon,
    pub nominal_voltage: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bpt_channel: Option<CsParametersBptChannel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator_mode: Option<CsParametersGeneratorMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_island_detection_mode: Option<CsParametersGridCodeIslandingDetectionMode>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsParametersConnectorServiceDc {
    #[serde(flatten)]
    pub common: CsParametersConnectorServiceCommon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nominal_voltage: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CsParametersConnectorServiceDcBpt {
    #[serde(flatten)]
    pub common: CsParametersConnectorServiceCommon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nominal_voltage: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bpt_channel: Option<CsParametersBptChannel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator_mode: Option<CsParametersGeneratorMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_island_detection_mode: Option<CsParametersGridCodeIslandingDetectionMode>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CsParametersControlMode {
    Scheduled,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CsParametersBptChannel {
    Unified,
    Separated,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CsParametersGeneratorMode {
    GridFollowing,
    GridForming,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CsParametersGridCodeIslandingDetectionMode {
    Active,
    Passive,
}


pub fn josev_registers() -> Result<(), AfbError> {
    // add binding custom converter
    authorization_update::register()?;
    cable_check_request::register()?;
    cable_check_response::register()?;
    cp_status_update::register()?;
    cs_contactor_status_request::register()?;
    cs_contactor_status_response::register()?;
    cs_contactor_status_update::register()?;
    power_electronics_setpoint_request::register()?;
    power_electronics_setpoint_response::register()?;
    cs_status_and_limits_response::register()?;
    cs_parameters_response::register()?;
    Ok(())
}
