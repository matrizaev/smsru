# smsru Roadmap

This plan tracks implementation milestones for the typed SMS.RU client.

## Completed baseline

- [x] Domain layer with strong value types (`ApiId`, `Login`, `Password`, `RawPhoneNumber`, `SmsId`, `StatusCode`, etc.).
- [x] `sms/send` support:
  - typed request model (`SendSms`, `SendOptions`)
  - form encoding + JSON decoding
  - client method `SmsRuClient::send_sms(...)`.
- [x] `sms/status` support:
  - typed request model (`CheckStatus`)
  - form encoding + JSON decoding
  - client method `SmsRuClient::check_status(...)`.
- [x] `sms/cost` support:
  - typed request model (`CheckCost`, `CheckCostOptions`)
  - typed response model (`CheckCostResponse`, `SmsCostResult`)
  - form encoding + JSON decoding
  - client method `SmsRuClient::check_cost(...)`.
- [x] Endpoint-specific builder overrides:
  - `send_endpoint(...)`
  - `cost_endpoint(...)`
  - `status_endpoint(...)`.

## Current milestone: phone-call authentication (`callcheck/add`, `callcheck/status`)

Source of truth: `Авторизовать пользователя по звонку с его номера.pdf` (SMS.RU docs export, updated 16 June 2025).

### 1) Domain layer

- [ ] Add strong value types:
  - `CallCheckId` (non-empty after trimming)
  - `CallCheckStatusCode` (known + unknown preserved)
- [ ] Add request models:
  - `StartCallAuth` + `StartCallAuthOptions`
  - `CheckCallAuthStatus` + `CheckCallAuthStatusOptions`
- [ ] Add response models:
  - `StartCallAuthResponse`
  - `CheckCallAuthStatusResponse`
- [ ] Keep invariants explicit and constructor-first validation (no implicit defaults besides `json=1`).

### 2) Transport layer

- [ ] Add form serializers:
  - `encode_start_call_auth_form(...)`
  - `encode_check_call_auth_status_form(...)`
- [ ] Add JSON decoders for:
  - `callcheck/add` response (`check_id`, `call_phone`, `call_phone_pretty`, `call_phone_html`)
  - `callcheck/status` response (`check_status`, `check_status_text`)
- [ ] Parse `check_status` from either numeric or numeric-string JSON values.
- [ ] Keep wire-format quirks isolated in transport DTOs.

### 3) Client layer

- [ ] Add endpoint constants:
  - `https://sms.ru/callcheck/add`
  - `https://sms.ru/callcheck/status`
- [ ] Extend `SmsRuClientBuilder` with endpoint overrides:
  - `callcheck_add_endpoint(...)`
  - `callcheck_status_endpoint(...)`
- [ ] Add high-level methods:
  - `SmsRuClient::start_call_auth(...)`
  - `SmsRuClient::check_call_auth_status(...)`
- [ ] Reuse existing error mapping behavior:
  - non-2xx -> `SmsRuError::HttpStatus`
  - top-level `status != OK` -> `SmsRuError::Api`
  - parse failures -> `SmsRuError::Parse`.

### 4) Status-code coverage

- [ ] Extend `KnownStatusCode` mapping for call-auth state/result codes:
  - `100` (request accepted)
  - `202` (invalid phone)
  - `400` (not confirmed yet)
  - `401` (confirmed)
  - `402` (expired or invalid `check_id`)
- [ ] Ensure unknown numeric values remain preserved and round-trippable.

### 5) Tests

- [ ] Unit tests for new domain constructors and validation failures.
- [ ] Unit tests for request serialization of both call-auth endpoints.
- [ ] Unit tests for JSON decoding:
  - successful start
  - successful status poll with `check_status = 401`
  - pending status (`400`)
  - expired/invalid status (`402`)
  - numeric-string `check_status` parsing.
- [ ] Client-layer tests with mocked transport:
  - API error mapping
  - HTTP status mapping
  - unsupported plain mode mapping.

### 6) Developer-facing docs/examples

- [ ] Add an executable example: `examples/call_auth.rs`.
- [ ] Update crate-level docs in `src/lib.rs` to include call-auth flow.
- [ ] Keep `SPEC.md`, `README.md`, and `CHANGELOG.md` synchronized after implementation.

## Next (after call-auth milestone)

- [ ] Add optional decimal helper parsing for money-like fields (`balance`, `cost`, `total_cost`) behind a Cargo feature.
- [ ] Add integration-style tests with mock HTTP server fixtures (no live SMS.RU calls).
- [ ] Evaluate a blocking client feature for non-async applications.
