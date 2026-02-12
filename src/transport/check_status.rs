use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

use super::money::TransportMoney;
use crate::domain::{CheckStatus, CheckStatusResponse, SmsId, SmsStatusResult, Status, StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("invalid JSON response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("response contains unknown sms id key: {key}")]
    UnknownSmsIdKey { key: String },
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
struct CheckStatusJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    balance: Option<TransportMoney>,
    #[serde(default)]
    sms: BTreeMap<String, SmsStatusJsonResult>,
}

#[derive(Debug, Clone, Deserialize)]
struct SmsStatusJsonResult {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    cost: Option<TransportMoney>,
}

pub fn encode_check_status_form(request: &CheckStatus) -> Vec<(String, String)> {
    vec![
        ("json".to_owned(), "1".to_owned()),
        (
            SmsId::FIELD.to_owned(),
            request
                .sms_ids()
                .iter()
                .map(SmsId::as_str)
                .collect::<Vec<_>>()
                .join(","),
        ),
    ]
}

pub fn decode_check_status_json_response(
    request: &CheckStatus,
    json: &str,
) -> Result<CheckStatusResponse, TransportError> {
    let parsed: CheckStatusJsonResponse = serde_json::from_str(json)?;
    let sms_id_lookup = sms_id_lookup_from_request(request);

    let sms = parsed
        .sms
        .into_iter()
        .map(|(key, value)| {
            let sms_id = match_sms_id_key(&sms_id_lookup, &key)?;
            Ok((
                sms_id,
                SmsStatusResult {
                    status: value.status.into(),
                    status_code: StatusCode::new(value.status_code),
                    status_text: value.status_text,
                    cost: value.cost.map(TransportMoney::into_string),
                },
            ))
        })
        .collect::<Result<BTreeMap<SmsId, SmsStatusResult>, TransportError>>()?;

    Ok(CheckStatusResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        balance: parsed.balance.map(TransportMoney::into_string),
        sms,
    })
}

fn sms_id_lookup_from_request(request: &CheckStatus) -> HashMap<String, SmsId> {
    request
        .sms_ids()
        .iter()
        .map(|sms_id| (sms_id.as_str().to_owned(), sms_id.clone()))
        .collect()
}

fn match_sms_id_key(
    sms_id_lookup: &HashMap<String, SmsId>,
    key: &str,
) -> Result<SmsId, TransportError> {
    let trimmed = key.trim();
    if let Some(found) = sms_id_lookup.get(trimmed) {
        return Ok(found.clone());
    }
    if let Some(found) = sms_id_lookup.get(key) {
        return Ok(found.clone());
    }
    Err(TransportError::UnknownSmsIdKey {
        key: key.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use crate::domain::CheckStatus;

    use super::*;

    #[test]
    fn encode_check_status_form_params() {
        let request = CheckStatus::new(vec![
            SmsId::new("000000-000001").unwrap(),
            SmsId::new("000000-000002").unwrap(),
        ])
        .unwrap();

        let params = encode_check_status_form(&request);
        assert_eq!(
            params,
            vec![
                ("json".to_owned(), "1".to_owned()),
                (
                    "sms_id".to_owned(),
                    "000000-000001,000000-000002".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn decode_json_response_maps_ids_and_numeric_money() {
        let id = SmsId::new("000000-000001").unwrap();
        let request = CheckStatus::one(id.clone());

        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "balance": 4122.56,
          "sms": {
            "000000-000001": {
              "status": "OK",
              "status_code": 103,
              "cost": 0.50,
              "status_text": "Delivered"
            }
          }
        }
        "#;

        let response = decode_check_status_json_response(&request, json).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.status_code, StatusCode::new(100));
        assert_eq!(response.balance.as_deref(), Some("4122.56"));

        let result = response.sms.get(&id).unwrap();
        assert_eq!(result.status, Status::Ok);
        assert_eq!(result.status_code, StatusCode::new(103));
        assert_eq!(result.cost.as_deref(), Some("0.50"));
    }

    #[test]
    fn decode_json_response_supports_trimmed_keys_and_string_cost() {
        let id = SmsId::new("000000-000001").unwrap();
        let request = CheckStatus::one(id.clone());

        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "sms": {
            " 000000-000001 ": {
              "status": "OK",
              "status_code": 104,
              "cost": "0.50"
            }
          }
        }
        "#;

        let response = decode_check_status_json_response(&request, json).unwrap();
        let result = response.sms.get(&id).unwrap();
        assert_eq!(result.status_code, StatusCode::new(104));
        assert_eq!(result.cost.as_deref(), Some("0.50"));
    }

    #[test]
    fn decode_json_response_keeps_per_id_errors_inside_payload() {
        let ok_id = SmsId::new("000000-000001").unwrap();
        let err_id = SmsId::new("000000-000002").unwrap();
        let request = CheckStatus::new(vec![ok_id.clone(), err_id.clone()]).unwrap();

        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "sms": {
            "000000-000001": {
              "status": "OK",
              "status_code": 103
            },
            "000000-000002": {
              "status": "ERROR",
              "status_code": -1,
              "status_text": "Message not found"
            }
          }
        }
        "#;

        let response = decode_check_status_json_response(&request, json).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.sms.get(&ok_id).unwrap().status, Status::Ok);
        assert_eq!(response.sms.get(&err_id).unwrap().status, Status::Error);
        assert_eq!(
            response.sms.get(&err_id).unwrap().status_code,
            StatusCode::new(-1)
        );
    }

    #[test]
    fn decode_json_response_parses_top_level_error_payload() {
        let id = SmsId::new("000000-000001").unwrap();
        let request = CheckStatus::one(id);

        let json = r#"
        {
          "status": "ERROR",
          "status_code": 200,
          "status_text": "Invalid api_id"
        }
        "#;

        let response = decode_check_status_json_response(&request, json).unwrap();
        assert_eq!(response.status, Status::Error);
        assert_eq!(response.status_code, StatusCode::new(200));
        assert_eq!(response.status_text.as_deref(), Some("Invalid api_id"));
        assert!(response.sms.is_empty());
    }

    #[test]
    fn decode_json_response_errors_on_unknown_sms_id_key() {
        let request = CheckStatus::one(SmsId::new("000000-000001").unwrap());
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "sms": {
            "000000-000999": {
              "status": "OK",
              "status_code": 103
            }
          }
        }
        "#;

        let err = decode_check_status_json_response(&request, json).unwrap_err();
        match err {
            TransportError::UnknownSmsIdKey { key } => assert_eq!(key, "000000-000999"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
