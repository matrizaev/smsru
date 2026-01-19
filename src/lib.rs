//! Typed Rust client for the SMS.RU HTTP API.
//!
//! This crate is implemented in milestones (see `PLANS.md`). The public API is
//! still evolving, but the design follows `SPEC.md`: a domain layer of strong
//! types, a transport layer for wire-format quirks, and a small client layer
//! orchestrating requests.
//!
//! ```rust,ignore
//! use smsru::{Auth, SmsRuClient};
//!
//! # async fn example() -> Result<(), smsru::SmsRuError> {
//! let client = SmsRuClient::new(Auth::api_id("...")?);
//! // let resp = client.send_sms(...).await?;
//! # Ok(())
//! # }
//! ```
#![forbid(unsafe_code)]

pub mod client;
pub mod domain;
pub mod transport;

pub use client::{Auth, SmsRuClient, SmsRuClientBuilder, SmsRuError};
pub use domain::{
    ApiId, JsonMode, Login, MessageText, PartnerId, Password, PhoneNumber, RawPhoneNumber,
    SendOptions, SendSms, SendSmsResponse, SenderId, SmsResult, Status, StatusCode, TtlMinutes,
    UnixTimestamp, ValidationError,
};
