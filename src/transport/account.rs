use serde::Deserialize;

use super::money::TransportMoney;
use crate::domain::{
    BalanceResponse, FreeUsageResponse, LimitUsageResponse, SendersResponse, Status, StatusCode,
    StatusOnlyResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("invalid JSON response: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum TransportStatus {
    Ok,
    Error,
}

impl From<TransportStatus> for Status {
    fn from(value: TransportStatus) -> Self {
        match value {
            TransportStatus::Ok => Status::Ok,
            TransportStatus::Error => Status::Error,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct StatusOnlyJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BalanceJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    balance: Option<TransportMoney>,
}

#[derive(Debug, Clone, Deserialize)]
struct FreeUsageJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    total_free: Option<TransportCount>,
    #[serde(default)]
    used_today: Option<TransportCount>,
}

#[derive(Debug, Clone, Deserialize)]
struct LimitUsageJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    total_limit: Option<TransportCount>,
    #[serde(default)]
    used_today: Option<TransportCount>,
}

#[derive(Debug, Clone, Deserialize)]
struct SendersJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    senders: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum TransportCount {
    Int(u32),
    String(String),
}

impl TransportCount {
    fn into_u32(self) -> Option<u32> {
        match self {
            Self::Int(value) => Some(value),
            Self::String(value) => value.trim().parse::<u32>().ok(),
        }
    }
}

fn encode_json_only_form() -> Vec<(String, String)> {
    vec![("json".to_owned(), "1".to_owned())]
}

pub fn encode_auth_check_form() -> Vec<(String, String)> {
    encode_json_only_form()
}

pub fn encode_get_balance_form() -> Vec<(String, String)> {
    encode_json_only_form()
}

pub fn encode_get_free_usage_form() -> Vec<(String, String)> {
    encode_json_only_form()
}

pub fn encode_get_limit_usage_form() -> Vec<(String, String)> {
    encode_json_only_form()
}

pub fn encode_get_senders_form() -> Vec<(String, String)> {
    encode_json_only_form()
}

pub fn decode_status_only_json_response(json: &str) -> Result<StatusOnlyResponse, TransportError> {
    let parsed: StatusOnlyJsonResponse = serde_json::from_str(json)?;
    Ok(StatusOnlyResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
    })
}

pub fn decode_balance_json_response(json: &str) -> Result<BalanceResponse, TransportError> {
    let parsed: BalanceJsonResponse = serde_json::from_str(json)?;
    Ok(BalanceResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        balance: parsed.balance.map(TransportMoney::into_string),
    })
}

pub fn decode_free_usage_json_response(json: &str) -> Result<FreeUsageResponse, TransportError> {
    let parsed: FreeUsageJsonResponse = serde_json::from_str(json)?;
    Ok(FreeUsageResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        total_free: parsed.total_free.and_then(TransportCount::into_u32),
        used_today: parsed.used_today.and_then(TransportCount::into_u32),
    })
}

pub fn decode_limit_usage_json_response(json: &str) -> Result<LimitUsageResponse, TransportError> {
    let parsed: LimitUsageJsonResponse = serde_json::from_str(json)?;
    Ok(LimitUsageResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        total_limit: parsed.total_limit.and_then(TransportCount::into_u32),
        used_today: parsed.used_today.and_then(TransportCount::into_u32),
    })
}

pub fn decode_senders_json_response(json: &str) -> Result<SendersResponse, TransportError> {
    let parsed: SendersJsonResponse = serde_json::from_str(json)?;
    Ok(SendersResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        senders: parsed.senders,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_json_form(params: &[(String, String)]) {
        assert_eq!(params, &[("json".to_owned(), "1".to_owned())]);
    }

    #[test]
    fn json_only_encoders_set_json_param() {
        assert_json_form(&encode_auth_check_form());
        assert_json_form(&encode_get_balance_form());
        assert_json_form(&encode_get_free_usage_form());
        assert_json_form(&encode_get_limit_usage_form());
        assert_json_form(&encode_get_senders_form());
    }

    #[test]
    fn decode_status_only_maps_payload() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100
        }
        "#;

        let parsed = decode_status_only_json_response(json).unwrap();
        assert_eq!(parsed.status, Status::Ok);
        assert_eq!(parsed.status_code, StatusCode::new(100));
        assert_eq!(parsed.status_text, None);
    }

    #[test]
    fn decode_balance_supports_numeric_and_string_money() {
        let numeric = r#"
        {
          "status": "OK",
          "status_code": 100,
          "balance": 10.50
        }
        "#;
        let parsed = decode_balance_json_response(numeric).unwrap();
        assert_eq!(parsed.balance.as_deref(), Some("10.50"));

        let string = r#"
        {
          "status": "OK",
          "status_code": 100,
          "balance": "10.50"
        }
        "#;
        let parsed = decode_balance_json_response(string).unwrap();
        assert_eq!(parsed.balance.as_deref(), Some("10.50"));
    }

    #[test]
    fn decode_free_usage_supports_numeric_or_string_counts() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "total_free": "5",
          "used_today": 3
        }
        "#;

        let parsed = decode_free_usage_json_response(json).unwrap();
        assert_eq!(parsed.total_free, Some(5));
        assert_eq!(parsed.used_today, Some(3));
    }

    #[test]
    fn decode_limit_usage_supports_numeric_or_string_counts() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "total_limit": 100,
          "used_today": "7"
        }
        "#;

        let parsed = decode_limit_usage_json_response(json).unwrap();
        assert_eq!(parsed.total_limit, Some(100));
        assert_eq!(parsed.used_today, Some(7));
    }

    #[test]
    fn decode_senders_uses_empty_default_for_missing_field() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100
        }
        "#;

        let parsed = decode_senders_json_response(json).unwrap();
        assert_eq!(parsed.senders, Vec::<String>::new());
    }
}
