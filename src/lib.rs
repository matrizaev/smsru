//! Typed Rust client for the SMS.RU HTTP API.
//!
//! This crate is implemented in milestones (see `PLANS.md`). The public API is
//! still evolving, but the design follows `SPEC.md`: a domain layer of strong
//! types, a transport layer for wire-format quirks, and a small client layer
//! orchestrating requests.
//!
//! ```rust,no_run
//! use smsru::{Auth, MessageText, RawPhoneNumber, SendOptions, SendSms, SmsRuClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), smsru::SmsRuError> {
//!     let client = SmsRuClient::new(Auth::api_id("...")?);
//!     let phone = RawPhoneNumber::new("+79251234567")?;
//!     let msg = MessageText::new("hello")?;
//!     let request = SendSms::to_many(vec![phone], msg, SendOptions::default())?;
//!     let _resp = client.send_sms(request).await?;
//!     Ok(())
//! }
//! ```
#![forbid(unsafe_code)]

pub mod client;
pub mod domain;
mod transport;

pub use client::{Auth, SmsRuClient, SmsRuClientBuilder, SmsRuError};
pub use domain::{
    ApiId, JsonMode, KnownStatusCode, Login, MessageText, PartnerId, Password, PhoneNumber,
    RawPhoneNumber, SendOptions, SendSms, SendSmsResponse, SenderId, SmsResult, Status, StatusCode,
    TtlMinutes, UnixTimestamp, ValidationError,
};
