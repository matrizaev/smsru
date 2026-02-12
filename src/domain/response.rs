use std::collections::BTreeMap;

use crate::domain::value::{RawPhoneNumber, SmsId, StatusCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Top-level status marker used by SMS.RU responses.
pub enum Status {
    /// Request or operation succeeded.
    Ok,
    /// Request or operation failed.
    Error,
}

#[derive(Debug, Clone, PartialEq)]
/// Parsed response from the SMS.RU "send SMS" API.
///
/// When using [`crate::client::SmsRuClient`], API-level failures (`status != OK`) are returned as
/// [`crate::SmsRuError::Api`] instead of a `SendSmsResponse`.
pub struct SendSmsResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Account balance as returned by SMS.RU (format is API-defined).
    pub balance: Option<String>,
    /// Per-recipient results keyed by the raw phone number used in the request.
    pub sms: BTreeMap<RawPhoneNumber, SmsResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Result for a single recipient in the SMS.RU response.
pub struct SmsResult {
    /// Per-recipient status.
    pub status: Status,
    /// Per-recipient status code.
    pub status_code: StatusCode,
    /// Optional per-recipient status text.
    pub status_text: Option<String>,
    /// Optional SMS id assigned by SMS.RU.
    pub sms_id: Option<SmsId>,
}

#[derive(Debug, Clone, PartialEq)]
/// Parsed response from the SMS.RU "check status" API.
///
/// When using [`crate::client::SmsRuClient`], API-level failures (`status != OK`) are returned as
/// [`crate::SmsRuError::Api`] instead of a `CheckStatusResponse`.
pub struct CheckStatusResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Account balance as returned by SMS.RU (format is API-defined).
    pub balance: Option<String>,
    /// Per-sms_id status results keyed by queried sms id.
    pub sms: BTreeMap<SmsId, SmsStatusResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Result for a single sms id in the SMS.RU status response.
pub struct SmsStatusResult {
    /// Per-id status.
    pub status: Status,
    /// Per-id status code.
    pub status_code: StatusCode,
    /// Optional per-id status text.
    pub status_text: Option<String>,
    /// Optional message cost as returned by SMS.RU.
    pub cost: Option<String>,
}
