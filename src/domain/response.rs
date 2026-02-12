use std::collections::BTreeMap;

use crate::domain::value::{
    CallCheckId, CallCheckStatusCode, CallbackUrl, RawPhoneNumber, SmsId, StatusCode,
};

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

#[derive(Debug, Clone, PartialEq)]
/// Parsed response from the SMS.RU "check cost" API.
///
/// When using [`crate::client::SmsRuClient`], API-level failures (`status != OK`) are returned as
/// [`crate::SmsRuError::Api`] instead of a `CheckCostResponse`.
pub struct CheckCostResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Total request cost as returned by SMS.RU.
    pub total_cost: Option<String>,
    /// Total number of SMS segments as returned by SMS.RU.
    pub total_sms: Option<u32>,
    /// Per-recipient cost results keyed by phone number.
    pub sms: BTreeMap<RawPhoneNumber, SmsCostResult>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Result for a single recipient in the SMS.RU cost response.
pub struct SmsCostResult {
    /// Per-recipient status.
    pub status: Status,
    /// Per-recipient status code.
    pub status_code: StatusCode,
    /// Optional per-recipient status text.
    pub status_text: Option<String>,
    /// Optional per-recipient message cost.
    pub cost: Option<String>,
    /// Optional per-recipient SMS segment count.
    pub sms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed response from the SMS.RU "start call authentication" API.
///
/// When using [`crate::client::SmsRuClient`], API-level failures (`status != OK`) are returned as
/// [`crate::SmsRuError::Api`] instead of a `StartCallAuthResponse`.
pub struct StartCallAuthResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Call-check id used for status polling.
    pub check_id: Option<CallCheckId>,
    /// Number the user must call to confirm ownership.
    pub call_phone: Option<RawPhoneNumber>,
    /// Human-readable call number.
    pub call_phone_pretty: Option<String>,
    /// HTML `callto:` link.
    pub call_phone_html: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed response from the SMS.RU "check call authentication status" API.
///
/// When using [`crate::client::SmsRuClient`], API-level failures (`status != OK`) are returned as
/// [`crate::SmsRuError::Api`] instead of a `CheckCallAuthStatusResponse`.
pub struct CheckCallAuthStatusResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Call-check status code (`400`/`401`/`402`, unknown preserved).
    pub check_status: Option<CallCheckStatusCode>,
    /// Optional call-check status description.
    pub check_status_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed response from methods that only return top-level status fields.
///
/// This shape is used by `auth/check` and status-only mutation methods in
/// other endpoint families.
pub struct StatusOnlyResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed response from `my/balance`.
pub struct BalanceResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Account balance as returned by SMS.RU (string-preserving representation).
    pub balance: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed response from `my/free`.
pub struct FreeUsageResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Total number of free messages available for own number.
    pub total_free: Option<u32>,
    /// Number of free messages used today.
    pub used_today: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed response from `my/limit`.
pub struct LimitUsageResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Daily sending limit configured for the account.
    pub total_limit: Option<u32>,
    /// Number of recipients used today against the daily limit.
    pub used_today: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed response from `my/senders`.
pub struct SendersResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Approved senders configured in SMS.RU account.
    pub senders: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed response from `stoplist/get`.
pub struct StoplistResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Stoplist entries keyed by phone number with associated notes.
    pub stoplist: BTreeMap<RawPhoneNumber, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed response from `callback/add`, `callback/del`, and `callback/get`.
pub struct CallbacksResponse {
    /// Top-level response status.
    pub status: Status,
    /// SMS.RU status code (known + unknown preserved).
    pub status_code: StatusCode,
    /// Optional status text provided by SMS.RU.
    pub status_text: Option<String>,
    /// Configured callback URLs.
    pub callback: Vec<CallbackUrl>,
}
