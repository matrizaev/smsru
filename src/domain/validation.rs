use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    Empty { field: &'static str },
    TooManyRecipients { max: usize, actual: usize },
    InvalidPhoneNumber { input: String },
    TtlOutOfRange { min: u16, max: u16, actual: u16 },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty { field } => write!(f, "{field} must not be empty"),
            Self::TooManyRecipients { max, actual } => {
                write!(f, "too many recipients: {actual} (max {max})")
            }
            Self::InvalidPhoneNumber { input } => write!(f, "invalid phone number: {input}"),
            Self::TtlOutOfRange { min, max, actual } => {
                write!(
                    f,
                    "ttl minutes out of range: {actual} (expected {min}..={max})"
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::ValidationError;

    #[test]
    fn display_messages_are_human_readable() {
        let err = ValidationError::Empty { field: "to" };
        assert_eq!(err.to_string(), "to must not be empty");

        let err = ValidationError::TooManyRecipients {
            max: 2,
            actual: 3,
        };
        assert_eq!(err.to_string(), "too many recipients: 3 (max 2)");

        let err = ValidationError::InvalidPhoneNumber {
            input: "bad".to_owned(),
        };
        assert_eq!(err.to_string(), "invalid phone number: bad");

        let err = ValidationError::TtlOutOfRange {
            min: 1,
            max: 10,
            actual: 11,
        };
        assert_eq!(
            err.to_string(),
            "ttl minutes out of range: 11 (expected 1..=10)"
        );
    }
}
