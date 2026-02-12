# smsru

Typed Rust client for the SMS.RU HTTP API (`sms/send`, `sms/cost`, `sms/status`).

This crate focuses on a small, explicit public API: strong domain types for inputs and a client that handles JSON responses. Transport details are internal and not exposed as public modules.

The client is async and expects a Tokio runtime.

## MSRV

Minimum supported Rust version: **1.85** (edition 2024).

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

## Authentication

- API key: `Auth::api_id("...")?`
- Login + password: `Auth::login_password("login", "password")?`

## Request shapes

- One message to many recipients: `SendSms::to_many(Vec<RawPhoneNumber>, MessageText, SendOptions)`
- Per-recipient messages: `SendSms::per_recipient(BTreeMap<RawPhoneNumber, MessageText>, SendOptions)`
- Check message cost before sending:
  - `CheckCost::to_many(Vec<RawPhoneNumber>, MessageText, CheckCostOptions)`
  - `CheckCost::per_recipient(BTreeMap<RawPhoneNumber, MessageText>, CheckCostOptions)`
- Check status by message ids: `CheckStatus::new(Vec<SmsId>)` / `CheckStatus::one(SmsId)`

```rust,no_run
use std::collections::BTreeMap;

use smsru::{CheckCost, CheckCostOptions, MessageText, RawPhoneNumber, SendOptions, SendSms};

fn build() -> Result<SendSms, smsru::ValidationError> {
let mut messages = BTreeMap::new();
messages.insert(
    RawPhoneNumber::new("+79251234567")?,
    MessageText::new("hello")?,
);
Ok(SendSms::per_recipient(messages, SendOptions::default())?)
}

fn build_cost() -> Result<CheckCost, smsru::ValidationError> {
CheckCost::to_many(
    vec![RawPhoneNumber::new("+79251234567")?],
    MessageText::new("hello")?,
    CheckCostOptions::default(),
)
}
```

```rust,no_run
use smsru::{CheckStatus, SmsId};

fn build_status() -> Result<CheckStatus, smsru::ValidationError> {
let ids = vec![
    SmsId::new("000000-000001")?,
    SmsId::new("000000-000002")?,
];
CheckStatus::new(ids)
}
```

## Phone numbers

- `RawPhoneNumber` preserves input as-is after trimming whitespace.
- `PhoneNumber::parse(default_region, input)` validates and normalizes to E.164. Convert to `RawPhoneNumber` when building requests.

## Client configuration

Use `SmsRuClient::builder(auth)` to configure `timeout`, `user_agent`, and endpoints:

- `endpoint(...)`: set `sms/send`, `sms/cost`, and `sms/status` endpoint URLs
- `send_endpoint(...)`: set only `sms/send` URL
- `cost_endpoint(...)`: set only `sms/cost` URL
- `status_endpoint(...)`: set only `sms/status` URL

## Check cost example

```rust,no_run
use smsru::{Auth, CheckCost, CheckCostOptions, MessageText, RawPhoneNumber, SmsRuClient};

# async fn run() -> Result<(), smsru::SmsRuError> {
let client = SmsRuClient::new(Auth::api_id("...")?);
let request = CheckCost::to_many(
    vec![RawPhoneNumber::new("+79251234567")?],
    MessageText::new("hello")?,
    CheckCostOptions::default(),
)?;
let response = client.check_cost(request).await?;
println!(
    "status: {:?} code: {:?} total_cost: {:?}",
    response.status, response.status_code, response.total_cost
);
# Ok(())
# }
```

## Check status example

```rust,no_run
use smsru::{Auth, CheckStatus, SmsId, SmsRuClient};

# async fn run() -> Result<(), smsru::SmsRuError> {
let client = SmsRuClient::new(Auth::api_id("...")?);
let request = CheckStatus::new(vec![
    SmsId::new("000000-000001")?,
    SmsId::new("000000-000002")?,
])?;
let response = client.check_status(request).await?;

for (sms_id, result) in response.sms {
    println!("{sms_id:?}: {:?} {:?}", result.status, result.status_code);
}
# Ok(())
# }
```

## Responses and status codes

Responses preserve SMS.RU status codes via `StatusCode`. Known codes are mapped to `KnownStatusCode` through `StatusCode::known_kind()`.

## JSON-only transport

The client always sends `json=1` and only supports JSON responses. `JsonMode::Plain` is rejected by the client.
