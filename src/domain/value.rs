use crate::domain::validation::ValidationError;

use phonenumber::country;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// SMS.RU `api_id` token.
///
/// Invariant: non-empty after trimming.
pub struct ApiId(String);

impl ApiId {
    /// Form field name used by SMS.RU (`api_id`).
    pub const FIELD: &'static str = "api_id";

    /// Create a validated [`ApiId`].
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrow the validated token.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// SMS.RU account login.
///
/// Invariant: non-empty after trimming.
pub struct Login(String);

impl Login {
    /// Form field name used by SMS.RU (`login`).
    pub const FIELD: &'static str = "login";

    /// Create a validated [`Login`].
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrow the validated login.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// SMS.RU account password.
///
/// Invariant: must not be empty (whitespace is preserved and allowed).
pub struct Password(String);

impl Password {
    /// Form field name used by SMS.RU (`password`).
    pub const FIELD: &'static str = "password";

    /// Create a validated [`Password`].
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(value))
    }

    /// Borrow the password as provided.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Optional partner identifier for SMS.RU (`partner_id`).
///
/// Invariant: non-empty after trimming.
pub struct PartnerId(String);

impl PartnerId {
    /// Form field name used by SMS.RU (`partner_id`).
    pub const FIELD: &'static str = "partner_id";

    /// Create a validated [`PartnerId`].
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrow the validated partner id.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// SMS.RU sender id (`from`).
///
/// Invariant: non-empty after trimming. The value must be enabled in your SMS.RU account.
pub struct SenderId(String);

impl SenderId {
    /// Form field name used by SMS.RU (`from`).
    pub const FIELD: &'static str = "from";

    /// Create a validated [`SenderId`].
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrow the validated sender id.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// SMS message text (`msg`).
///
/// Invariant: non-empty after trimming. The original value (including whitespace) is preserved.
pub struct MessageText(String);

impl MessageText {
    /// Form field name used by SMS.RU (`msg`).
    pub const FIELD: &'static str = "msg";

    /// Create validated message text.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(value))
    }

    /// Borrow the message text as provided.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// SMS.RU message id (`sms_id`) returned by `sms/send`.
///
/// Invariant: non-empty after trimming.
pub struct SmsId(String);

impl SmsId {
    /// Form field name used by SMS.RU (`sms_id`).
    pub const FIELD: &'static str = "sms_id";

    /// Create a validated [`SmsId`].
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrow the validated sms id.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// SMS.RU call-auth check id (`check_id`) returned by `callcheck/add`.
///
/// Invariant: non-empty after trimming.
pub struct CallCheckId(String);

impl CallCheckId {
    /// Form field name used by SMS.RU (`check_id`).
    pub const FIELD: &'static str = "check_id";

    /// Create a validated [`CallCheckId`].
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrow the validated check id.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Unvalidated phone number as sent to SMS.RU (`to`).
///
/// Invariant: non-empty after trimming. This type does not normalize; if you want E.164
/// normalization, parse into [`PhoneNumber`] and convert it into [`RawPhoneNumber`].
pub struct RawPhoneNumber(String);

impl RawPhoneNumber {
    /// Form field name used by SMS.RU (`to`).
    pub const FIELD: &'static str = "to";

    /// Create a validated (non-empty) raw phone number.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Raw (trimmed) value as sent to SMS.RU.
    pub fn raw(&self) -> &str {
        &self.0
    }
}

impl From<PhoneNumber> for RawPhoneNumber {
    /// Convert an already-parsed phone number to a normalized raw value (E.164).
    fn from(value: PhoneNumber) -> Self {
        // Preserve E.164 normalization semantics for opt-in `PhoneNumber`.
        Self(value.e164)
    }
}

#[derive(Debug, Clone)]
/// Parsed phone number with an E.164 representation.
///
/// Equality, ordering, and hashing are based on the E.164 form.
pub struct PhoneNumber {
    raw: String,
    e164: String,
    parsed: phonenumber::PhoneNumber,
}

impl PhoneNumber {
    /// Form field name used by SMS.RU (`to`).
    pub const FIELD: &'static str = "to";

    /// Parse and normalize a phone number into E.164.
    ///
    /// `default_region` is used when the input does not contain an explicit country prefix.
    pub fn parse(
        default_region: Option<country::Id>,
        input: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let input = input.into();
        let raw = input.trim().to_owned();
        if raw.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }

        let parsed = phonenumber::parse(default_region, &raw)
            .map_err(|_| ValidationError::InvalidPhoneNumber { input: raw.clone() })?;

        let e164 = phonenumber::format(&parsed)
            .mode(phonenumber::Mode::E164)
            .to_string();

        Ok(Self { raw, e164, parsed })
    }

    /// Raw input after trimming.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Normalized E.164 representation.
    pub fn e164(&self) -> &str {
        &self.e164
    }

    /// The parsed phone number from the `phonenumber` crate.
    pub fn parsed(&self) -> &phonenumber::PhoneNumber {
        &self.parsed
    }
}

impl PartialEq for PhoneNumber {
    fn eq(&self, other: &Self) -> bool {
        self.e164 == other.e164
    }
}

impl Eq for PhoneNumber {}

impl std::hash::Hash for PhoneNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.e164.hash(state);
    }
}

impl std::cmp::PartialOrd for PhoneNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for PhoneNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.e164.cmp(&other.e164)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Unix timestamp in seconds (`time`).
///
/// This is used by SMS.RU for scheduled sends.
pub struct UnixTimestamp(u64);

impl UnixTimestamp {
    /// Form field name used by SMS.RU (`time`).
    pub const FIELD: &'static str = "time";

    /// Create a timestamp value (no range validation is performed).
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Get the underlying timestamp in seconds.
    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// TTL (time-to-live) for delivery attempts in minutes (`ttl`).
///
/// Invariant: `1..=1440`.
pub struct TtlMinutes(u16);

impl TtlMinutes {
    /// Form field name used by SMS.RU (`ttl`).
    pub const FIELD: &'static str = "ttl";

    /// Minimum allowed TTL value.
    pub const MIN: u16 = 1;
    /// Maximum allowed TTL value.
    pub const MAX: u16 = 1440;

    /// Create a validated TTL value.
    pub fn new(value: u16) -> Result<Self, ValidationError> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(ValidationError::TtlOutOfRange {
                min: Self::MIN,
                max: Self::MAX,
                actual: value,
            });
        }
        Ok(Self(value))
    }

    /// Get the underlying TTL value.
    pub fn value(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// SMS.RU status code.
///
/// This value is preserved as-is even when the code is unknown to this crate.
pub struct StatusCode(i32);

impl StatusCode {
    /// Construct a status code from its integer representation.
    pub fn new(code: i32) -> Self {
        Self(code)
    }

    /// Get the integer code as provided by SMS.RU.
    pub fn as_i32(self) -> i32 {
        self.0
    }

    /// Map this code to a known status code variant, if one exists.
    pub fn known(self) -> Option<KnownStatusCode> {
        KnownStatusCode::from_code(self.0)
    }

    /// Alias for [`StatusCode::known`].
    pub fn known_kind(self) -> Option<KnownStatusCode> {
        self.known()
    }

    /// Returns `true` if this status code is considered retryable by the crate.
    pub fn is_retryable(self) -> bool {
        matches!(
            self.known_kind(),
            Some(kind) if kind.is_retryable()
        )
    }

    /// Returns `true` if this status code represents an authentication/authorization error.
    pub fn is_auth_error(self) -> bool {
        matches!(
            self.known_kind(),
            Some(kind) if kind.is_auth_error()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Call-auth status code returned by `callcheck/status` in `check_status`.
///
/// This value is preserved as-is even when unknown to this crate.
pub struct CallCheckStatusCode(i32);

impl CallCheckStatusCode {
    /// Construct a call-check status code from its integer representation.
    pub fn new(code: i32) -> Self {
        Self(code)
    }

    /// Get the integer code as provided by SMS.RU.
    pub fn as_i32(self) -> i32 {
        self.0
    }

    /// Map this code to a known call-check status variant, if one exists.
    pub fn known_kind(self) -> Option<KnownCallCheckStatusCode> {
        KnownCallCheckStatusCode::from_code(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Known `check_status` values for `callcheck/status`.
pub enum KnownCallCheckStatusCode {
    NotConfirmedYet,
    Confirmed,
    ExpiredOrInvalidCheckId,
}

impl KnownCallCheckStatusCode {
    /// Convert a raw integer call-check status code into a known variant.
    pub fn from_code(code: i32) -> Option<Self> {
        Some(match code {
            400 => Self::NotConfirmedYet,
            401 => Self::Confirmed,
            402 => Self::ExpiredOrInvalidCheckId,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Known SMS.RU status codes supported by this crate.
///
/// Unknown codes are preserved as [`StatusCode`] and return `None` from [`KnownStatusCode::from_code`].
pub enum KnownStatusCode {
    MessageNotFound,
    RequestOkOrQueued,
    BeingDeliveredToOperator,
    SentInTransit,
    Delivered,
    NotDeliveredTtlExpired,
    NotDeliveredDeletedByOperator,
    NotDeliveredPhoneFailure,
    NotDeliveredUnknown,
    NotDeliveredRejected,
    Read,
    NotDeliveredNoRoute,
    InvalidApiId,
    InsufficientFunds,
    InvalidRecipientOrNoRoute,
    EmptyMessageText,
    SenderNotEnabled,
    MessageTooLong,
    DailyLimitExceeded,
    NoDeliveryRoute,
    InvalidTime,
    RecipientInStopList,
    UsedGetInsteadOfPost,
    MethodNotFound,
    MessageNotUtf8,
    TooManyNumbers,
    RecipientAbroadBlocked,
    RecipientInGlobalStopList,
    ForbiddenWordInText,
    MissingDisclaimerPhrase,
    ServiceTemporarilyUnavailable,
    SenderMustMatchBrand,
    ExceededDailyLimitToNumber,
    ExceededIdenticalPerMinute,
    ExceededIdenticalPerDay,
    ExceededRepeatSendLimit,
    InvalidToken,
    InvalidAuth,
    AccountNotConfirmed,
    ConfirmationCodeWrong,
    TooManyConfirmationCodes,
    TooManyWrongAttempts,
    ServerError,
    LimitIpCountryMismatchCategory1,
    LimitIpCountryMismatchCategory2,
    LimitTooManyToCountry,
    LimitTooManyForeignAuth,
    LimitTooManyFromIp,
    LimitHostingProviderIp,
    InvalidEndUserIp,
    LimitTooManyCalls,
    CountryBlocked,
    CallbackUrlInvalid,
    CallbackHandlerNotFound,
    CallCheckNotConfirmedYet,
    CallCheckConfirmed,
    CallCheckExpiredOrInvalidCheckId,
}

impl KnownStatusCode {
    /// Convert a raw SMS.RU integer code into a known variant.
    pub fn from_code(code: i32) -> Option<Self> {
        Some(match code {
            -1 => Self::MessageNotFound,
            100 => Self::RequestOkOrQueued,
            101 => Self::BeingDeliveredToOperator,
            102 => Self::SentInTransit,
            103 => Self::Delivered,
            104 => Self::NotDeliveredTtlExpired,
            105 => Self::NotDeliveredDeletedByOperator,
            106 => Self::NotDeliveredPhoneFailure,
            107 => Self::NotDeliveredUnknown,
            108 => Self::NotDeliveredRejected,
            110 => Self::Read,
            150 => Self::NotDeliveredNoRoute,
            200 => Self::InvalidApiId,
            201 => Self::InsufficientFunds,
            202 => Self::InvalidRecipientOrNoRoute,
            203 => Self::EmptyMessageText,
            204 => Self::SenderNotEnabled,
            205 => Self::MessageTooLong,
            206 => Self::DailyLimitExceeded,
            207 => Self::NoDeliveryRoute,
            208 => Self::InvalidTime,
            209 => Self::RecipientInStopList,
            210 => Self::UsedGetInsteadOfPost,
            211 => Self::MethodNotFound,
            212 => Self::MessageNotUtf8,
            213 => Self::TooManyNumbers,
            214 => Self::RecipientAbroadBlocked,
            215 => Self::RecipientInGlobalStopList,
            216 => Self::ForbiddenWordInText,
            217 => Self::MissingDisclaimerPhrase,
            220 => Self::ServiceTemporarilyUnavailable,
            221 => Self::SenderMustMatchBrand,
            230 => Self::ExceededDailyLimitToNumber,
            231 => Self::ExceededIdenticalPerMinute,
            232 => Self::ExceededIdenticalPerDay,
            233 => Self::ExceededRepeatSendLimit,
            300 => Self::InvalidToken,
            301 => Self::InvalidAuth,
            302 => Self::AccountNotConfirmed,
            303 => Self::ConfirmationCodeWrong,
            304 => Self::TooManyConfirmationCodes,
            305 => Self::TooManyWrongAttempts,
            500 => Self::ServerError,
            501 => Self::LimitIpCountryMismatchCategory1,
            502 => Self::LimitIpCountryMismatchCategory2,
            503 => Self::LimitTooManyToCountry,
            504 => Self::LimitTooManyForeignAuth,
            505 => Self::LimitTooManyFromIp,
            506 => Self::LimitHostingProviderIp,
            507 => Self::InvalidEndUserIp,
            508 => Self::LimitTooManyCalls,
            550 => Self::CountryBlocked,
            901 => Self::CallbackUrlInvalid,
            902 => Self::CallbackHandlerNotFound,
            400 => Self::CallCheckNotConfirmedYet,
            401 => Self::CallCheckConfirmed,
            402 => Self::CallCheckExpiredOrInvalidCheckId,
            _ => return None,
        })
    }

    /// Whether this status is likely transient and can be retried.
    pub fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::ServiceTemporarilyUnavailable
                | Self::TooManyConfirmationCodes
                | Self::TooManyWrongAttempts
                | Self::ServerError
        )
    }

    /// Whether this status indicates invalid/expired credentials.
    pub fn is_auth_error(self) -> bool {
        matches!(
            self,
            Self::InvalidApiId | Self::InvalidToken | Self::InvalidAuth | Self::AccountNotConfirmed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_newtypes_trim_or_validate() {
        let api_id = ApiId::new("  key ").unwrap();
        assert_eq!(api_id.as_str(), "key");
        assert!(ApiId::new("  ").is_err());

        let login = Login::new(" user ").unwrap();
        assert_eq!(login.as_str(), "user");
        assert!(Login::new("").is_err());

        let password = Password::new(" secret ").unwrap();
        assert_eq!(password.as_str(), " secret ");
        assert!(Password::new("").is_err());

        let sender = SenderId::new(" sender ").unwrap();
        assert_eq!(sender.as_str(), "sender");

        let partner = PartnerId::new(" partner ").unwrap();
        assert_eq!(partner.as_str(), "partner");

        let msg = MessageText::new(" hi ").unwrap();
        assert_eq!(msg.as_str(), " hi ");
        assert!(MessageText::new("  ").is_err());

        let sms_id = SmsId::new(" 000000-000001 ").unwrap();
        assert_eq!(sms_id.as_str(), "000000-000001");
        assert!(SmsId::new("  ").is_err());

        let check_id = CallCheckId::new(" 201737-542 ").unwrap();
        assert_eq!(check_id.as_str(), "201737-542");
        assert!(CallCheckId::new("  ").is_err());
    }

    #[test]
    fn raw_phone_number_trims_and_exposes_raw() {
        let raw = RawPhoneNumber::new(" +79251234567 ").unwrap();
        assert_eq!(raw.raw(), "+79251234567");
        assert!(RawPhoneNumber::new("").is_err());
    }

    #[test]
    fn phone_number_parsing_and_equality_use_e164() {
        let p1 = PhoneNumber::parse(None, "+79251234567").unwrap();
        let p2 = PhoneNumber::parse(None, "+7 925 123-45-67").unwrap();
        assert_eq!(p1, p2);
        assert_eq!(p1.e164(), "+79251234567");
        assert_eq!(p1.raw(), "+79251234567");

        let raw: RawPhoneNumber = p1.clone().into();
        assert_eq!(raw.raw(), "+79251234567");
        assert!(PhoneNumber::parse(None, "not-a-number").is_err());
    }

    #[test]
    fn ttl_minutes_enforces_range() {
        assert!(TtlMinutes::new(TtlMinutes::MIN).is_ok());
        assert!(TtlMinutes::new(TtlMinutes::MAX).is_ok());
        assert!(TtlMinutes::new(0).is_err());
        assert!(TtlMinutes::new(TtlMinutes::MAX + 1).is_err());
    }

    #[test]
    fn status_code_knows_retryable_and_auth_errors() {
        let retryable = StatusCode::new(220);
        assert!(retryable.is_retryable());
        assert!(!retryable.is_auth_error());

        let auth = StatusCode::new(301);
        assert!(auth.is_auth_error());
        assert!(!auth.is_retryable());

        let unknown = StatusCode::new(9999);
        assert!(unknown.known().is_none());
        assert!(!unknown.is_retryable());
        assert!(!unknown.is_auth_error());
    }

    #[test]
    fn call_check_status_code_known_mapping() {
        let pending = CallCheckStatusCode::new(400);
        assert_eq!(
            pending.known_kind(),
            Some(KnownCallCheckStatusCode::NotConfirmedYet)
        );

        let confirmed = CallCheckStatusCode::new(401);
        assert_eq!(
            confirmed.known_kind(),
            Some(KnownCallCheckStatusCode::Confirmed)
        );

        let unknown = CallCheckStatusCode::new(9999);
        assert_eq!(unknown.known_kind(), None);
    }
}
