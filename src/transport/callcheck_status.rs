use serde::Deserialize;

use crate::domain::{
    CallCheckStatusCode, CheckCallAuthStatus, CheckCallAuthStatusResponse, JsonMode, Status,
    StatusCode,
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
struct CheckCallAuthStatusJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    check_status: Option<TransportCheckStatusCode>,
    #[serde(default)]
    check_status_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum TransportCheckStatusCode {
    Int(i32),
    String(String),
}

impl TransportCheckStatusCode {
    fn into_code(self) -> Option<CallCheckStatusCode> {
        match self {
            Self::Int(value) => Some(CallCheckStatusCode::new(value)),
            Self::String(value) => value
                .trim()
                .parse::<i32>()
                .ok()
                .map(CallCheckStatusCode::new),
        }
    }
}

pub fn encode_check_call_auth_status_form(request: &CheckCallAuthStatus) -> Vec<(String, String)> {
    let mut params = Vec::<(String, String)>::new();

    if request.options().json == JsonMode::Json {
        params.push(("json".to_owned(), "1".to_owned()));
    }

    params.push((
        crate::domain::CallCheckId::FIELD.to_owned(),
        request.check_id().as_str().to_owned(),
    ));

    params
}

pub fn decode_check_call_auth_status_json_response(
    json: &str,
) -> Result<CheckCallAuthStatusResponse, TransportError> {
    let parsed: CheckCallAuthStatusJsonResponse = serde_json::from_str(json)?;

    Ok(CheckCallAuthStatusResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        check_status: parsed
            .check_status
            .and_then(TransportCheckStatusCode::into_code),
        check_status_text: parsed.check_status_text,
    })
}

#[cfg(test)]
mod tests {
    use crate::domain::{
        CallCheckId, CheckCallAuthStatus, CheckCallAuthStatusOptions, JsonMode,
        KnownCallCheckStatusCode,
    };

    use super::*;

    #[test]
    fn encode_check_call_auth_status_form_params() {
        let request = CheckCallAuthStatus::new(
            CallCheckId::new("201737-542").unwrap(),
            CheckCallAuthStatusOptions::default(),
        );

        let params = encode_check_call_auth_status_form(&request);
        assert_eq!(
            params,
            vec![
                ("json".to_owned(), "1".to_owned()),
                ("check_id".to_owned(), "201737-542".to_owned()),
            ]
        );
    }

    #[test]
    fn encode_can_omit_json_param_when_overridden() {
        let request = CheckCallAuthStatus::new(
            CallCheckId::new("201737-542").unwrap(),
            CheckCallAuthStatusOptions {
                json: JsonMode::Plain,
            },
        );

        let params = encode_check_call_auth_status_form(&request);
        assert_eq!(
            params,
            vec![("check_id".to_owned(), "201737-542".to_owned())]
        );
    }

    #[test]
    fn decode_json_response_maps_success_payload_with_string_code() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "check_status": "401",
          "check_status_text": "Авторизация по звонку: номер подтвержден"
        }
        "#;

        let response = decode_check_call_auth_status_json_response(json).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.status_code, StatusCode::new(100));
        assert_eq!(
            response
                .check_status
                .and_then(CallCheckStatusCode::known_kind),
            Some(KnownCallCheckStatusCode::Confirmed)
        );
    }

    #[test]
    fn decode_json_response_maps_success_payload_with_numeric_code() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "check_status": 400,
          "check_status_text": "Номер пока не подтвержден"
        }
        "#;

        let response = decode_check_call_auth_status_json_response(json).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(
            response
                .check_status
                .and_then(CallCheckStatusCode::known_kind),
            Some(KnownCallCheckStatusCode::NotConfirmedYet)
        );
    }

    #[test]
    fn decode_json_response_parses_top_level_error_payload() {
        let json = r#"
        {
          "status": "ERROR",
          "status_code": 200,
          "status_text": "Invalid api_id"
        }
        "#;

        let response = decode_check_call_auth_status_json_response(json).unwrap();
        assert_eq!(response.status, Status::Error);
        assert_eq!(response.status_code, StatusCode::new(200));
        assert_eq!(response.status_text.as_deref(), Some("Invalid api_id"));
        assert!(response.check_status.is_none());
    }
}
