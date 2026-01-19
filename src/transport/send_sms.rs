use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

use crate::domain::{
    JsonMode, MessageText, PartnerId, RawPhoneNumber, SendOptions, SendSms, SendSmsResponse,
    SenderId, SmsResult, Status, StatusCode, TtlMinutes, UnixTimestamp,
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
struct SendSmsJsonResponse {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    balance: Option<TransportBalance>,
    #[serde(default)]
    sms: BTreeMap<String, SmsJsonResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum TransportBalance {
    String(String),
    Number(serde_json::Number),
}

impl TransportBalance {
    fn into_string(self) -> String {
        match self {
            Self::String(value) => value,
            Self::Number(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SmsJsonResult {
    status: TransportStatus,
    status_code: i32,
    #[serde(default)]
    status_text: Option<String>,
    #[serde(default)]
    sms_id: Option<String>,
}

pub fn encode_send_sms_form(request: &SendSms) -> Vec<(String, String)> {
    let mut params = Vec::<(String, String)>::new();

    match request {
        SendSms::ToMany(to_many) => {
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
        SendSms::PerRecipient(per_recipient) => {
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

fn push_json(params: &mut Vec<(String, String)>, options: &SendOptions) {
    if options.json == JsonMode::Json {
        params.push(("json".to_owned(), "1".to_owned()));
    }
}

fn push_options(params: &mut Vec<(String, String)>, options: &SendOptions) {
    if let Some(from) = options.from.as_ref() {
        params.push((SenderId::FIELD.to_owned(), from.as_str().to_owned()));
    }
    if let Some(ip) = options.ip {
        params.push(("ip".to_owned(), ip.to_string()));
    }
    if let Some(time) = options.time {
        params.push((UnixTimestamp::FIELD.to_owned(), time.value().to_string()));
    }
    if let Some(ttl) = options.ttl {
        params.push((TtlMinutes::FIELD.to_owned(), ttl.value().to_string()));
    }
    if options.daytime {
        params.push(("daytime".to_owned(), "1".to_owned()));
    }
    if options.translit {
        params.push(("translit".to_owned(), "1".to_owned()));
    }
    if options.test {
        params.push(("test".to_owned(), "1".to_owned()));
    }
    if let Some(partner_id) = options.partner_id.as_ref() {
        params.push((PartnerId::FIELD.to_owned(), partner_id.as_str().to_owned()));
    }
}

pub fn decode_send_sms_json_response(
    request: &SendSms,
    json: &str,
) -> Result<SendSmsResponse, TransportError> {
    let parsed: SendSmsJsonResponse = serde_json::from_str(json)?;
    let phone_lookup = phone_lookup_from_request(request);

    let sms = parsed
        .sms
        .into_iter()
        .map(|(key, value)| {
            let phone = match_phone_key(&phone_lookup, &key)?;
            Ok((
                phone,
                SmsResult {
                    status: value.status.into(),
                    status_code: StatusCode::new(value.status_code),
                    status_text: value.status_text,
                    sms_id: value.sms_id,
                },
            ))
        })
        .collect::<Result<BTreeMap<RawPhoneNumber, SmsResult>, TransportError>>()?;

    Ok(SendSmsResponse {
        status: parsed.status.into(),
        status_code: StatusCode::new(parsed.status_code),
        status_text: parsed.status_text,
        balance: parsed.balance.map(TransportBalance::into_string),
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

fn phone_lookup_from_request(request: &SendSms) -> HashMap<String, RawPhoneNumber> {
    let mut lookup = HashMap::<String, RawPhoneNumber>::new();
    let phones: Vec<RawPhoneNumber> = match request {
        SendSms::ToMany(to_many) => to_many.recipients().to_vec(),
        SendSms::PerRecipient(per_recipient) => per_recipient.messages().keys().cloned().collect(),
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
    use std::net::IpAddr;

    use crate::domain::{
        JsonMode, MessageText, RawPhoneNumber, SendOptions, SendSms, TtlMinutes, UnixTimestamp,
    };

    use super::*;

    #[test]
    fn encode_to_many_form_params() {
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let p2 = RawPhoneNumber::new("+74993221627").unwrap();
        let msg = MessageText::new("hello").unwrap();

        let options = SendOptions {
            ip: Some(IpAddr::from([127, 0, 0, 1])),
            time: Some(UnixTimestamp::new(1_700_000_000)),
            ttl: Some(TtlMinutes::new(60).unwrap()),
            daytime: true,
            test: true,
            ..Default::default()
        };

        let req = SendSms::to_many(vec![p1, p2], msg, options).unwrap();
        let params = encode_send_sms_form(&req);

        assert_eq!(
            params,
            vec![
                ("json".to_owned(), "1".to_owned()),
                ("to".to_owned(), "+79251234567,+74993221627".to_owned()),
                ("msg".to_owned(), "hello".to_owned()),
                ("ip".to_owned(), "127.0.0.1".to_owned()),
                ("time".to_owned(), "1700000000".to_owned()),
                ("ttl".to_owned(), "60".to_owned()),
                ("daytime".to_owned(), "1".to_owned()),
                ("test".to_owned(), "1".to_owned()),
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

        let req = SendSms::per_recipient(messages, SendOptions::default()).unwrap();
        let params = encode_send_sms_form(&req);

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

        let options = SendOptions {
            json: JsonMode::Plain,
            ..Default::default()
        };
        let req = SendSms::to_many(vec![p1], msg, options).unwrap();
        let params = encode_send_sms_form(&req);

        assert!(!params.iter().any(|(k, _)| k == "json"));
    }

    #[test]
    fn decode_json_response_maps_phone_keys_using_request_context() {
        let p1 = RawPhoneNumber::new("+79251234567").unwrap();
        let msg = MessageText::new("hello").unwrap();
        let req = SendSms::to_many(vec![p1.clone()], msg, SendOptions::default()).unwrap();

        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "balance": 12.34,
          "sms": {
            "+79251234567": {
              "status": "OK",
              "status_code": 100,
              "sms_id": "abc123"
            }
          }
        }
        "#;

        let resp = decode_send_sms_json_response(&req, json).unwrap();
        assert_eq!(resp.status, Status::Ok);
        assert_eq!(resp.status_code, StatusCode::new(100));
        assert_eq!(resp.balance.as_deref(), Some("12.34"));
        assert_eq!(resp.sms.len(), 1);

        let result = resp.sms.get(&p1).unwrap();
        assert_eq!(result.status, Status::Ok);
        assert_eq!(result.status_code, StatusCode::new(100));
        assert_eq!(result.sms_id.as_deref(), Some("abc123"));
    }
}
