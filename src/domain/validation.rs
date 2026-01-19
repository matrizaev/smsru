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
