//! Domain layer: strong types with validation and invariants (no I/O).

mod request;
mod response;
mod validation;
mod value;

pub use request::JsonMode;
pub use request::{SEND_SMS_MAX_RECIPIENTS, SendOptions, SendSms};
pub use response::{SendSmsResponse, SmsResult, Status};
pub use validation::ValidationError;
pub use value::{
    ApiId, Login, MessageText, PartnerId, Password, PhoneNumber, RawPhoneNumber, SenderId,
    StatusCode, TtlMinutes, UnixTimestamp,
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn api_id_rejects_empty() {
        assert!(matches!(
            ApiId::new("   "),
            Err(ValidationError::Empty {
                field: ApiId::FIELD
            })
        ));
    }

    #[test]
    fn password_rejects_empty() {
        assert!(matches!(
            Password::new(""),
            Err(ValidationError::Empty {
                field: Password::FIELD
            })
        ));
    }

    #[test]
    fn phone_number_parses_with_region_and_trims() {
        let pn = PhoneNumber::parse(Some(phonenumber::country::Id::RU), " 79251234567 ").unwrap();
        assert_eq!(pn.raw(), "79251234567");
    }

    #[test]
    fn raw_phone_number_from_phone_number_uses_e164() {
        let pn = PhoneNumber::parse(Some(phonenumber::country::Id::RU), "79251234567").unwrap();
        let raw: RawPhoneNumber = pn.into();
        assert_eq!(raw.raw(), "+79251234567");
    }

    #[test]
    fn ttl_minutes_range_is_enforced() {
        assert!(TtlMinutes::new(0).is_err());
        assert!(TtlMinutes::new(1).is_ok());
        assert!(TtlMinutes::new(1440).is_ok());
        assert!(TtlMinutes::new(1441).is_err());
    }

    #[test]
    fn send_sms_recipient_limit_is_enforced() {
        let pn = RawPhoneNumber::new("79251234567").unwrap();
        let msg = MessageText::new("hi").unwrap();
        let recipients = vec![pn; SEND_SMS_MAX_RECIPIENTS + 1];
        let err = SendSms::to_many(recipients, msg, SendOptions::default()).unwrap_err();
        assert!(matches!(err, ValidationError::TooManyRecipients { .. }));
    }

    #[test]
    fn per_recipient_requires_non_empty() {
        let err = SendSms::per_recipient(BTreeMap::new(), SendOptions::default()).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::Empty {
                field: RawPhoneNumber::FIELD
            }
        ));
    }

    #[test]
    fn status_code_known_mapping() {
        let code = StatusCode::new(100);
        assert_eq!(
            code.known(),
            Some(super::value::KnownStatusCode::RequestOkOrQueued)
        );

        let unknown = StatusCode::new(999_999);
        assert_eq!(unknown.known(), None);
    }
}
