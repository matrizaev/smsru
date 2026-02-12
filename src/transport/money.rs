use serde::Deserialize;
use serde::de::Error as DeError;

/// Money-like value returned by SMS.RU as either JSON string or JSON number.
///
/// For numbers, the raw JSON token is preserved to avoid formatting drift
/// (`10.00` remains `"10.00"` instead of becoming `"10.0"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportMoney(String);

impl TransportMoney {
    pub fn into_string(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for TransportMoney {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw: Box<serde_json::value::RawValue> = Deserialize::deserialize(deserializer)?;
        let token = raw.get();

        match token.as_bytes().first().copied() {
            Some(b'"') => {
                let parsed = serde_json::from_str::<String>(token).map_err(D::Error::custom)?;
                Ok(Self(parsed))
            }
            Some(b'-' | b'0'..=b'9') => Ok(Self(token.to_owned())),
            _ => Err(D::Error::custom(
                "expected money field to be JSON string or number",
            )),
        }
    }
}
