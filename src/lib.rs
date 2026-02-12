//! Typed Rust client for the SMS.RU HTTP API.
//!
//! This crate is implemented in milestones (see `PLANS.md`). The public API is
//! still evolving, but the design follows `SPEC.md`: a domain layer of strong
//! types, a transport layer for wire-format quirks, and a small client layer
//! orchestrating requests.
//!
//! ```rust,no_run
//! use smsru::{
//!     Auth, CheckStatus, CheckCost, CheckCostOptions, MessageText, RawPhoneNumber, SendOptions,
//!     SendSms, SmsId, SmsRuClient,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), smsru::SmsRuError> {
//!     let client = SmsRuClient::new(Auth::api_id("...")?);
//!
//!     let phone = RawPhoneNumber::new("+79251234567")?;
//!     let msg = MessageText::new("hello")?;
//!     let request = SendSms::to_many(vec![phone], msg, SendOptions::default())?;
//!     let _resp = client.send_sms(request).await?;
//!
//!     let status_req = CheckStatus::one(SmsId::new("000000-000001")?);
//!     let _status = client.check_status(status_req).await?;
//!
//!     let cost_req = CheckCost::to_many(
//!         vec![RawPhoneNumber::new("+79251234567")?],
//!         MessageText::new("hello")?,
//!         CheckCostOptions::default(),
//!     )?;
//!     let _cost = client.check_cost(cost_req).await?;
//!     Ok(())
//! }
//! ```
#![forbid(unsafe_code)]

pub mod client;
pub mod domain;
mod transport;

pub use client::{Auth, SmsRuClient, SmsRuClientBuilder, SmsRuError};
pub use domain::{
    ApiId, CheckCost, CheckCostOptions, CheckCostResponse, CheckStatus, CheckStatusResponse,
    JsonMode, KnownStatusCode, Login, MessageText, PartnerId, Password, PhoneNumber,
    RawPhoneNumber, SendOptions, SendSms, SendSmsResponse, SenderId, SmsCostResult, SmsId,
    SmsResult, SmsStatusResult, Status, StatusCode, TtlMinutes, UnixTimestamp, ValidationError,
};
