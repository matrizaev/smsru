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
- **Encoding**: parameters must be UTF-8 (`212` indicates wrong encoding)

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

Notes:

- The documentation examples use numbers like `7925...` (no leading `+`). The library should accept either format and transmit exactly what the user provided (except for trimming surrounding whitespace).

### Optional parameters

- `json=1` (recommended): request JSON response. The library should always send this unless explicitly overridden.
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

- `SendSms::ToMany { to: Vec<String>, msg: String, ... }`
- `SendSms::PerRecipient { messages: BTreeMap<String, String>, ... }`

The client must serialize these into the correct SMS.RU parameter format.

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
- `balance`: numeric balance after sending

Library behavior:

- If top-level `status != "OK"`, treat this as a request-level failure and return an error containing `status_code` and `status_text` (if present).
- If top-level `status == "OK"`, return a `SendSmsResponse` including per-recipient results. Per-recipient failures should not fail the whole call by default; they should be represented in the returned structure.

### Non-JSON response (legacy)

When `json` is not set, the server returns a plain-text multi-line response:

- First line: request status code (e.g., `100`)
- Next line(s): message ids or error codes per recipient (format varies)
- Last line may include `balance=...`

Library behavior:

- The library should default to JSON and only support non-JSON parsing behind an explicit opt-in API (since its format is less structured).

## Status codes

The API uses numeric codes both for request-level results and per-message delivery / error statuses. The crate should:

- Provide a `StatusCode` newtype or enum covering known codes.
- Preserve unknown numeric codes (forward compatibility).

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
  - `auth`: `Auth::ApiId(String)` or `Auth::LoginPassword { login: String, password: String }`
- `SmsRuClient::with_http_client(auth, http)` optionally constructs a client with a caller-supplied HTTP backend (initially likely `reqwest::Client`).
- `SmsRuClient::send_sms(request) -> Result<SendSmsResponse, SmsRuError>`

### Types

- `SendSms` request model:
  - recipients: `ToMany` or `PerRecipient`
  - options: `from`, `ip`, `time`, `ttl`, `daytime`, `translit`, `test`, `partner_id`
- `SendSmsResponse`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `balance`: `Option<f64>`
  - `sms`: `BTreeMap<String, SmsResult>`
- `SmsResult`:
  - `status`: `Ok`/`Error`
  - `status_code`: `StatusCode`
  - `status_text`: `Option<String>`
  - `sms_id`: `Option<String>`

### Errors

- `SmsRuError::Transport` (network/HTTP layer)
- `SmsRuError::Parse` (invalid JSON or unexpected format)
- `SmsRuError::Api { status_code, status_text }` (top-level API error)

Per-recipient errors should not automatically map to `SmsRuError::Api` when the top-level response is OK.

## Implementation notes (non-normative)

- JSON parsing: `serde` + `serde_json`.
- Transport: `reqwest` is the expected default backend; the crate may offer `async` and optional `blocking` clients via Cargo features.

## Examples (intended usage)

### Send one text to multiple recipients (JSON)

```rust
use smsru::{Auth, SendSms, SmsRuClient};

let client = SmsRuClient::new(Auth::ApiId("...".into()));
let resp = client.send_sms(SendSms::to_many(
    vec!["+79255070602".into(), "+74993221627".into()],
    "hello world".into(),
))?;

for (phone, result) in resp.sms {
    println!("{phone}: {:?} {:?}", result.status, result.status_code);
}
```

### Send different texts per recipient

```rust
use std::collections::BTreeMap;
use smsru::{Auth, SendSms, SmsRuClient};

let mut messages = BTreeMap::new();
messages.insert("+79255070602".into(), "Привет 1".into());
messages.insert("+74993221627".into(), "Привет 2".into());

let client = SmsRuClient::new(Auth::ApiId("...".into()));
let resp = client.send_sms(SendSms::per_recipient(messages))?;
```
