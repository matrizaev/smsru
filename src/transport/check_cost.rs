use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

use super::money::TransportMoney;
use crate::domain::{
    CheckCost, CheckCostOptions, CheckCostResponse, JsonMode, MessageText, RawPhoneNumber,
    SenderId, SmsCostResult, Status, StatusCode,
};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("invalid JSON response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("response contains unknown phone number key: {key}")]
    UnknownPhoneNumberKey { key: String },
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
struct CheckCostJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    total_cost: Option<TransportMoney>,
    #[serde(default)]
    total_sms: Option<u32>,
    #[serde(default)]
    sms: BTreeMap<String, SmsCostJsonResult>,
}

#[derive(Debug, Clone, Deserialize)]
struct SmsCostJsonResult {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    cost: Option<TransportMoney>,
    #[serde(default)]
    sms: Option<u32>,
}

pub fn encode_check_cost_form(request: &CheckCost) -> Vec<(String, String)> {
    let mut params = Vec::<(String, String)>::new();

    match request {
        CheckCost::ToMany(to_many) => {
            push_json(&mut params, to_many.options());
            let to = to_many
                .recipients()
                .iter()
                .map(RawPhoneNumber::raw)
                .collect::<Vec<_>>()
                .join(",");
            params.push((RawPhoneNumber::FIELD.to_owned(), to));
            params.push((
                MessageText::FIELD.to_owned(),
                to_many.msg().as_str().to_owned(),
            ));
            push_options(&mut params, to_many.options());
        }
        CheckCost::PerRecipient(per_recipient) => {
            push_json(&mut params, per_recipient.options());
            for (phone, text) in per_recipient.messages() {
                let key = format!("{}[{}]", RawPhoneNumber::FIELD, phone.raw());
                params.push((key, text.as_str().to_owned()));
            }
            push_options(&mut params, per_recipient.options());
        }
    }

    params
}

fn push_json(params: &mut Vec<(String, String)>, options: &CheckCostOptions) {
    if options.json == JsonMode::Json {
        params.push(("json".to_owned(), "1".to_owned()));
    }
}

fn push_options(params: &mut Vec<(String, String)>, options: &CheckCostOptions) {
    if let Some(from) = options.from.as_ref() {
        params.push((SenderId::FIELD.to_owned(), from.as_str().to_owned()));
    }
    if options.translit {
        params.push(("translit".to_owned(), "1".to_owned()));
    }
}

pub fn decode_check_cost_json_response(
    request: &CheckCost,
    json: &str,
) -> Result<CheckCostResponse, TransportError> {
    let parsed: CheckCostJsonResponse = serde_json::from_str(json)?;
    let phone_lookup = phone_lookup_from_request(request);

    let sms = parsed
        .sms
        .into_iter()
        .map(|(key, value)| {
            let phone = match_phone_key(&phone_lookup, &key)?;
            Ok((
                phone,
                SmsCostResult {
                    status: value.status.into(),
                    status_code: StatusCode::new(value.status_code),
                    status_text: value.status_text,
                    cost: value.cost.map(TransportMoney::into_string),
                    sms: value.sms,
                },
            ))
        })
        .collect::<Result<BTreeMap<RawPhoneNumber, SmsCostResult>, TransportError>>()?;

    Ok(CheckCostResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        total_cost: parsed.total_cost.map(TransportMoney::into_string),
        total_sms: parsed.total_sms,
        sms,
    })
}

fn match_phone_key(
    phone_lookup: &HashMap<String, RawPhoneNumber>,
    key: &str,
) -> Result<RawPhoneNumber, TransportError> {
    let trimmed = key.trim();
    if let Some(found) = phone_lookup.get(trimmed) {
        return Ok(found.clone());
    }

    if let Some(found) = phone_lookup.get(key) {
        return Ok(found.clone());
    }

    Err(TransportError::UnknownPhoneNumberKey {
        key: key.to_owned(),
    })
}

fn phone_lookup_from_request(request: &CheckCost) -> HashMap<String, RawPhoneNumber> {
    let mut lookup = HashMap::<String, RawPhoneNumber>::new();
    let phones: Vec<RawPhoneNumber> = match request {
        CheckCost::ToMany(to_many) => to_many.recipients().to_vec(),
        CheckCost::PerRecipient(per_recipient) => {
            per_recipient.messages().keys().cloned().collect()
        }
    };

    for phone in phones {
        insert_phone_keys(&mut lookup, &phone);
    }
    lookup
}

fn insert_phone_keys(lookup: &mut HashMap<String, RawPhoneNumber>, phone: &RawPhoneNumber) {
    let raw = phone.raw().to_owned();
    lookup.entry(raw.clone()).or_insert_with(|| phone.clone());

    if let Some(without_plus) = raw.strip_prefix('+') {
        lookup
            .entry(without_plus.to_owned())
            .or_insert_with(|| phone.clone());
    } else {
        lookup
            .entry(format!("+{raw}"))
            .or_insert_with(|| phone.clone());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::domain::{
        CheckCost, CheckCostOptions, JsonMode, MessageText, RawPhoneNumber, SenderId,
    };

    use super::*;

    #[test]
    fn encode_to_many_form_params() {
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let p2 = RawPhoneNumber::new("+74993221627").unwrap();
        let msg = MessageText::new("hello").unwrap();

        let options = CheckCostOptions {
            from: Some(SenderId::new("MySender").unwrap()),
            translit: true,
            ..Default::default()
        };

        let req = CheckCost::to_many(vec![p1, p2], msg, options).unwrap();
        let params = encode_check_cost_form(&req);

        assert_eq!(
            params,
            vec![
                ("json".to_owned(), "1".to_owned()),
                ("to".to_owned(), "+79251234567,+74993221627".to_owned()),
                ("msg".to_owned(), "hello".to_owned()),
                ("from".to_owned(), "MySender".to_owned()),
                ("translit".to_owned(), "1".to_owned()),
            ]
        );
    }

    #[test]
    fn encode_per_recipient_expands_to_array_like_keys() {
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let p2 = RawPhoneNumber::new("+74993221627").unwrap();

        let mut messages = BTreeMap::new();
        messages.insert(p1, MessageText::new("hi 1").unwrap());
        messages.insert(p2, MessageText::new("hi 2").unwrap());

        let req = CheckCost::per_recipient(messages, CheckCostOptions::default()).unwrap();
        let params = encode_check_cost_form(&req);

        assert_eq!(
            params,
            vec![
                ("json".to_owned(), "1".to_owned()),
                ("to[+74993221627]".to_owned(), "hi 2".to_owned()),
                ("to[+79251234567]".to_owned(), "hi 1".to_owned()),
            ]
        );
    }

    #[test]
    fn encode_can_omit_json_param_when_overridden() {
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let msg = MessageText::new("hello").unwrap();

        let options = CheckCostOptions {
            json: JsonMode::Plain,
            ..Default::default()
        };
        let req = CheckCost::to_many(vec![p1], msg, options).unwrap();
        let params = encode_check_cost_form(&req);

        assert!(!params.iter().any(|(k, _)| k == "json"));
    }

    #[test]
    fn decode_json_response_maps_phone_keys_using_request_context() {
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let msg = MessageText::new("hello").unwrap();
        let req = CheckCost::to_many(vec![p1.clone()], msg, CheckCostOptions::default()).unwrap();

        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "total_cost": 0.00,
          "total_sms": 1,
          "sms": {
            "+79251234567": {
              "status": "OK",
              "status_code": 100,
              "cost": 0.00,
              "sms": 1
            }
          }
        }
        "#;

        let resp = decode_check_cost_json_response(&req, json).unwrap();
        assert_eq!(resp.status, Status::Ok);
        assert_eq!(resp.status_code, StatusCode::new(100));
        assert_eq!(resp.total_cost.as_deref(), Some("0.00"));
        assert_eq!(resp.total_sms, Some(1));
        assert_eq!(resp.sms.len(), 1);

        let result = resp.sms.get(&p1).unwrap();
        assert_eq!(result.status, Status::Ok);
        assert_eq!(result.status_code, StatusCode::new(100));
        assert_eq!(result.cost.as_deref(), Some("0.00"));
        assert_eq!(result.sms, Some(1));
    }

    #[test]
    fn decode_json_response_supports_trimmed_keys_and_string_cost() {
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let msg = MessageText::new("hello").unwrap();
        let req = CheckCost::to_many(vec![p1.clone()], msg, CheckCostOptions::default()).unwrap();

        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "total_cost": "5.00",
          "sms": {
            " +79251234567 ": {
              "status": "OK",
              "status_code": 100,
              "cost": "0.50",
              "sms": 1
            }
          }
        }
        "#;

        let resp = decode_check_cost_json_response(&req, json).unwrap();
        assert_eq!(resp.total_cost.as_deref(), Some("5.00"));
        let result = resp.sms.get(&p1).unwrap();
        assert_eq!(result.cost.as_deref(), Some("0.50"));
    }

    #[test]
    fn decode_json_response_keeps_per_recipient_errors_inside_payload() {
        let ok_phone = RawPhoneNumber::new("+79251234567").unwrap();
        let err_phone = RawPhoneNumber::new("+74993221627").unwrap();
        let msg = MessageText::new("hello").unwrap();
        let req = CheckCost::to_many(
            vec![ok_phone.clone(), err_phone.clone()],
            msg,
            CheckCostOptions::default(),
        )
        .unwrap();

        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "sms": {
            "79251234567": {
              "status": "OK",
              "status_code": 100,
              "cost": 0.00,
              "sms": 1
            },
            "74993221627": {
              "status": "ERROR",
              "status_code": 207,
              "status_text": "No route"
            }
          }
        }
        "#;

        let response = decode_check_cost_json_response(&req, json).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.sms.get(&ok_phone).unwrap().status, Status::Ok);
        assert_eq!(response.sms.get(&err_phone).unwrap().status, Status::Error);
        assert_eq!(
            response.sms.get(&err_phone).unwrap().status_code,
            StatusCode::new(207)
        );
    }

    #[test]
    fn decode_json_response_parses_top_level_error_payload() {
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let msg = MessageText::new("hello").unwrap();
        let req = CheckCost::to_many(vec![p1], msg, CheckCostOptions::default()).unwrap();

        let json = r#"
        {
          "status": "ERROR",
          "status_code": 200,
          "status_text": "Invalid api_id"
        }
        "#;

        let response = decode_check_cost_json_response(&req, json).unwrap();
        assert_eq!(response.status, Status::Error);
        assert_eq!(response.status_code, StatusCode::new(200));
        assert_eq!(response.status_text.as_deref(), Some("Invalid api_id"));
        assert!(response.sms.is_empty());
    }

    #[test]
    fn decode_json_response_errors_on_unknown_phone_key() {
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let msg = MessageText::new("hello").unwrap();
        let req = CheckCost::to_many(vec![p1], msg, CheckCostOptions::default()).unwrap();

        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "sms": {
            "79999999999": {
              "status": "OK",
              "status_code": 100,
              "cost": 0.00,
              "sms": 1
            }
          }
        }
        "#;

        let err = decode_check_cost_json_response(&req, json).unwrap_err();
        match err {
            TransportError::UnknownPhoneNumberKey { key } => assert_eq!(key, "79999999999"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
