use afbv4::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};
use time::OffsetDateTime;

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

AfbDataConverter!(authorization_request, AuthorizationRequest);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AuthorizationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    pub token_type: AuthorizationTokenType,
}

AfbDataConverter!(authorization_response, AuthorizationResponse);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AuthorizationResponse {
    pub status: AuthorizationStatus,
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

AfbDataConverter!(cs_status_and_limits_response, CsStatusAndLimitsResponse);
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

AfbDataConverter!(cs_parameters_response, CsParametersResponse);
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MeterValuesUpdateVoltage {
    pub l1: f32,
    pub l2: f32,
    pub l3: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MeterValuesUpdateCurrent {
    pub l1: f32,
    pub l2: f32,
    pub l3: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SignedMeterValuesUpdate {
    pub measurand: SignedMeterValuesMeasurand,
    pub signed_meter_data: String,
    pub signing_method: String,
    pub encoding_method: String,
    pub public_key: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, IntoStaticStr, Copy)]
pub enum SignedMeterValuesMeasurand {
    #[serde(rename = "Current.Export")]
    CurrentExport,
    #[serde(rename = "Current.Import")]
    CurrentImport,
    #[serde(rename = "Current.Offered")]
    CurrentOffered,
    #[serde(rename = "Energy.Active.Export.Register")]
    EnergyActiveExportRegister,
    #[serde(rename = "Energy.Active.Import.Register")]
    EnergyActiveImportRegister,
    #[serde(rename = "Energy.Reactive.Export.Register")]
    EnergyReactiveExportRegister,
    #[serde(rename = "Energy.Reactive.Import.Register")]
    EnergyReactiveImportRegister,
    #[serde(rename = "Energy.Active.Export.Interval")]
    EnergyActiveExportInterval,
    #[serde(rename = "Energy.Active.Import.Interval")]
    EnergyActiveImportInterval,
    #[serde(rename = "Energy.Active.Net")]
    EnergyActiveNet,
    #[serde(rename = "Energy.Reactive.Export.Interval")]
    EnergyReactiveExportInterval,
    #[serde(rename = "Energy.Reactive.Import.Interval")]
    EnergyReactiveImportInterval,
    #[serde(rename = "Energy.Reactive.Net")]
    EnergyReactiveNet,
    #[serde(rename = "Energy.Apparent.Net")]
    EnergyApparentNet,
    #[serde(rename = "Energy.Apparent.Import")]
    EnergyApparentImport,
    #[serde(rename = "Energy.Apparent.Export")]
    EnergyApparentExport,
    #[serde(rename = "Frequency")]
    Frequency,
    #[serde(rename = "Power.Active.Export")]
    PowerActiveExport,
    #[serde(rename = "Power.Active.Import")]
    PowerActiveImport,
    #[serde(rename = "Power.Factor")]
    PowerFactor,
    #[serde(rename = "Power.Offered")]
    PowerOffered,
    #[serde(rename = "Power.Reactive.Export")]
    PowerReactiveExport,
    #[serde(rename = "Power.Reactive.Import")]
    PowerReactiveImport,
    SoC,
    Voltage,
}

AfbDataConverter!(meter_values_request, MeterValuesRequest);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MeterValuesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

AfbDataConverter!(meter_values_response, MeterValuesResponse);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MeterValuesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub voltage: MeterValuesUpdateVoltage,
    pub current: MeterValuesUpdateCurrent,
    pub power_factor: f32, // TODO: 0 <= x <= 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_current: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_voltage: Option<f32>,
    pub frequency: f32,
    pub total_active_energy_imported: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_active_energy_exported: Option<f32>,
    pub total_reactive_energy_imported: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_reactive_energy_exported: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soc: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_meter_values: Option<Vec<SignedMeterValuesUpdate>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MessageStatus {
    Accepted,
    Rejected,
}

AfbDataConverter!(stop_charging_request, StopChargingRequest);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StopChargingRequest {
    pub evse_id: String,
}

AfbDataConverter!(stop_charging_response, StopChargingResponse);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StopChargingResponse {
    pub evse_id: String,
    pub status: MessageStatus,
}

AfbDataConverter!(slac_status_update, SlacStatusUpdate);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SlacStatusUpdate {
    pub evse_id: String,
    pub run_id: String,
    pub status: SlacStatusUpdateStatus,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SlacStatusUpdateStatus {
    Matching,
    Failed,
    Matched,
    Unmatched,
    BasicCharging,
}

AfbDataConverter!(cp_pwm_request, CpPwmRequest);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CpPwmRequest {
    pub evse_id: String,
    pub hlc: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<f32>, // TODO: constrain minimum 0, precision 1.d.p
    pub error_state: bool,
    pub fault_state: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CpPwmResponseStatus {
    Valid,
    Invalid,
    Error,
}

AfbDataConverter!(cp_pwm_response, CpPwmResponse);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CpPwmResponse {
    pub evse_id: String,
    pub status: CpPwmResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

AfbDataConverter!(device_model_response, DeviceModelResponse);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceModelResponse {
    pub model: String,
    pub vendor: String,
    pub identity: String,
    pub ocpp_csms_url: String,
    pub security_profile: u8,
    pub basic_auth_password: String,
    pub serial_number: String,
    pub firmware_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sim_iccid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sim_imsi: Option<String>,
    pub evses: Vec<DeviceModelEvse>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub components: Vec<DeviceModelComponent>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceModelEvse {
    pub ocpp_id: u32,
    pub iso15118_id: String,
    pub power_kw: f32,
    pub supply_phases: u32,
    pub connectors: Vec<DeviceModelEvseConnector>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceModelEvseConnector {
    pub id: u32,
    pub connector_type: DeviceModelEvseConnectorType,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
pub enum DeviceModelEvseConnectorType {
    #[serde(rename = "cCCS1")]
    #[strum(serialize = "cCCS1")]
    CCcs1,
    #[serde(rename = "cCCS2")]
    #[strum(serialize = "cCCS2")]
    CCcs2,
    #[serde(rename = "cG105")]
    #[strum(serialize = "cG105")]
    CG105,
    #[serde(rename = "cTesla")]
    #[strum(serialize = "cTesla")]
    CTesla,
    #[serde(rename = "cType1")]
    #[strum(serialize = "cType1")]
    CType1,
    #[serde(rename = "cType2")]
    #[strum(serialize = "cType2")]
    CType2,
    #[serde(rename = "s309-1P-16A")]
    #[strum(serialize = "s309-1P-16A")]
    S3091P16A,
    #[serde(rename = "s309-1P-32A")]
    #[strum(serialize = "s309-1P-32A")]
    S3091P32A,
    #[serde(rename = "s309-3P-16A")]
    #[strum(serialize = "s309-3P-16A")]
    S3093P16A,
    #[serde(rename = "s309-3P-32A")]
    #[strum(serialize = "s309-3P-32A")]
    S3093P32A,
    #[serde(rename = "sBS1361")]
    #[strum(serialize = "sBS1361")]
    SBs1361,
    #[serde(rename = "sCEE-7-7")]
    #[strum(serialize = "sCEE-7-7")]
    SCee77,
    #[serde(rename = "sType2")]
    #[strum(serialize = "sType2")]
    SType2,
    #[serde(rename = "sType3")]
    #[strum(serialize = "sType3")]
    SType3,
    Other1PhMax16A,
    Other1PhOver16A,
    Other3Ph,
    Pan,
    #[serde(rename = "wInductive")]
    #[strum(serialize = "wInductive")]
    WInductive,
    #[serde(rename = "wResonant")]
    #[strum(serialize = "wResonant")]
    WResonant,
    Undetermined,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceModelComponent {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub variables: Vec<DeviceModelComponentVariable>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceModelComponentVariable {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutability: Option<DeviceModelComponentVariableMutability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values_list: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
pub enum DeviceModelComponentVariableMutability {
    ReadOnly,
    ReadWrite,
    WriteOnly,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Hash, Eq, Display, Copy)]
pub enum DataType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "decimal")]
    Decimal,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "dateTime")]
    DateTime,
    #[serde(rename = "boolean")]
    Boolean,
    OptionList,
    SequenceList,
    MemberList,
}

pub fn josev_registers() -> Result<(), AfbError> {
    // add binding custom converter
    authorization_update::register()?;
    authorization_request::register()?;
    authorization_response::register()?;
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
    meter_values_request::register()?;
    meter_values_response::register()?;
    stop_charging_request::register()?;
    stop_charging_response::register()?;
    slac_status_update::register()?;
    cp_pwm_request::register()?;
    cp_pwm_response::register()?;
    device_model_response::register()?;
    Ok(())
}
