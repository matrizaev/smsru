# smsru

Typed Rust client for the SMS.RU HTTP API (`sms/send`).

This crate focuses on a small, explicit public API: strong domain types for inputs and a client that handles JSON responses.

The client is async and expects a Tokio runtime.

## Quickstart

```rust,no_run
use smsru::{Auth, MessageText, RawPhoneNumber, SendOptions, SendSms, SmsRuClient};

# async fn run() -> Result<(), smsru::SmsRuError> {
let client = SmsRuClient::new(Auth::api_id("...")?);
let recipients = vec![
    RawPhoneNumber::new("+79255070602")?,
    RawPhoneNumber::new("+74993221627")?,
];
let msg = MessageText::new("hello world")?;
let request = SendSms::to_many(recipients, msg, SendOptions::default())?;
let response = client.send_sms(request).await?;
println!("status: {:?} code: {:?}", response.status, response.status_code);
# Ok(())
# }
```

## Phone numbers

- `RawPhoneNumber` preserves input as-is after trimming whitespace.
- `PhoneNumber::parse(default_region, input)` validates and normalizes to E.164. Convert to `RawPhoneNumber` when building requests.

## Responses and status codes

Responses preserve SMS.RU status codes via `StatusCode`. Known codes are mapped to `KnownStatusCode` through `StatusCode::known_kind()`.

## JSON-only transport

The client always sends `json=1` and only supports JSON responses. `JsonMode::Plain` is rejected by the client.
