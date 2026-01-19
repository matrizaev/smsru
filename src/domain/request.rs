use std::collections::BTreeMap;
use std::net::IpAddr;

use crate::domain::validation::ValidationError;
use crate::domain::value::{
    MessageText, PartnerId, RawPhoneNumber, SenderId, TtlMinutes, UnixTimestamp,
};

pub const SEND_SMS_MAX_RECIPIENTS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JsonMode {
    #[default]
    Json,
    Plain,
}

#[derive(Debug, Clone, Default)]
pub struct SendOptions {
    pub json: JsonMode,
    pub from: Option<SenderId>,
    pub ip: Option<IpAddr>,
    pub time: Option<UnixTimestamp>,
    pub ttl: Option<TtlMinutes>,
    pub daytime: bool,
    pub translit: bool,
    pub test: bool,
    pub partner_id: Option<PartnerId>,
}

#[derive(Debug, Clone)]
pub enum SendSms {
    ToMany(ToMany),
    PerRecipient(PerRecipient),
}

#[derive(Debug, Clone)]
pub struct ToMany {
    recipients: Vec<RawPhoneNumber>,
    msg: MessageText,
    options: SendOptions,
}

#[derive(Debug, Clone)]
pub struct PerRecipient {
    messages: BTreeMap<RawPhoneNumber, MessageText>,
    options: SendOptions,
}

impl SendSms {
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
    pub fn recipients(&self) -> &[RawPhoneNumber] {
        &self.recipients
    }

    pub fn msg(&self) -> &MessageText {
        &self.msg
    }

    pub fn options(&self) -> &SendOptions {
        &self.options
    }
}

impl PerRecipient {
    pub fn messages(&self) -> &BTreeMap<RawPhoneNumber, MessageText> {
        &self.messages
    }

    pub fn options(&self) -> &SendOptions {
        &self.options
    }
}
