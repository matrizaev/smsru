use std::collections::BTreeMap;

use serde::Deserialize;

use crate::domain::{
    AddStoplistEntry, RawPhoneNumber, RemoveStoplistEntry, Status, StatusCode, StoplistResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("invalid JSON response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("response contains invalid stoplist phone key: {key}")]
    InvalidStoplistPhoneKey { key: String },
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
struct StoplistJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    stoplist: BTreeMap<String, String>,
}

pub fn encode_add_stoplist_form(request: &AddStoplistEntry) -> Vec<(String, String)> {
    vec![
        ("json".to_owned(), "1".to_owned()),
        (
            "stoplist_phone".to_owned(),
            request.phone().raw().to_owned(),
        ),
        (
            crate::domain::StoplistText::FIELD.to_owned(),
            request.text().as_str().to_owned(),
        ),
    ]
}

pub fn encode_remove_stoplist_form(request: &RemoveStoplistEntry) -> Vec<(String, String)> {
    vec![
        ("json".to_owned(), "1".to_owned()),
        (
            "stoplist_phone".to_owned(),
            request.phone().raw().to_owned(),
        ),
    ]
}

pub fn encode_get_stoplist_form() -> Vec<(String, String)> {
    vec![("json".to_owned(), "1".to_owned())]
}

pub fn decode_get_stoplist_json_response(json: &str) -> Result<StoplistResponse, TransportError> {
    let parsed: StoplistJsonResponse = serde_json::from_str(json)?;
    let stoplist = parsed
        .stoplist
        .into_iter()
        .map(|(phone_key, note)| {
            let phone = RawPhoneNumber::new(phone_key.clone())
                .map_err(|_| TransportError::InvalidStoplistPhoneKey { key: phone_key })?;
            Ok((phone, note))
        })
        .collect::<Result<BTreeMap<RawPhoneNumber, String>, TransportError>>()?;

    Ok(StoplistResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        stoplist,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::StoplistText;

    #[test]
    fn encode_add_stoplist_form_params() {
        let request = AddStoplistEntry::new(
            RawPhoneNumber::new("79251234567").unwrap(),
            StoplistText::new("fraud").unwrap(),
        );
        assert_eq!(
            encode_add_stoplist_form(&request),
            vec![
                ("json".to_owned(), "1".to_owned()),
                ("stoplist_phone".to_owned(), "79251234567".to_owned()),
                ("stoplist_text".to_owned(), "fraud".to_owned()),
            ]
        );
    }

    #[test]
    fn encode_remove_stoplist_form_params() {
        let request = RemoveStoplistEntry::new(RawPhoneNumber::new("79251234567").unwrap());
        assert_eq!(
            encode_remove_stoplist_form(&request),
            vec![
                ("json".to_owned(), "1".to_owned()),
                ("stoplist_phone".to_owned(), "79251234567".to_owned()),
            ]
        );
    }

    #[test]
    fn encode_get_stoplist_form_params() {
        assert_eq!(
            encode_get_stoplist_form(),
            vec![("json".to_owned(), "1".to_owned())]
        );
    }

    #[test]
    fn decode_get_stoplist_json_response_maps_payload() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "stoplist": {
            "79251234567": "fraud"
          }
        }
        "#;

        let parsed = decode_get_stoplist_json_response(json).unwrap();
        assert_eq!(parsed.status, Status::Ok);
        assert_eq!(parsed.status_code, StatusCode::new(100));
        assert_eq!(
            parsed
                .stoplist
                .get(&RawPhoneNumber::new("79251234567").unwrap())
                .map(String::as_str),
            Some("fraud")
        );
    }

    #[test]
    fn decode_get_stoplist_json_response_errors_on_invalid_phone_key() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "stoplist": {
            "   ": "fraud"
          }
        }
        "#;

        let err = decode_get_stoplist_json_response(json).unwrap_err();
        assert!(matches!(
            err,
            TransportError::InvalidStoplistPhoneKey { .. }
        ));
    }
}
