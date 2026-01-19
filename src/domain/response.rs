use std::collections::BTreeMap;

use crate::domain::value::{RawPhoneNumber, StatusCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Ok,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SendSmsResponse {
    pub status: Status,
    pub status_code: StatusCode,
    pub status_text: Option<String>,
    pub balance: Option<String>,
    pub sms: BTreeMap<RawPhoneNumber, SmsResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmsResult {
    pub status: Status,
    pub status_code: StatusCode,
    pub status_text: Option<String>,
    pub sms_id: Option<String>,
}
