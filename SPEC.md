# smsru crate – Specification

This document specifies the intended public API and behavior of the `smsru` Rust library: a typed client for the SMS.RU HTTP API described in `smsru.pdf` (method: “Отправить СМС сообщение HTTP запросом”).

## Goals

- Provide a safe, typed Rust interface for sending SMS via `https://sms.ru/sms/send`.
- Default to JSON responses (`json=1`) and parse them into structured types.
- Surface SMS.RU status codes and errors in a predictable way.
- Keep request construction explicit (no hidden defaults besides `json=1`).

## Non-goals (initial scope)

- Implement all SMS.RU API methods (only `sms/send` is in scope).
- Implement CAPTCHA/anti-fraud flows (the documentation recommends CAPTCHA for end-user forms; this crate only performs server-to-server API calls).

## API endpoint

- **URL**: `https://sms.ru/sms/send`
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

## Response

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

## Status codes

The API uses numeric codes both for request-level results and per-message delivery / error statuses. The crate should:

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
- `SmsRuClient::builder(auth)` provides endpoint/timeout/user-agent customization without exposing `reqwest`.

HTTP backend policy:

- The initial implementation commits to `reqwest` internally.
- Public APIs must not expose `reqwest::Client` (or other backend-specific types) in function signatures.
- If HTTP customization is needed, provide a crate-owned builder/config type (e.g., `SmsRuClientBuilder`) rather than taking a raw HTTP client.
- Optional `blocking` support may be provided behind a Cargo feature.

### Types

- `SendSms` request model:
  - recipients: `ToMany` or `PerRecipient`
  - options: `json`, `from`, `ip`, `time`, `ttl`, `daytime`, `translit`, `test`, `partner_id`
- `SendSmsResponse`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `balance`: `Option<String>`
  - `sms`: `BTreeMap<String, SmsResult>`
- `SmsResult`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `sms_id`: `Option<String>`

`StatusCode` is a forward-compatible newtype; known codes are available via `StatusCode::known_kind() -> Option<KnownStatusCode>`.

### Errors

- `SmsRuError::Transport` (network/HTTP layer)
- `SmsRuError::HttpStatus { status: u16, body: Option<String> }` (non-2xx HTTP response; body captured for debugging when possible)
- `SmsRuError::Parse` (invalid JSON or unexpected format)
- `SmsRuError::Api { status_code, status_text }` (top-level API error)
- `SmsRuError::UnsupportedResponseFormat` (non-JSON response requested)
- `SmsRuError::Validation` (failed construction / invalid inputs)

Per-recipient errors should not automatically map to `SmsRuError::Api` when the top-level response is OK.

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
