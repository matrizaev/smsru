# Implementation plan

This plan implements the `smsru` crate per `SPEC.md`.

## Milestones

### M0 — Project scaffolding

- Replace placeholder code in `src/lib.rs` with crate module layout.
- Add dependencies (initial):
  - `serde`, `serde_json`
  - `reqwest` (likely with `json`, `rustls-tls`)
  - `thiserror`
  - `phonenumber` (optional: parse/validate/normalize numbers for opt-in E.164 APIs)
  - `url` (optional; only if needed)
- Add crate-level docs with a short example.
- Acceptance: `cargo test` passes, `cargo fmt` clean.

### M1 — Domain layer (strong types + invariants)

Create `src/domain/` with:

- Newtypes with validation:
  - `ApiId`, `Login`, `Password`
  - `PhoneNumber` (opt-in E.164 normalization backed by `phonenumber`; region handling must be explicit)
  - `RawPhoneNumber` (opaque string wrapper for the default pass-through behavior)
  - `MessageText` (UTF-8 implied; validate non-empty and max length policy if desired)
  - `SenderId` (optional)
  - `UnixTimestamp`, `TtlMinutes`
  - `PartnerId`
- Core request model:
  - `SendSms` enum: `ToMany { recipients, msg, options }` vs `PerRecipient { messages, options }`
  - `SendOptions` struct: `from`, `ip`, `time`, `ttl`, `daytime`, `translit`, `test`, `partner_id`
- Response model:
  - `SendSmsResponse`, `SmsResult`
  - `Status` enum (`Ok`/`Error`)
  - `StatusCode(i32)` newtype + `KnownStatusCode` (non-exhaustive) + helpers (`known_kind`, optional `is_retryable`)
  - `balance: Option<String>` (preserve exact API formatting; no floats in the public model)
- Acceptance: unit tests for constructors/validation; no transport code yet.

### M2 — Transport layer (wire format + serde models)

Create `src/transport/` with:

- Request encoding:
  - Serialize `SendSms` into `application/x-www-form-urlencoded` parameters.
  - Implement `to` as comma-separated list for `ToMany`.
  - Implement `to[PHONE]=TEXT` expansion for `PerRecipient`.
  - Always include `json=1` by default.
- Response decoding:
  - `serde` structs mirroring SMS.RU JSON schema (`status`, `status_code`, `sms`, `balance`, optional `status_text`).
  - Use `#[serde(default)]` for optional fields; allow unknown fields (no `deny_unknown_fields`).
  - Convert transport structs into domain response types.
- Acceptance: unit tests for parameter encoding and JSON parsing fixtures.

### M3 — Client layer (public API)

Create `src/client/` with:

- `Auth` enum:
  - `ApiId(ApiId)` or `LoginPassword { login: Login, password: Password }`
- `SmsRuClient`:
  - `new(auth)` using an internal `reqwest` client (not exposed in public signatures)
  - optional `SmsRuClientBuilder`/config API for timeouts, user-agent, etc. (crate-owned types only)
  - `send_sms(request) -> Result<SendSmsResponse, SmsRuError>`
- Error type `SmsRuError`:
  - `Transport` (HTTP errors / timeouts)
  - `HttpStatus { status: u16, body: Option<String> }` (non-2xx response)
  - `Api { status_code, status_text }`
  - `Parse` (invalid JSON / schema drift)
  - `Validation` (failed construction / invalid inputs)
- Acceptance: integration-style tests using mocked HTTP (no real network).

### M4 — Status codes and ergonomics

- Provide a list/enum of known `StatusCode` values with readable helpers:
  - `is_retryable()`, `is_auth_error()`, etc. (keep minimal initially)
- Ensure unknown codes are preserved (forward compatible).
- Add convenience constructors:
  - `SendSms::to_many(...)`
  - `SendSms::per_recipient(...)`
- Acceptance: doc examples compile; clippy clean.

### M5 — Documentation and release readiness

- Ensure `SPEC.md` stays consistent with implemented API.
- Add `README.md` (if desired) with quickstart.
- Add a `CHANGELOG.md` entry for `0.1.0` if releasing.
- Acceptance: `cargo doc` renders cleanly; examples run as doctests (optional).

## Module layout (proposed)

- `src/lib.rs` — re-exports + top-level docs
- `src/domain/` — strong types + domain requests/responses
- `src/transport/` — wire-format encoding/decoding
- `src/client/` — `SmsRuClient`, `Auth`, `SmsRuError`

## Testing strategy

- Domain: unit tests for newtype validation and invariants.
- Transport: golden tests for form encoding; JSON fixtures for parsing.
- Client: HTTP mocking (e.g., `wiremock` or `httpmock`) to validate request/response mapping without network.

## Open decisions (resolve before locking public API)

- Sync vs async: async-only first, or provide `blocking` behind a feature.
- Phone number API shape: keep default pass-through (`RawPhoneNumber`) and define the opt-in E.164 parsing API (including explicit region input when needed).
- `MessageText` length rules: enforce “<= 8 SMS parts” as a soft validation or leave to server.
