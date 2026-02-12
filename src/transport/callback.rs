use serde::Deserialize;

use crate::domain::{
    AddCallback, CallbackUrl, CallbacksResponse, RemoveCallback, Status, StatusCode,
};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("invalid JSON response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("response contains invalid callback url: {value}")]
    InvalidCallbackUrl { value: String },
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
struct CallbackJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    callback: Vec<String>,
}

pub fn encode_add_callback_form(request: &AddCallback) -> Vec<(String, String)> {
    vec![
        ("json".to_owned(), "1".to_owned()),
        (
            CallbackUrl::FIELD.to_owned(),
            request.url().as_str().to_owned(),
        ),
    ]
}

pub fn encode_remove_callback_form(request: &RemoveCallback) -> Vec<(String, String)> {
    vec![
        ("json".to_owned(), "1".to_owned()),
        (
            CallbackUrl::FIELD.to_owned(),
            request.url().as_str().to_owned(),
        ),
    ]
}

pub fn encode_get_callbacks_form() -> Vec<(String, String)> {
    vec![("json".to_owned(), "1".to_owned())]
}

pub fn decode_callbacks_json_response(json: &str) -> Result<CallbacksResponse, TransportError> {
    let parsed: CallbackJsonResponse = serde_json::from_str(json)?;
    let callback = parsed
        .callback
        .into_iter()
        .map(|url| {
            CallbackUrl::new(url.clone())
                .map_err(|_| TransportError::InvalidCallbackUrl { value: url })
        })
        .collect::<Result<Vec<CallbackUrl>, TransportError>>()?;

    Ok(CallbacksResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        callback,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_add_callback_form_params() {
        let request = AddCallback::new(CallbackUrl::new("https://example.com/callback").unwrap());
        assert_eq!(
            encode_add_callback_form(&request),
            vec![
                ("json".to_owned(), "1".to_owned()),
                ("url".to_owned(), "https://example.com/callback".to_owned()),
            ]
        );
    }

    #[test]
    fn encode_remove_callback_form_params() {
        let request =
            RemoveCallback::new(CallbackUrl::new("https://example.com/callback").unwrap());
        assert_eq!(
            encode_remove_callback_form(&request),
            vec![
                ("json".to_owned(), "1".to_owned()),
                ("url".to_owned(), "https://example.com/callback".to_owned()),
            ]
        );
    }

    #[test]
    fn encode_get_callbacks_form_params() {
        assert_eq!(
            encode_get_callbacks_form(),
            vec![("json".to_owned(), "1".to_owned())]
        );
    }

    #[test]
    fn decode_callbacks_json_response_maps_payload() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "callback": ["https://example.com/a", "http://example.com/b"]
        }
        "#;
        let parsed = decode_callbacks_json_response(json).unwrap();
        assert_eq!(parsed.status, Status::Ok);
        assert_eq!(parsed.status_code, StatusCode::new(100));
        assert_eq!(parsed.callback.len(), 2);
    }

    #[test]
    fn decode_callbacks_json_response_errors_on_invalid_url() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "callback": ["bad"]
        }
        "#;
        let err = decode_callbacks_json_response(json).unwrap_err();
        assert!(matches!(err, TransportError::InvalidCallbackUrl { .. }));
    }
}
