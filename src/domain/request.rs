use std::collections::BTreeMap;
use std::net::IpAddr;

use crate::domain::validation::ValidationError;
use crate::domain::value::{
    MessageText, PartnerId, RawPhoneNumber, SenderId, TtlMinutes, UnixTimestamp,
};

/// SMS.RU "send SMS" API limit: maximum number of recipients per request.
pub const SEND_SMS_MAX_RECIPIENTS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Response format mode requested from SMS.RU.
///
/// The client currently supports only [`JsonMode::Json`].
pub enum JsonMode {
    #[default]
    /// Request JSON responses (`json=1`).
    Json,
    /// Request plain-text responses (`json=0`).
    Plain,
}

#[derive(Debug, Clone, Default)]
/// Optional parameters for the "send SMS" request.
///
/// These map to SMS.RU form fields; most are optional and default to "not set".
pub struct SendOptions {
    /// Response format requested from SMS.RU (defaults to JSON).
    pub json: JsonMode,
    /// Sender id (`from=`). Must be enabled in your SMS.RU account.
    pub from: Option<SenderId>,
    /// End user IP (`ip=`), used by SMS.RU for anti-fraud/limits in some modes.
    pub ip: Option<IpAddr>,
    /// Scheduled send time as a unix timestamp in seconds (`time=`).
    pub time: Option<UnixTimestamp>,
    /// Per-recipient TTL in minutes (`ttl=`). See [`TtlMinutes`] for range.
    pub ttl: Option<TtlMinutes>,
    /// Send only during daytime (`daytime=1`).
    pub daytime: bool,
    /// Transliterate message (`translit=1`).
    pub translit: bool,
    /// Test mode (`test=1`): validate request without sending an SMS.
    pub test: bool,
    /// Optional partner identifier (`partner_id=`).
    pub partner_id: Option<PartnerId>,
}

#[derive(Debug, Clone)]
/// A validated "send SMS" request.
///
/// Use [`SendSms::to_many`] to send one message to many recipients, or
/// [`SendSms::per_recipient`] to send per-recipient messages.
pub enum SendSms {
    /// One message to many recipients.
    ToMany(ToMany),
    /// Different messages per recipient.
    PerRecipient(PerRecipient),
}

#[derive(Debug, Clone)]
/// "One message to many recipients" request shape.
pub struct ToMany {
    recipients: Vec<RawPhoneNumber>,
    msg: MessageText,
    options: SendOptions,
}

#[derive(Debug, Clone)]
/// "Per-recipient message" request shape.
pub struct PerRecipient {
    messages: BTreeMap<RawPhoneNumber, MessageText>,
    options: SendOptions,
}

impl SendSms {
    /// Create a "one message to many recipients" request.
    ///
    /// Constraints:
    /// - `recipients` must be non-empty
    /// - `recipients.len()` must be `<= SEND_SMS_MAX_RECIPIENTS` (100)
    pub fn to_many(
        recipients: Vec<RawPhoneNumber>,
        msg: MessageText,
        options: SendOptions,
    ) -> Result<Self, ValidationError> {
        if recipients.is_empty() {
            return Err(ValidationError::Empty {
                field: RawPhoneNumber::FIELD,
            });
        }
        if recipients.len() > SEND_SMS_MAX_RECIPIENTS {
            return Err(ValidationError::TooManyRecipients {
                max: SEND_SMS_MAX_RECIPIENTS,
                actual: recipients.len(),
            });
        }
        Ok(Self::ToMany(ToMany {
            recipients,
            msg,
            options,
        }))
    }

    /// Create a "per-recipient message" request.
    ///
    /// Constraints:
    /// - `messages` must be non-empty
    /// - `messages.len()` must be `<= SEND_SMS_MAX_RECIPIENTS` (100)
    pub fn per_recipient(
        messages: BTreeMap<RawPhoneNumber, MessageText>,
        options: SendOptions,
    ) -> Result<Self, ValidationError> {
        if messages.is_empty() {
            return Err(ValidationError::Empty {
                field: RawPhoneNumber::FIELD,
            });
        }
        if messages.len() > SEND_SMS_MAX_RECIPIENTS {
            return Err(ValidationError::TooManyRecipients {
                max: SEND_SMS_MAX_RECIPIENTS,
                actual: messages.len(),
            });
        }
        Ok(Self::PerRecipient(PerRecipient { messages, options }))
    }
}

impl ToMany {
    /// Recipient phone numbers as provided (not normalized).
    pub fn recipients(&self) -> &[RawPhoneNumber] {
        &self.recipients
    }

    /// Message text (must be non-empty; see [`MessageText`]).
    pub fn msg(&self) -> &MessageText {
        &self.msg
    }

    /// Request options.
    pub fn options(&self) -> &SendOptions {
        &self.options
    }
}

impl PerRecipient {
    /// Per-recipient messages.
    pub fn messages(&self) -> &BTreeMap<RawPhoneNumber, MessageText> {
        &self.messages
    }

    /// Request options.
    pub fn options(&self) -> &SendOptions {
        &self.options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_recipients(count: usize) -> Vec<RawPhoneNumber> {
        (0..count)
            .map(|idx| RawPhoneNumber::new(format!("+792512300{idx:02}")).unwrap())
            .collect()
    }

    #[test]
    fn to_many_rejects_empty_recipients() {
        let msg = MessageText::new("hi").unwrap();
        let err = SendSms::to_many(Vec::new(), msg, SendOptions::default()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::Empty {
                field: RawPhoneNumber::FIELD
            }
        );
    }

    #[test]
    fn to_many_rejects_too_many_recipients() {
        let msg = MessageText::new("hi").unwrap();
        let recipients = make_recipients(SEND_SMS_MAX_RECIPIENTS + 1);
        let err = SendSms::to_many(recipients, msg, SendOptions::default()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::TooManyRecipients {
                max: SEND_SMS_MAX_RECIPIENTS,
                actual: SEND_SMS_MAX_RECIPIENTS + 1
            }
        );
    }

    #[test]
    fn per_recipient_rejects_empty_messages() {
        let err = SendSms::per_recipient(BTreeMap::new(), SendOptions::default()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::Empty {
                field: RawPhoneNumber::FIELD
            }
        );
    }

    #[test]
    fn per_recipient_rejects_too_many_messages() {
        let mut messages = BTreeMap::new();
        for idx in 0..(SEND_SMS_MAX_RECIPIENTS + 1) {
            messages.insert(
                RawPhoneNumber::new(format!("+792512310{idx:02}")).unwrap(),
                MessageText::new("hi").unwrap(),
            );
        }
        let err = SendSms::per_recipient(messages, SendOptions::default()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::TooManyRecipients {
                max: SEND_SMS_MAX_RECIPIENTS,
                actual: SEND_SMS_MAX_RECIPIENTS + 1
            }
        );
    }

    #[test]
    fn to_many_exposes_fields() {
        let recipients = make_recipients(2);
        let msg = MessageText::new("hello").unwrap();
        let options = SendOptions::default();
        let req = SendSms::to_many(recipients.clone(), msg.clone(), options.clone()).unwrap();

        match req {
            SendSms::ToMany(to_many) => {
                assert_eq!(to_many.recipients(), recipients.as_slice());
                assert_eq!(to_many.msg(), &msg);
                assert_eq!(to_many.options().json, options.json);
            }
            SendSms::PerRecipient(_) => panic!("expected to_many request"),
        }
    }

    #[test]
    fn per_recipient_exposes_fields() {
        let mut messages = BTreeMap::new();
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let msg = MessageText::new("hello").unwrap();
        messages.insert(p1.clone(), msg.clone());
        let options = SendOptions::default();

        let req = SendSms::per_recipient(messages.clone(), options.clone()).unwrap();
        match req {
            SendSms::PerRecipient(per_recipient) => {
                assert_eq!(per_recipient.messages(), &messages);
                assert_eq!(per_recipient.options().json, options.json);
            }
            SendSms::ToMany(_) => panic!("expected per_recipient request"),
        }
    }
}
