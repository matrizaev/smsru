# smsru

Typed Rust client for the SMS.RU HTTP API.

Supported methods:
- `sms/send`
- `sms/cost`
- `sms/status`
- `callcheck/add`
- `callcheck/status`
- `auth/check`
- `my/balance`
- `my/free`
- `my/limit`
- `my/senders`
- `stoplist/add`
- `stoplist/del`
- `stoplist/get`
- `callback/add`
- `callback/del`
- `callback/get`

The client is async and expects a Tokio runtime.

## MSRV

Minimum supported Rust version: **1.85** (edition 2024).

## Quickstart

```rust,no_run
use smsru::{Auth, MessageText, RawPhoneNumber, SendOptions, SendSms, SmsRuClient};

# async fn run() -> Result<(), smsru::SmsRuError> {
let client = SmsRuClient::new(Auth::api_id("...")?);
let request = SendSms::to_many(
    vec![RawPhoneNumber::new("+79255070602")?],
    MessageText::new("hello world")?,
    SendOptions::default(),
)?;
let response = client.send_sms(request).await?;
println!("status: {:?} code: {:?}", response.status, response.status_code);
# Ok(())
# }
```

## Authentication

- API key: `Auth::api_id("...")?`
- Login + password: `Auth::login_password("login", "password")?`

## Request shapes

- One message to many recipients:
  - `SendSms::to_many(Vec<RawPhoneNumber>, MessageText, SendOptions)`
- Per-recipient messages:
  - `SendSms::per_recipient(BTreeMap<RawPhoneNumber, MessageText>, SendOptions)`
- Check message cost:
  - `CheckCost::to_many(Vec<RawPhoneNumber>, MessageText, CheckCostOptions)`
  - `CheckCost::per_recipient(BTreeMap<RawPhoneNumber, MessageText>, CheckCostOptions)`
- Check status:
  - `CheckStatus::new(Vec<SmsId>)`
  - `CheckStatus::one(SmsId)`
- Stoplist:
  - `AddStoplistEntry::new(RawPhoneNumber, StoplistText)`
  - `RemoveStoplistEntry::new(RawPhoneNumber)`
- Callback handlers:
  - `AddCallback::new(CallbackUrl)`
  - `RemoveCallback::new(CallbackUrl)`

## Account and utility methods

No-arg client methods (auth + `json=1`):
- `check_auth()`
- `get_balance()`
- `get_free_usage()`
- `get_limit_usage()`
- `get_senders()`
- `get_stoplist()`
- `get_callbacks()`

Input-based client methods:
- `send_sms(...)`
- `check_cost(...)`
- `check_status(...)`
- `start_call_auth(...)`
- `check_call_auth_status(...)`
- `add_stoplist_entry(...)`
- `remove_stoplist_entry(...)`
- `add_callback(...)`
- `remove_callback(...)`

## Strong types

- `RawPhoneNumber`: non-empty, no normalization.
- `PhoneNumber::parse(...)`: optional E.164 normalization path.
- `StoplistText`: non-empty note for stoplist entries.
- `CallbackUrl`: absolute `http://` or `https://` URL.

## Client configuration

Use `SmsRuClient::builder(auth)` to configure `timeout`, `user_agent`, and endpoints.

Per-endpoint overrides:
- `send_endpoint(...)`
- `cost_endpoint(...)`
- `status_endpoint(...)`
- `callcheck_add_endpoint(...)`
- `callcheck_status_endpoint(...)`
- `auth_check_endpoint(...)`
- `my_balance_endpoint(...)`
- `my_free_endpoint(...)`
- `my_limit_endpoint(...)`
- `my_senders_endpoint(...)`
- `stoplist_add_endpoint(...)`
- `stoplist_del_endpoint(...)`
- `stoplist_get_endpoint(...)`
- `callback_add_endpoint(...)`
- `callback_del_endpoint(...)`
- `callback_get_endpoint(...)`

`endpoint(...)` sets all method endpoints at once.

## JSON-only transport

The client always sends `json=1` and only supports JSON responses.
When a request type exposes `JsonMode`, `JsonMode::Plain` is rejected by the client.

## Status codes

Responses preserve SMS.RU status codes via `StatusCode`.
Known codes are mapped through `StatusCode::known_kind()`; unknown codes are preserved.
