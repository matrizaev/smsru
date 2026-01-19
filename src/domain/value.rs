use crate::domain::validation::ValidationError;

use phonenumber::country;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiId(String);

impl ApiId {
    pub const FIELD: &'static str = "api_id";

    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Login(String);

impl Login {
    pub const FIELD: &'static str = "login";

    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Password(String);

impl Password {
    pub const FIELD: &'static str = "password";

    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartnerId(String);

impl PartnerId {
    pub const FIELD: &'static str = "partner_id";

    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SenderId(String);

impl SenderId {
    pub const FIELD: &'static str = "from";

    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageText(String);

impl MessageText {
    pub const FIELD: &'static str = "msg";

    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawPhoneNumber(String);

impl RawPhoneNumber {
    pub const FIELD: &'static str = "to";

    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty { field: Self::FIELD });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn raw(&self) -> &str {
        &self.0
    }
}

impl From<PhoneNumber> for RawPhoneNumber {
    fn from(value: PhoneNumber) -> Self {
        // Preserve E.164 normalization semantics for opt-in `PhoneNumber`.
        Self(value.e164)
    }
}

#[derive(Debug, Clone)]
pub struct PhoneNumber {
    raw: String,
    e164: String,
    parsed: phonenumber::PhoneNumber,
}

impl PhoneNumber {
    pub const FIELD: &'static str = "to";

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

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn e164(&self) -> &str {
        &self.e164
    }

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
pub struct UnixTimestamp(u64);

impl UnixTimestamp {
    pub const FIELD: &'static str = "time";

    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TtlMinutes(u16);

impl TtlMinutes {
    pub const FIELD: &'static str = "ttl";

    pub const MIN: u16 = 1;
    pub const MAX: u16 = 1440;

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

    pub fn value(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusCode(i32);

impl StatusCode {
    pub fn new(code: i32) -> Self {
        Self(code)
    }

    pub fn as_i32(self) -> i32 {
        self.0
    }

    pub fn known(self) -> Option<KnownStatusCode> {
        KnownStatusCode::from_code(self.0)
    }

    pub fn known_kind(self) -> Option<KnownStatusCode> {
        self.known()
    }

    pub fn is_retryable(self) -> bool {
        matches!(
            self.known_kind(),
            Some(kind) if kind.is_retryable()
        )
    }

    pub fn is_auth_error(self) -> bool {
        matches!(
            self.known_kind(),
            Some(kind) if kind.is_auth_error()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
}

impl KnownStatusCode {
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
            _ => return None,
        })
    }

    pub fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::ServiceTemporarilyUnavailable
                | Self::TooManyConfirmationCodes
                | Self::TooManyWrongAttempts
                | Self::ServerError
        )
    }

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
}
