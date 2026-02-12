use serde::Deserialize;

use crate::domain::{
    CallCheckId, JsonMode, RawPhoneNumber, StartCallAuth, StartCallAuthResponse, Status, StatusCode,
};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("invalid JSON response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("response contains invalid check id: {value}")]
    InvalidCheckId { value: String },

    #[error("response contains invalid call phone: {value}")]
    InvalidCallPhone { value: String },
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
struct StartCallAuthJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    check_id: Option<String>,
    #[serde(default)]
    call_phone: Option<String>,
    #[serde(default)]
    call_phone_pretty: Option<String>,
    #[serde(default)]
    call_phone_html: Option<String>,
}

pub fn encode_start_call_auth_form(request: &StartCallAuth) -> Vec<(String, String)> {
    let mut params = Vec::<(String, String)>::new();

    if request.options().json == JsonMode::Json {
        params.push(("json".to_owned(), "1".to_owned()));
    }

    params.push(("phone".to_owned(), request.phone().raw().to_owned()));

    params
}

pub fn decode_start_call_auth_json_response(
    json: &str,
) -> Result<StartCallAuthResponse, TransportError> {
    let parsed: StartCallAuthJsonResponse = serde_json::from_str(json)?;

    let check_id = parsed
        .check_id
        .map(|value| {
            CallCheckId::new(value.clone()).map_err(|_| TransportError::InvalidCheckId { value })
        })
        .transpose()?;

    let call_phone = parsed
        .call_phone
        .map(|value| {
            RawPhoneNumber::new(value.clone())
                .map_err(|_| TransportError::InvalidCallPhone { value })
        })
        .transpose()?;

    Ok(StartCallAuthResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        check_id,
        call_phone,
        call_phone_pretty: parsed.call_phone_pretty,
        call_phone_html: parsed.call_phone_html,
    })
}

#[cfg(test)]
mod tests {
    use crate::domain::{JsonMode, RawPhoneNumber, StartCallAuth, StartCallAuthOptions};

    use super::*;

    #[test]
    fn encode_start_call_auth_form_params() {
        let request = StartCallAuth::new(
            RawPhoneNumber::new("79251234567").unwrap(),
            StartCallAuthOptions::default(),
        );

        let params = encode_start_call_auth_form(&request);
        assert_eq!(
            params,
            vec![
                ("json".to_owned(), "1".to_owned()),
                ("phone".to_owned(), "79251234567".to_owned()),
            ]
        );
    }

    #[test]
    fn encode_can_omit_json_param_when_overridden() {
        let request = StartCallAuth::new(
            RawPhoneNumber::new("79251234567").unwrap(),
            StartCallAuthOptions {
                json: JsonMode::Plain,
            },
        );

        let params = encode_start_call_auth_form(&request);
        assert_eq!(params, vec![("phone".to_owned(), "79251234567".to_owned())]);
    }

    #[test]
    fn decode_json_response_maps_success_payload() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "check_id": "201737-542",
          "call_phone": "78005008275",
          "call_phone_pretty": "+7 (800) 500-8275",
          "call_phone_html": "<a href=\"callto:78005008275\">+7 (800) 500-8275</a>"
        }
        "#;

        let response = decode_start_call_auth_json_response(json).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.status_code, StatusCode::new(100));
        assert_eq!(
            response.check_id.as_ref().map(CallCheckId::as_str),
            Some("201737-542")
        );
        assert_eq!(
            response.call_phone.as_ref().map(RawPhoneNumber::raw),
            Some("78005008275")
        );
        assert_eq!(
            response.call_phone_pretty.as_deref(),
            Some("+7 (800) 500-8275")
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

        let response = decode_start_call_auth_json_response(json).unwrap();
        assert_eq!(response.status, Status::Error);
        assert_eq!(response.status_code, StatusCode::new(200));
        assert_eq!(response.status_text.as_deref(), Some("Invalid api_id"));
        assert!(response.check_id.is_none());
        assert!(response.call_phone.is_none());
    }
}
