# smsru crate - Specification

This document specifies the intended public API and behavior of the `smsru` Rust library: a typed client for SMS.RU HTTP API methods.

Implemented methods:
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

## Goals

- Provide safe, typed Rust interfaces for the supported methods.
- Define decision-complete API contracts for all supported methods listed above.
- Default to JSON responses (`json=1`) and parse them into typed domain structures.
- Preserve SMS.RU status codes, including unknown values (forward compatibility).
- Keep request construction explicit with strong domain types and constructor validation.

## Non-goals

- Implement methods not covered by the supported method set above.
- Implement CAPTCHA or anti-fraud UI workflows.
- Implement webhook receiver infrastructure.

## API endpoints

All methods use `application/x-www-form-urlencoded` requests with UTF-8 values.
The client uses `POST` for all methods, even where SMS.RU docs show query-string examples.

Implemented endpoints:
- `https://sms.ru/sms/send`
- `https://sms.ru/sms/cost`
- `https://sms.ru/sms/status`
- `https://sms.ru/callcheck/add`
- `https://sms.ru/callcheck/status`
- `https://sms.ru/auth/check`
- `https://sms.ru/my/balance`
- `https://sms.ru/my/free`
- `https://sms.ru/my/limit`
- `https://sms.ru/my/senders`
- `https://sms.ru/stoplist/add`
- `https://sms.ru/stoplist/del`
- `https://sms.ru/stoplist/get`
- `https://sms.ru/callback/add`
- `https://sms.ru/callback/del`
- `https://sms.ru/callback/get`

## Authentication

Every method requires one auth variant:

1. API key:
- `api_id`

2. Login/password:
- `login`
- `password`

## Request model

## Request: `sms/send`

Required:
- `to` and `msg` for one-text-to-many
- or `to[PHONE]=TEXT` per recipient

Optional:
- `json=1` (client JSON-only)
- `from`, `ip`, `time`, `ttl`, `daytime`, `translit`, `test`, `partner_id`

Request API:
- `SendSms::to_many(Vec<RawPhoneNumber>, MessageText, SendOptions)`
- `SendSms::per_recipient(BTreeMap<RawPhoneNumber, MessageText>, SendOptions)`

## Request: `sms/cost`

Required:
- `to` and `msg` for one-text-to-many
- or `to[PHONE]=TEXT` per recipient

Optional:
- `json=1` (client JSON-only)
- `from`, `translit`

Request API:
- `CheckCost::to_many(Vec<RawPhoneNumber>, MessageText, CheckCostOptions)`
- `CheckCost::per_recipient(BTreeMap<RawPhoneNumber, MessageText>, CheckCostOptions)`

## Request: `sms/status`

Required:
- `sms_id` (single or comma-separated list, max 100)

Optional:
- `json=1` (client JSON-only)

Request API:
- `CheckStatus::new(Vec<SmsId>)`
- `CheckStatus::one(SmsId)`

## Request: `callcheck/add`

Required:
- `phone`

Optional:
- `json=1` (client JSON-only)

Request API:
- `StartCallAuth::new(RawPhoneNumber, StartCallAuthOptions)`

## Request: `callcheck/status`

Required:
- `check_id`

Optional:
- `json=1` (client JSON-only)

Request API:
- `CheckCallAuthStatus::new(CallCheckId, CheckCallAuthStatusOptions)`

## Request family: `auth/check`

Required:
- auth only (`api_id` or `login` + `password`)

Optional:
- `json=1` (client JSON-only)

Request API:
- no-arg method: `SmsRuClient::check_auth()`

## Request family: `my/*`

Methods:
- `my/balance`
- `my/free`
- `my/limit`
- `my/senders`

Required:
- auth only

Optional:
- `json=1` (client JSON-only)

Request API:
- no-arg methods:
  - `SmsRuClient::get_balance()`
  - `SmsRuClient::get_free_usage()`
  - `SmsRuClient::get_limit_usage()`
  - `SmsRuClient::get_senders()`

## Request family: `stoplist/*`

### `stoplist/add`
Required:
- `stoplist_phone`
- `stoplist_text`
- auth

Optional:
- `json=1` (client JSON-only)

### `stoplist/del`
Required:
- `stoplist_phone`
- auth

Optional:
- `json=1` (client JSON-only)

### `stoplist/get`
Required:
- auth

Optional:
- `json=1` (client JSON-only)

Request API:
- `SmsRuClient::add_stoplist_entry(AddStoplistEntry)`
- `SmsRuClient::remove_stoplist_entry(RemoveStoplistEntry)`
- `SmsRuClient::get_stoplist()`

## Request family: `callback/*`

### `callback/add`
Required:
- `url`
- auth

Optional:
- `json=1` (client JSON-only)

### `callback/del`
Required:
- `url`
- auth

Optional:
- `json=1` (client JSON-only)

### `callback/get`
Required:
- auth

Optional:
- `json=1` (client JSON-only)

Request API:
- `SmsRuClient::add_callback(AddCallback)`
- `SmsRuClient::remove_callback(RemoveCallback)`
- `SmsRuClient::get_callbacks()`

Callback URL policy:
- Use `CallbackUrl` domain type.
- Accept only absolute `http://` or `https://` URLs.

## Response model

Top-level policy for all methods:
- parse `status`, `status_code`, and optional `status_text`
- if top-level `status != OK`, client returns `SmsRuError::Api`
- unknown JSON fields must be tolerated

## Response: `sms/send`

JSON fields:
- `status`, `status_code`, `status_text`
- `balance`
- `sms` map keyed by phone with per-recipient status payload

Public response type:
- `SendSmsResponse`

## Response: `sms/cost`

JSON fields:
- `status`, `status_code`, `status_text`
- `total_cost`, `total_sms`
- `sms` map keyed by phone with per-recipient cost payload

Public response type:
- `CheckCostResponse`

## Response: `sms/status`

JSON fields:
- `status`, `status_code`, `status_text`
- `balance`
- `sms` map keyed by `sms_id`

Public response type:
- `CheckStatusResponse`

## Response: `callcheck/add`

JSON fields:
- `status`, `status_code`, `status_text`
- `check_id`, `call_phone`, `call_phone_pretty`, `call_phone_html`

Public response type:
- `StartCallAuthResponse`

## Response: `callcheck/status`

JSON fields:
- `status`, `status_code`, `status_text`
- `check_status`, `check_status_text`

Public response type:
- `CheckCallAuthStatusResponse`

## Response: `auth/check`

JSON fields:
- `status`, `status_code`, optional `status_text`

Public response type:
- `StatusOnlyResponse`

## Response: `my/balance`

JSON fields:
- `status`, `status_code`, optional `status_text`
- `balance`

Public response type:
- `BalanceResponse`

## Response: `my/free`

JSON fields:
- `status`, `status_code`, optional `status_text`
- `total_free`, `used_today`

Public response type:
- `FreeUsageResponse`

## Response: `my/limit`

JSON fields:
- `status`, `status_code`, optional `status_text`
- `total_limit`, `used_today`

Public response type:
- `LimitUsageResponse`

## Response: `my/senders`

JSON fields:
- `status`, `status_code`, optional `status_text`
- `senders` array

Public response type:
- `SendersResponse`

## Response: `stoplist/add`

JSON fields:
- `status`, `status_code`, optional `status_text`

Public response type:
- `StatusOnlyResponse`

## Response: `stoplist/del`

JSON fields:
- `status`, `status_code`, optional `status_text`

Public response type:
- `StatusOnlyResponse`

## Response: `stoplist/get`

JSON fields:
- `status`, `status_code`, optional `status_text`
- `stoplist` map: phone -> note

Public response type:
- `StoplistResponse`

## Response: `callback/add`

JSON fields:
- `status`, `status_code`, optional `status_text`
- `callback` array (current configured callback URLs)

Public response type:
- `CallbacksResponse`

## Response: `callback/del`

JSON fields:
- `status`, `status_code`, optional `status_text`
- `callback` array

Public response type:
- `CallbacksResponse`

## Response: `callback/get`

JSON fields:
- `status`, `status_code`, optional `status_text`
- `callback` array

Public response type:
- `CallbacksResponse`

## Status codes

`StatusCode` must remain a forward-compatible newtype over integer values.
Unknown codes are preserved and never discarded.

Known mappings include, at minimum:

Delivery/message states:
- `-1`, `100`, `101`, `102`, `103`, `104`, `105`, `106`, `107`, `108`, `110`, `150`

Request/validation/rate/auth/server:
- `200`, `201`, `202`, `203`, `204`, `205`, `206`, `207`, `208`, `209`, `210`, `211`, `212`, `213`, `214`, `215`, `216`, `217`, `220`, `221`, `230`, `231`, `232`, `233`, `300`, `301`, `302`, `303`, `304`, `305`, `500`, `501`, `502`, `503`, `504`, `505`, `506`, `507`, `508`, `550`

Callback-specific (must remain mapped):
- `901` (invalid callback URL)
- `902` (callback handler not found)

Call-check states in `check_status`:
- `400`, `401`, `402`

## Rust public API

## Client methods

Implemented methods:
- `SmsRuClient::send_sms(request) -> Result<SendSmsResponse, SmsRuError>`
- `SmsRuClient::check_cost(request) -> Result<CheckCostResponse, SmsRuError>`
- `SmsRuClient::check_status(request) -> Result<CheckStatusResponse, SmsRuError>`
- `SmsRuClient::start_call_auth(request) -> Result<StartCallAuthResponse, SmsRuError>`
- `SmsRuClient::check_call_auth_status(request) -> Result<CheckCallAuthStatusResponse, SmsRuError>`
- `SmsRuClient::check_auth() -> Result<StatusOnlyResponse, SmsRuError>`
- `SmsRuClient::get_balance() -> Result<BalanceResponse, SmsRuError>`
- `SmsRuClient::get_free_usage() -> Result<FreeUsageResponse, SmsRuError>`
- `SmsRuClient::get_limit_usage() -> Result<LimitUsageResponse, SmsRuError>`
- `SmsRuClient::get_senders() -> Result<SendersResponse, SmsRuError>`
- `SmsRuClient::add_stoplist_entry(AddStoplistEntry) -> Result<StatusOnlyResponse, SmsRuError>`
- `SmsRuClient::remove_stoplist_entry(RemoveStoplistEntry) -> Result<StatusOnlyResponse, SmsRuError>`
- `SmsRuClient::get_stoplist() -> Result<StoplistResponse, SmsRuError>`
- `SmsRuClient::add_callback(AddCallback) -> Result<CallbacksResponse, SmsRuError>`
- `SmsRuClient::remove_callback(RemoveCallback) -> Result<CallbacksResponse, SmsRuError>`
- `SmsRuClient::get_callbacks() -> Result<CallbacksResponse, SmsRuError>`

## Builder endpoint overrides

Implemented:
- `endpoint(...)`
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

## Types/interfaces

Existing request/response types remain unchanged.

Additional domain/public types:
- `CallbackUrl` (absolute `http://` or `https://` only)
- `StoplistText` (non-empty after trimming)
- `AddStoplistEntry`
- `RemoveStoplistEntry`
- `AddCallback`
- `RemoveCallback`
- `StatusOnlyResponse`
- `BalanceResponse`
- `FreeUsageResponse`
- `LimitUsageResponse`
- `SendersResponse`
- `StoplistResponse`
- `CallbacksResponse`

## Errors

Single crate error type remains:
- `SmsRuError::Transport`
- `SmsRuError::HttpStatus`
- `SmsRuError::Parse`
- `SmsRuError::Api`
- `SmsRuError::UnsupportedResponseFormat`
- `SmsRuError::Validation`

## Implementation notes

- `serde` is the default JSON mapping mechanism.
- `reqwest` is the default HTTP backend behind the crate-owned client abstraction.
- JSON-only mode is enforced by client methods.

## Examples (intended usage)

### Auth check + balance read

```rust,no_run
use smsru::{Auth, SmsRuClient};

# async fn run() -> Result<(), smsru::SmsRuError> {
let client = SmsRuClient::new(Auth::api_id("...")?);
let auth = client.check_auth().await?;
let balance = client.get_balance().await?;
println!("auth={:?} code={:?}", auth.status, auth.status_code);
println!("balance={:?}", balance.balance);
# Ok(())
# }
```

### Stoplist lifecycle

```rust,no_run
use smsru::{
    AddStoplistEntry, Auth, RawPhoneNumber, RemoveStoplistEntry, SmsRuClient, StoplistText,
};

# async fn run() -> Result<(), smsru::SmsRuError> {
let client = SmsRuClient::new(Auth::api_id("...")?);

client
    .add_stoplist_entry(AddStoplistEntry::new(
        RawPhoneNumber::new("79250000001")?,
        StoplistText::new("fraud")?,
    ))
    .await?;

let all = client.get_stoplist().await?;
println!("stoplist entries={}", all.stoplist.len());

client
    .remove_stoplist_entry(RemoveStoplistEntry::new(RawPhoneNumber::new("79250000001")?))
    .await?;
# Ok(())
# }
```

### Callback lifecycle

```rust,no_run
use smsru::{AddCallback, Auth, CallbackUrl, RemoveCallback, SmsRuClient};

# async fn run() -> Result<(), smsru::SmsRuError> {
let client = SmsRuClient::new(Auth::api_id("...")?);

let url = CallbackUrl::new("https://example.com/sms/callback")?;
client.add_callback(AddCallback::new(url.clone())).await?;

let callbacks = client.get_callbacks().await?;
println!("callbacks={}", callbacks.callback.len());

client.remove_callback(RemoveCallback::new(url)).await?;
# Ok(())
# }
```
