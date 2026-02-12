use std::collections::BTreeMap;
use std::net::IpAddr;

use crate::domain::validation::ValidationError;
use crate::domain::value::{
    CallCheckId, MessageText, PartnerId, RawPhoneNumber, SenderId, SmsId, TtlMinutes, UnixTimestamp,
};

/// SMS.RU "send SMS" API limit: maximum number of recipients per request.
pub const SEND_SMS_MAX_RECIPIENTS: usize = 100;
/// SMS.RU "check cost" API limit: maximum number of recipients per request.
pub const CHECK_COST_MAX_RECIPIENTS: usize = 100;
/// SMS.RU "check status" API limit: maximum number of ids per request.
pub const CHECK_STATUS_MAX_SMS_IDS: usize = 100;

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

#[derive(Debug, Clone, Default)]
/// Optional parameters for the "check cost" request.
///
/// These map to SMS.RU form fields supported by `sms/cost`.
pub struct CheckCostOptions {
    /// Response format requested from SMS.RU (defaults to JSON).
    pub json: JsonMode,
    /// Sender id (`from=`). Must be enabled in your SMS.RU account.
    pub from: Option<SenderId>,
    /// Transliterate message (`translit=1`).
    pub translit: bool,
}

#[derive(Debug, Clone, Default)]
/// Optional parameters for the "start call authentication" request.
pub struct StartCallAuthOptions {
    /// Response format requested from SMS.RU (defaults to JSON).
    pub json: JsonMode,
}

#[derive(Debug, Clone, Default)]
/// Optional parameters for the "check call authentication status" request.
pub struct CheckCallAuthStatusOptions {
    /// Response format requested from SMS.RU (defaults to JSON).
    pub json: JsonMode,
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

#[derive(Debug, Clone)]
/// A validated "check cost" request.
///
/// Use [`CheckCost::to_many`] to estimate one message for many recipients, or
/// [`CheckCost::per_recipient`] to estimate per-recipient messages.
pub enum CheckCost {
    /// One message to many recipients.
    ToMany(CostToMany),
    /// Different messages per recipient.
    PerRecipient(CostPerRecipient),
}

#[derive(Debug, Clone)]
/// "One message to many recipients" request shape for `sms/cost`.
pub struct CostToMany {
    recipients: Vec<RawPhoneNumber>,
    msg: MessageText,
    options: CheckCostOptions,
}

#[derive(Debug, Clone)]
/// "Per-recipient message" request shape for `sms/cost`.
pub struct CostPerRecipient {
    messages: BTreeMap<RawPhoneNumber, MessageText>,
    options: CheckCostOptions,
}

#[derive(Debug, Clone)]
/// A validated "check status" request.
///
/// Use [`CheckStatus::new`] for one or many ids or [`CheckStatus::one`] as a convenience.
pub struct CheckStatus {
    sms_ids: Vec<SmsId>,
}

#[derive(Debug, Clone)]
/// A validated "start call authentication" request.
pub struct StartCallAuth {
    phone: RawPhoneNumber,
    options: StartCallAuthOptions,
}

#[derive(Debug, Clone)]
/// A validated "check call authentication status" request.
pub struct CheckCallAuthStatus {
    check_id: CallCheckId,
    options: CheckCallAuthStatusOptions,
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

impl CheckCost {
    /// Create a "one message to many recipients" cost request.
    ///
    /// Constraints:
    /// - `recipients` must be non-empty
    /// - `recipients.len()` must be `<= CHECK_COST_MAX_RECIPIENTS` (100)
    pub fn to_many(
        recipients: Vec<RawPhoneNumber>,
        msg: MessageText,
        options: CheckCostOptions,
    ) -> Result<Self, ValidationError> {
        if recipients.is_empty() {
            return Err(ValidationError::Empty {
                field: RawPhoneNumber::FIELD,
            });
        }
        if recipients.len() > CHECK_COST_MAX_RECIPIENTS {
            return Err(ValidationError::TooManyRecipients {
                max: CHECK_COST_MAX_RECIPIENTS,
                actual: recipients.len(),
            });
        }
        Ok(Self::ToMany(CostToMany {
            recipients,
            msg,
            options,
        }))
    }

    /// Create a "per-recipient message" cost request.
    ///
    /// Constraints:
    /// - `messages` must be non-empty
    /// - `messages.len()` must be `<= CHECK_COST_MAX_RECIPIENTS` (100)
    pub fn per_recipient(
        messages: BTreeMap<RawPhoneNumber, MessageText>,
        options: CheckCostOptions,
    ) -> Result<Self, ValidationError> {
        if messages.is_empty() {
            return Err(ValidationError::Empty {
                field: RawPhoneNumber::FIELD,
            });
        }
        if messages.len() > CHECK_COST_MAX_RECIPIENTS {
            return Err(ValidationError::TooManyRecipients {
                max: CHECK_COST_MAX_RECIPIENTS,
                actual: messages.len(),
            });
        }
        Ok(Self::PerRecipient(CostPerRecipient { messages, options }))
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

impl CostToMany {
    /// Recipient phone numbers as provided (not normalized).
    pub fn recipients(&self) -> &[RawPhoneNumber] {
        &self.recipients
    }

    /// Message text (must be non-empty; see [`MessageText`]).
    pub fn msg(&self) -> &MessageText {
        &self.msg
    }

    /// Request options.
    pub fn options(&self) -> &CheckCostOptions {
        &self.options
    }
}

impl CostPerRecipient {
    /// Per-recipient messages.
    pub fn messages(&self) -> &BTreeMap<RawPhoneNumber, MessageText> {
        &self.messages
    }

    /// Request options.
    pub fn options(&self) -> &CheckCostOptions {
        &self.options
    }
}

impl CheckStatus {
    /// Create a "check status" request.
    ///
    /// Constraints:
    /// - `sms_ids` must be non-empty
    /// - `sms_ids.len()` must be `<= CHECK_STATUS_MAX_SMS_IDS` (100)
    pub fn new(sms_ids: Vec<SmsId>) -> Result<Self, ValidationError> {
        if sms_ids.is_empty() {
            return Err(ValidationError::Empty {
                field: SmsId::FIELD,
            });
        }
        if sms_ids.len() > CHECK_STATUS_MAX_SMS_IDS {
            return Err(ValidationError::TooManySmsIds {
                max: CHECK_STATUS_MAX_SMS_IDS,
                actual: sms_ids.len(),
            });
        }
        Ok(Self { sms_ids })
    }

    /// Create a "check status" request for one message id.
    pub fn one(sms_id: SmsId) -> Self {
        Self {
            sms_ids: vec![sms_id],
        }
    }

    /// Message ids to query.
    pub fn sms_ids(&self) -> &[SmsId] {
        &self.sms_ids
    }
}

impl StartCallAuth {
    /// Create a "start call authentication" request.
    pub fn new(phone: RawPhoneNumber, options: StartCallAuthOptions) -> Self {
        Self { phone, options }
    }

    /// Phone number expected to place the confirmation call.
    pub fn phone(&self) -> &RawPhoneNumber {
        &self.phone
    }

    /// Request options.
    pub fn options(&self) -> &StartCallAuthOptions {
        &self.options
    }
}

impl CheckCallAuthStatus {
    /// Create a "check call authentication status" request.
    pub fn new(check_id: CallCheckId, options: CheckCallAuthStatusOptions) -> Self {
        Self { check_id, options }
    }

    /// Check id returned by `callcheck/add`.
    pub fn check_id(&self) -> &CallCheckId {
        &self.check_id
    }

    /// Request options.
    pub fn options(&self) -> &CheckCallAuthStatusOptions {
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

    #[test]
    fn check_cost_to_many_rejects_empty_recipients() {
        let msg = MessageText::new("hi").unwrap();
        let err = CheckCost::to_many(Vec::new(), msg, CheckCostOptions::default()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::Empty {
                field: RawPhoneNumber::FIELD
            }
        );
    }

    #[test]
    fn check_cost_to_many_rejects_too_many_recipients() {
        let msg = MessageText::new("hi").unwrap();
        let recipients = make_recipients(CHECK_COST_MAX_RECIPIENTS + 1);
        let err = CheckCost::to_many(recipients, msg, CheckCostOptions::default()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::TooManyRecipients {
                max: CHECK_COST_MAX_RECIPIENTS,
                actual: CHECK_COST_MAX_RECIPIENTS + 1
            }
        );
    }

    #[test]
    fn check_cost_per_recipient_rejects_empty_messages() {
        let err =
            CheckCost::per_recipient(BTreeMap::new(), CheckCostOptions::default()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::Empty {
                field: RawPhoneNumber::FIELD
            }
        );
    }

    #[test]
    fn check_cost_per_recipient_rejects_too_many_messages() {
        let mut messages = BTreeMap::new();
        for idx in 0..(CHECK_COST_MAX_RECIPIENTS + 1) {
            messages.insert(
                RawPhoneNumber::new(format!("+792512330{idx:02}")).unwrap(),
                MessageText::new("hi").unwrap(),
            );
        }
        let err = CheckCost::per_recipient(messages, CheckCostOptions::default()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::TooManyRecipients {
                max: CHECK_COST_MAX_RECIPIENTS,
                actual: CHECK_COST_MAX_RECIPIENTS + 1
            }
        );
    }

    #[test]
    fn check_cost_to_many_exposes_fields() {
        let recipients = make_recipients(2);
        let msg = MessageText::new("hello").unwrap();
        let options = CheckCostOptions::default();
        let req = CheckCost::to_many(recipients.clone(), msg.clone(), options.clone()).unwrap();

        match req {
            CheckCost::ToMany(to_many) => {
                assert_eq!(to_many.recipients(), recipients.as_slice());
                assert_eq!(to_many.msg(), &msg);
                assert_eq!(to_many.options().json, options.json);
            }
            CheckCost::PerRecipient(_) => panic!("expected to_many request"),
        }
    }

    #[test]
    fn check_cost_per_recipient_exposes_fields() {
        let mut messages = BTreeMap::new();
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let msg = MessageText::new("hello").unwrap();
        messages.insert(p1.clone(), msg.clone());
        let options = CheckCostOptions::default();

        let req = CheckCost::per_recipient(messages.clone(), options.clone()).unwrap();
        match req {
            CheckCost::PerRecipient(per_recipient) => {
                assert_eq!(per_recipient.messages(), &messages);
                assert_eq!(per_recipient.options().json, options.json);
            }
            CheckCost::ToMany(_) => panic!("expected per_recipient request"),
        }
    }

    #[test]
    fn check_status_rejects_empty_sms_ids() {
        let err = CheckStatus::new(Vec::new()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::Empty {
                field: SmsId::FIELD
            }
        );
    }

    #[test]
    fn check_status_rejects_too_many_sms_ids() {
        let sms_ids = (0..(CHECK_STATUS_MAX_SMS_IDS + 1))
            .map(|idx| SmsId::new(format!("000000-{:06}", idx)).unwrap())
            .collect::<Vec<_>>();
        let err = CheckStatus::new(sms_ids).unwrap_err();
        assert_eq!(
            err,
            ValidationError::TooManySmsIds {
                max: CHECK_STATUS_MAX_SMS_IDS,
                actual: CHECK_STATUS_MAX_SMS_IDS + 1
            }
        );
    }

    #[test]
    fn check_status_one_creates_single_id_request() {
        let sms_id = SmsId::new("000000-000001").unwrap();
        let request = CheckStatus::one(sms_id.clone());
        assert_eq!(request.sms_ids(), &[sms_id]);
    }

    #[test]
    fn check_status_new_exposes_sms_ids() {
        let ids = vec![
            SmsId::new("000000-000001").unwrap(),
            SmsId::new("000000-000002").unwrap(),
        ];
        let request = CheckStatus::new(ids.clone()).unwrap();
        assert_eq!(request.sms_ids(), ids.as_slice());
    }

    #[test]
    fn start_call_auth_exposes_fields() {
        let phone = RawPhoneNumber::new("79251234567").unwrap();
        let request = StartCallAuth::new(phone.clone(), StartCallAuthOptions::default());
        assert_eq!(request.phone(), &phone);
        assert_eq!(request.options().json, JsonMode::Json);
    }

    #[test]
    fn check_call_auth_status_exposes_fields() {
        let check_id = CallCheckId::new("201737-542").unwrap();
        let request =
            CheckCallAuthStatus::new(check_id.clone(), CheckCallAuthStatusOptions::default());
        assert_eq!(request.check_id(), &check_id);
        assert_eq!(request.options().json, JsonMode::Json);
    }
}
