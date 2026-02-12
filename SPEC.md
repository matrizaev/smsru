# smsru crate – Specification

This document specifies the intended public API and behavior of the `smsru` Rust library: a typed client for selected SMS.RU HTTP API methods (`sms/send`, `sms/cost`, and `sms/status`).

## Goals

- Provide a safe, typed Rust interface for sending SMS via `https://sms.ru/sms/send`.
- Provide a safe, typed Rust interface for checking message cost via `https://sms.ru/sms/cost`.
- Provide a safe, typed Rust interface for checking message delivery status via `https://sms.ru/sms/status`.
- Default to JSON responses (`json=1`) and parse them into structured types.
- Surface SMS.RU status codes and errors in a predictable way.
- Keep request construction explicit (no hidden defaults besides `json=1`).

## Non-goals (initial scope)

- Implement all SMS.RU API methods (only `sms/send`, `sms/cost`, and `sms/status` are in scope).
- Implement CAPTCHA/anti-fraud flows (the documentation recommends CAPTCHA for end-user forms; this crate only performs server-to-server API calls).

## API endpoints

- **Send SMS**: `https://sms.ru/sms/send`
- **Check message cost**: `https://sms.ru/sms/cost`
- **Check message status**: `https://sms.ru/sms/status`
- **HTTP method**: `POST` (documentation returns code `210` if `GET` is used)
- **Content-Type**: `application/x-www-form-urlencoded`
- **Encoding**: parameters are UTF-8 (`212` indicates wrong encoding)

## Authentication

The request must include one of:

1) **API key**
- Parameter: `api_id` (required)

2) **Login + password**
- Parameters: `login` (required), `password` (required)

The library must support both variants.

## Request: “send SMS”

### Required parameters

- `to` (required): destination phone number(s)
  - A single number or a comma-separated list (up to 100 numbers per request).
  - Alternate form: pass per-recipient messages via an array-like form: `to[PHONE]=TEXT`.
- `msg` (required when `to` is not `to[PHONE]=TEXT`): message text in UTF-8.

Clarification:

- When using `to=...` (a phone list), you provide a single shared `msg=...`.
- When using `to[PHONE]=TEXT` (per-recipient messages), you do not send `msg`; the per-phone values are the messages.

Notes:

- The documentation examples use numbers like `7925...` (no leading `+`).

Phone-number policy (normative):

- By default, phone numbers are treated as opaque strings and are transmitted as provided (after trimming surrounding whitespace). This is modeled with `RawPhoneNumber`.
- E.164 normalization is opt-in via `PhoneNumber::parse(default_region, input)` backed by the `phonenumber` crate. `PhoneNumber` can be converted into `RawPhoneNumber` for transport.
- E.164 normalization must not rely on an implicit region default. If parsing requires a region, it must be an explicit input to the API.
- Parse/normalization failures:
  - APIs that accept `PhoneNumber` must fail at construction/validation time.
  - APIs that accept `RawPhoneNumber` do not perform phone parsing; invalid numbers are rejected by the server via status codes (e.g., `202`, `207`).

### Optional parameters

- `json=1` (recommended): request JSON response. The library always sends this; `JsonMode::Plain` is rejected by the client.
- `from`: sender name/phone. Must be approved by an administrator; if omitted, default sender is used (per SMS.RU settings).
- `ip`: end-user IP address (not the server IP). Used for anti-fraud protections when SMS is sent as a user action (e.g., OTP). If many messages are sent with the same `ip`, SMS.RU may block sending (settings exist in the dashboard).
- `time`: delayed send time as a UNIX timestamp. Must be no more than 2 months in the future; if less than current time, send immediately.
- `ttl`: message lifetime in minutes (1..=1440). If not delivered within TTL (e.g., subscriber offline), it is deleted by the operator; cost is not refunded.
- `daytime=1`: deliver only during recipient daytime (if it’s night for the recipient, delivery is postponed until 10:00). If set, `time` is ignored.
- `translit=1`: transliterate Cyrillic to Latin.
- `test=1`: test mode; does not send and does not charge balance.
- `partner_id`: partner program identifier (if applicable).

### Library request model

The crate should expose a request type that can represent both “same text to many numbers” and “different texts per number”:

- `SendSms::to_many(Vec<RawPhoneNumber>, MessageText, SendOptions)`
- `SendSms::per_recipient(BTreeMap<RawPhoneNumber, MessageText>, SendOptions)`

The client must serialize these into the correct SMS.RU parameter format.

For `PerRecipient`, the map contains at most one message per phone number (unique keys). If constructed from an iterator with duplicate phone keys, later entries overwrite earlier ones (standard map semantics).

## Request: “check cost” (`sms/cost`)

Use this method to estimate message price and segment count before sending.
Normative details in this section are based on the attached SMS.RU documentation export (`Проверить стоимость сообщений перед отправкой.pdf`, updated 03 October 2017).

### Required parameters

- `to` (required): destination phone number(s)
  - A single number or a comma-separated list (up to 100 numbers per request).
  - Alternate form: pass per-recipient messages via an array-like form: `to[PHONE]=TEXT`.
- `msg` (required when `to` is not `to[PHONE]=TEXT`): message text in UTF-8.

Clarification:

- When using `to=...` (a phone list), you provide a single shared `msg=...`.
- When using `to[PHONE]=TEXT` (per-recipient messages), you do not send `msg`; the per-phone values are the messages.

### Optional parameters

- `json=1` (recommended): request JSON response. The library always sends this; `JsonMode::Plain` is rejected by the client.
- `from`: sender name/phone. Must be approved by an administrator.
- `translit=1`: transliterate Cyrillic to Latin before cost calculation.

### Library request model

The crate should expose a dedicated request type for cost checking:

- `CheckCost::to_many(Vec<RawPhoneNumber>, MessageText, CheckCostOptions)`
- `CheckCost::per_recipient(BTreeMap<RawPhoneNumber, MessageText>, CheckCostOptions)`

Constraints:

- Recipient collection must be non-empty.
- Recipient count must be `<= 100`.

`CheckCostOptions` includes:

- `json` (JSON-only in the client),
- `from`,
- `translit`.

## Request: “check status” (`sms/status`)

SMS.RU recommends webhooks for near-real-time status delivery; this method is for explicit polling when needed.

### Required parameters

- `sms_id` (required): message id(s) returned by `sms/send`.
  - A single id or a comma-separated list.
  - Maximum 100 ids per request.

### Optional parameters

- `json=1` (recommended): request JSON response. The library always sends this and rejects non-JSON mode.

### Library request model

The crate should expose a request type for querying one or more message ids:

- `CheckStatus::new(Vec<SmsId>)`
- `CheckStatus::one(SmsId)` (convenience constructor)

Constraints:

- The list must be non-empty.
- The list length must be `<= 100`.

`SmsId` is modeled as an opaque validated newtype (non-empty after trimming). The crate does not enforce a specific SMS.RU id format beyond non-empty validation.

## Response: “send SMS”

### JSON response (`json=1`)

The API returns JSON with at least:

- `status`: `"OK"` or `"ERROR"`
- `status_code`: numeric code for the overall request
- `status_text`: optional textual description (observed in sample comments)
- `sms`: object keyed by recipient phone number, each value containing:
  - `status`: `"OK"` or `"ERROR"`
  - `status_code`: numeric code for that recipient
  - `status_text`: optional textual description
  - `sms_id`: present on success (message identifier)
- `balance`: account balance after sending (treat as a string for exact preservation; format is controlled by the API)

Schema/compatibility requirements:

- Treat `status_text` as optional: it may be absent.
- Optional fields must use `#[serde(default)]` (or equivalent) to avoid hard failures when fields are missing.
- Response parsing must allow unknown fields (do not use `#[serde(deny_unknown_fields)]`), to remain forward-compatible if SMS.RU adds fields later.

Money/balance representation:

- `balance` must be modeled as `Option<String>` in the public API to preserve the exact value and avoid floating-point issues.
- Convenience parsing (e.g., into `rust_decimal::Decimal`) may be provided behind an optional feature later.

Library behavior:

- If top-level `status != "OK"`, treat this as a request-level failure and return an error containing `status_code` and `status_text` (if present).
- If top-level `status == "OK"`, return a `SendSmsResponse` including per-recipient results. Per-recipient failures should not fail the whole call by default; they should be represented in the returned structure.

### Non-JSON response (legacy)

When `json` is not set, the server returns a plain-text multi-line response:

- First line: request status code (e.g., `100`)
- Next line(s): message ids or error codes per recipient (format varies)
- Last line may include `balance=...`

Library behavior:

- The library should default to JSON and does not support non-JSON parsing in the client; `SendOptions.json` must be `JsonMode::Json`.

## Response: “check cost” (`sms/cost`)

### JSON response (`json=1`)

The API returns JSON with at least:

- `status`: `"OK"` or `"ERROR"`
- `status_code`: numeric code for the overall request
- `status_text`: optional textual description
- `sms`: object keyed by recipient phone number, each value containing:
  - `status`: `"OK"` or `"ERROR"` for this recipient
  - `status_code`: numeric status code
  - `status_text`: optional textual description
  - `cost`: optional per-recipient cost
  - `sms`: optional per-recipient SMS segment count
- `total_cost`: optional total cost across recipients
- `total_sms`: optional total SMS segment count across recipients

Schema/compatibility requirements:

- Treat `status_text`, `cost`, `sms`, `total_cost`, and `total_sms` as optional.
- Optional fields must use `#[serde(default)]` (or equivalent) to avoid hard failures when fields are missing.
- Response parsing must allow unknown fields (do not use `#[serde(deny_unknown_fields)]`), to remain forward-compatible if SMS.RU adds fields later.

Money/quantity representation:

- `cost` and `total_cost` must be modeled as `Option<String>` in the public API to avoid floating-point precision loss (transport may receive JSON number or string).
- `sms` and `total_sms` should be modeled as integer counts (`Option<u32>`).

Library behavior:

- If top-level `status != "OK"`, return `SmsRuError::Api { status_code, status_text }`.
- If top-level `status == "OK"`, return `CheckCostResponse` with per-recipient results.
- Per-recipient errors (for example `207`) must be represented in the returned structure and must not fail the whole call when top-level status is OK.

### Non-JSON response (legacy)

When `json` is not set, the cost method returns plain text where:

- First line: request status code (e.g., `100`)
- Second line: total cost
- Third line: total SMS segment count

Library behavior:

- The library should default to JSON and does not support non-JSON parsing in the client; cost-check requests are JSON-only.

## Response: “check status” (`sms/status`)

### JSON response (`json=1`)

The API returns JSON with at least:

- `status`: `"OK"` or `"ERROR"`
- `status_code`: numeric code for the overall request
- `status_text`: optional textual description
- `sms`: object keyed by `sms_id`, each value containing:
  - `status`: `"OK"` or `"ERROR"` for this id lookup
  - `status_code`: numeric status code for the message
  - `status_text`: optional textual description
  - `cost`: optional message cost (present in documented successful examples)
- `balance`: account balance after the request

Schema/compatibility requirements:

- Treat `status_text`, `cost`, and `balance` as optional.
- Optional fields must use `#[serde(default)]` (or equivalent) to avoid hard failures when fields are missing.
- Response parsing must allow unknown fields (do not use `#[serde(deny_unknown_fields)]`), to remain forward-compatible if SMS.RU adds fields later.

Money representation:

- `balance` must be modeled as `Option<String>` to preserve exact formatting.
- `cost` must be modeled as `Option<String>` in the public API to avoid floating-point precision loss (transport may receive JSON number or string).

Library behavior:

- If top-level `status != "OK"`, return `SmsRuError::Api { status_code, status_text }`.
- If top-level `status == "OK"`, return `CheckStatusResponse` with per-id results.
- Per-id errors (for example `-1` "message not found") must be represented in the returned structure and must not fail the whole call when top-level status is OK.

### Non-JSON response (legacy)

When `json` is not set, the status method returns plain text where:

- First line: request status code (e.g., `100`)
- Next lines: one status code per requested `sms_id`

Library behavior:

- The library should default to JSON and does not support non-JSON parsing in the client; status-check requests are JSON-only.

## Status codes

The API uses numeric codes for request-level results and per-message delivery/error states (for `sms/send`, `sms/cost`, and `sms/status`). The crate should:

- Provide a `StatusCode` newtype wrapping an integer (recommend `i32`).
- Preserve unknown numeric codes (forward compatibility).
- Expose a non-exhaustive `KnownStatusCode` enum for the known codes listed in this spec.
- Provide helpers on `StatusCode`, at minimum:
  - `fn known_kind(&self) -> Option<KnownStatusCode>`
  - `fn is_retryable(&self) -> bool` (optional convenience)
- Equality and comparisons must be based on the underlying numeric value (`StatusCode(…)`), not on `KnownStatusCode` variants (since unknown codes must be handled).

Recommended shape:

```rust
pub struct StatusCode(pub i32);

#[non_exhaustive]
pub enum KnownStatusCode {
    // ... known variants for codes in “Status codes”
}
```

### Delivery / message state codes

- `-1`: Message not found
- `100`: Request succeeded or message is queued
- `101`: Message is being handed to operator
- `102`: Message sent (in transit)
- `103`: Delivered
- `104`: Not delivered: TTL expired
- `105`: Not delivered: deleted by operator
- `106`: Not delivered: phone failure
- `107`: Not delivered: unknown reason
- `108`: Not delivered: rejected
- `110`: Read (Viber; temporarily not working per doc)
- `150`: Not delivered: no route to this number

### Request / validation / limits codes

- `200`: Invalid `api_id`
- `201`: Insufficient funds
- `202`: Invalid recipient phone number or no route
- `203`: Empty message text
- `204`: Operator not enabled for this sender (or fallback/default sender). Create/enable in “Отправители”.
- `205`: Message too long (more than 8 SMS parts)
- `206`: Daily sending limit exceeded (or would be exceeded)
- `207`: No delivery route for this number
- `208`: Invalid `time`
- `209`: Recipient in your stop-list
- `210`: Used `GET` instead of required `POST`
- `211`: Method not found
- `212`: Message must be UTF-8
- `213`: More than 5000 numbers provided
- `214`: Recipient abroad (enabled “send only to РФ numbers” setting)
- `215`: Recipient in SMS.RU global stop-list (spam complaint)
- `216`: Forbidden word in text
- `217`: Credit advertisement without required disclaimer phrase
- `220`: Service temporarily unavailable; retry later
- `221`: Need a letter sender matching your site/company/trademark
- `230`: Exceeded total daily limit to this number
- `231`: Exceeded limit of identical messages to this number per minute
- `232`: Exceeded limit of identical messages to this number per day
- `233`: Exceeded repeat-send limit for code messages to this number (anti-fraud); can be disabled in “Настройки”
- `300`: Invalid token (expired and/or IP changed)
- `301`: Invalid `api_id` or `login/password`
- `302`: Authorized but account not confirmed
- `303`: Confirmation code is wrong
- `304`: Too many confirmation codes sent; retry later
- `305`: Too many wrong attempts; retry later
- `500`: Server error; retry
- `501`: Limit exceeded: user IP country mismatch (category 1; configurable)
- `502`: Limit exceeded: user IP country mismatch (category 2; configurable)
- `503`: Limit exceeded: too many messages to this country in short time (configurable)
- `504`: Limit exceeded: too many foreign authorizations in short time (configurable)
- `505`: Limit exceeded: too many messages per one IP (configurable)
- `506`: Limit exceeded: too many messages from hosting-provider IP ranges (%s over last 10 minutes)
- `507`: Invalid end-user IP or private network IP (192.*, 10.*, etc). Can be whitelisted.
- `508`: Limit exceeded: too many calls in 5 minutes (requests=%s, limit=%s)
- `550`: Country blocked for security reasons
- `901`: Callback URL invalid (must start with `http://`)
- `902`: Callback handler not found (may have been deleted)

## Rust public API (proposed)

This section describes the intended shape of the library API. Exact naming may evolve, but compatibility should be preserved once released.

### Client

- `SmsRuClient::new(auth)` constructs a client:
  - `auth`: `Auth::ApiId(ApiId)` or `Auth::LoginPassword { login: Login, password: Password }`
- `SmsRuClient::send_sms(request) -> Result<SendSmsResponse, SmsRuError>`
- `SmsRuClient::check_cost(request) -> Result<CheckCostResponse, SmsRuError>`
- `SmsRuClient::check_status(request) -> Result<CheckStatusResponse, SmsRuError>`
- `SmsRuClient::builder(auth)` provides endpoint/timeout/user-agent customization without exposing `reqwest`.
  - `SmsRuClientBuilder::endpoint(url)` sets all method endpoints.
  - `SmsRuClientBuilder::send_endpoint(url)` sets the `sms/send` endpoint only.
  - `SmsRuClientBuilder::cost_endpoint(url)` sets the `sms/cost` endpoint only.
  - `SmsRuClientBuilder::status_endpoint(url)` sets the `sms/status` endpoint only.

HTTP backend policy:

- The initial implementation commits to `reqwest` internally.
- Public APIs must not expose `reqwest::Client` (or other backend-specific types) in function signatures.
- If HTTP customization is needed, provide a crate-owned builder/config type (e.g., `SmsRuClientBuilder`) rather than taking a raw HTTP client.
- Optional `blocking` support may be provided behind a Cargo feature.

### Types

- `SmsId`:
  - opaque validated id of an SMS message (non-empty after trimming)
- `SendSms` request model:
  - recipients: `ToMany` or `PerRecipient`
  - options: `json`, `from`, `ip`, `time`, `ttl`, `daytime`, `translit`, `test`, `partner_id`
- `CheckCost` request model:
  - recipients: `ToMany` or `PerRecipient`
  - options: `json`, `from`, `translit`
- `CheckStatus` request model:
  - `sms_ids: Vec<SmsId>`
  - JSON-only request mode
- `SendSmsResponse`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `balance`: `Option<String>`
  - `sms`: `BTreeMap<RawPhoneNumber, SmsResult>`
- `CheckStatusResponse`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `balance`: `Option<String>`
  - `sms`: `BTreeMap<SmsId, SmsStatusResult>`
- `CheckCostResponse`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `total_cost`: `Option<String>`
  - `total_sms`: `Option<u32>`
  - `sms`: `BTreeMap<RawPhoneNumber, SmsCostResult>`
- `SmsResult`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `sms_id`: `Option<SmsId>`
- `SmsCostResult`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `cost`: `Option<String>`
  - `sms`: `Option<u32>`
- `SmsStatusResult`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `cost`: `Option<String>`

`StatusCode` is a forward-compatible newtype; known codes are available via `StatusCode::known_kind() -> Option<KnownStatusCode>`.

### Errors

- `SmsRuError::Transport` (network/HTTP layer)
- `SmsRuError::HttpStatus { status: u16, body: Option<String> }` (non-2xx HTTP response; body captured for debugging when possible)
- `SmsRuError::Parse` (invalid JSON or unexpected format)
- `SmsRuError::Api { status_code, status_text }` (top-level API error)
- `SmsRuError::UnsupportedResponseFormat` (non-JSON response requested for methods that this crate supports only in JSON mode)
- `SmsRuError::Validation` (failed construction / invalid inputs)

Per-recipient (`sms/send` and `sms/cost`) and per-id (`sms/status`) errors should not automatically map to `SmsRuError::Api` when the top-level response is OK.

## Implementation notes (non-normative)

- JSON parsing: `serde` + `serde_json`.
- Transport: `reqwest` is the expected default backend; the crate may offer `blocking` support via a Cargo feature.

## Examples (intended usage)

Note: examples use `+7...` for readability; SMS.RU also accepts numbers without a leading `+` (as shown in the upstream documentation).

### Send one text to multiple recipients (JSON)

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
let resp = client.send_sms(request).await?;

for (phone, result) in resp.sms {
    println!("{phone}: {:?} {:?}", result.status, result.status_code);
}
# Ok(())
# }
```

### Send different texts per recipient

```rust,no_run
use std::collections::BTreeMap;
use smsru::{Auth, MessageText, RawPhoneNumber, SendOptions, SendSms, SmsRuClient};

# async fn run() -> Result<(), smsru::SmsRuError> {
let mut messages = BTreeMap::new();
messages.insert(
    RawPhoneNumber::new("+79255070602")?,
    MessageText::new("Привет 1")?,
);
messages.insert(
    RawPhoneNumber::new("+74993221627")?,
    MessageText::new("Привет 2")?,
);

let client = SmsRuClient::new(Auth::api_id("...")?);
let request = SendSms::per_recipient(messages, SendOptions::default())?;
let resp = client.send_sms(request).await?;
# let _ = resp;
# Ok(())
# }
```

### Check cost before sending

```rust,no_run
use smsru::{Auth, CheckCost, CheckCostOptions, MessageText, RawPhoneNumber, SmsRuClient};

# async fn run() -> Result<(), smsru::SmsRuError> {
let client = SmsRuClient::new(Auth::api_id("...")?);
let recipients = vec![
    RawPhoneNumber::new("+79255070602")?,
    RawPhoneNumber::new("+74993221627")?,
];
let msg = MessageText::new("hello world")?;
let request = CheckCost::to_many(recipients, msg, CheckCostOptions::default())?;
let resp = client.check_cost(request).await?;

println!(
    "status={:?} code={:?} total_cost={:?} total_sms={:?}",
    resp.status, resp.status_code, resp.total_cost, resp.total_sms
);
# Ok(())
# }
```

### Check status for sent messages

```rust,no_run
use smsru::{Auth, CheckStatus, SmsId, SmsRuClient};

# async fn run() -> Result<(), smsru::SmsRuError> {
let client = SmsRuClient::new(Auth::api_id("...")?);
let ids = vec![
    SmsId::new("000000-000001")?,
    SmsId::new("000000-000002")?,
];
let request = CheckStatus::new(ids)?;
let resp = client.check_status(request).await?;

for (sms_id, result) in resp.sms {
    println!(
        "{sms_id:?}: {:?} {:?} {:?}",
        result.status, result.status_code, result.cost
    );
}
# Ok(())
# }
```
