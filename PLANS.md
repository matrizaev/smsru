# Implementation plan

This plan extends the crate with SMS.RU message status checks (`sms/status`) based on `SPEC.md` and `Проверить статус отправленных сообщений.pdf`.

## Implementation status

- M1: completed
- M2: completed
- M3: completed
- M4: completed
- M5: completed

## Scope

- Add a typed request/response API for `https://sms.ru/sms/status`.
- Keep existing `sms/send` behavior stable.
- Preserve DDD layering: `domain` (types/invariants), `transport` (wire format), `client` (orchestration).

## Milestones

### M1 - Domain model for status checks

- Add `SmsId` newtype (non-empty after trimming).
- Add `CheckStatus` request model with constructors:
  - `CheckStatus::new(Vec<SmsId>)`
  - `CheckStatus::one(SmsId)`
- Add limit constant for status checks: max `100` ids per request.
- Add response types:
  - `CheckStatusResponse`
  - `SmsStatusResult` (`status`, `status_code`, `status_text`, `cost`)
- Extend `ValidationError` with status-check-specific variants if needed (`TooManySmsIds`, etc.).
- Acceptance:
  - Unit tests cover constructors, limits, and validation failures.

### M2 - Transport encoding/decoding for `sms/status`

- Add transport encoder for form parameters:
  - `sms_id` as comma-separated ids
  - `json=1`
- Add transport decoder for JSON responses:
  - Top-level: `status`, `status_code`, optional `status_text`, optional `balance`, `sms`
  - Per id: `status`, `status_code`, optional `status_text`, optional `cost`
- Preserve unknown fields and optional fields via `serde` defaults.
- Normalize numeric/string money values into `Option<String>` for `cost` and `balance`.
- Acceptance:
  - Unit tests for form encoding and JSON fixtures (OK + ERROR + partial per-id failures).

### M3 - Client API integration

- Add `SmsRuClient::check_status(request) -> Result<CheckStatusResponse, SmsRuError>`.
- Add default status endpoint constant: `https://sms.ru/sms/status`.
- Extend builder config to support status endpoint override (while keeping current send configuration backward-compatible).
- Reuse existing error mapping strategy:
  - non-2xx -> `SmsRuError::HttpStatus`
  - parse issues -> `SmsRuError::Parse`
  - top-level `status != OK` -> `SmsRuError::Api`
- Acceptance:
  - Client tests with mocked transport verify request params and error mapping.

### M4 - Public exports and docs

- Re-export new public types from `src/lib.rs`.
- Update crate docs and README with a status-check example.
- Keep `SPEC.md` and API docs consistent with implemented names.
- Acceptance:
  - Doctests/examples compile (`no_run` is acceptable).

### M5 - Quality gates

- Run `cargo fmt`.
- Run `cargo clippy --all-targets --all-features` and fix warnings (no blanket `allow`).
- Run `cargo test`.
- Acceptance:
  - All checks pass without network-dependent tests.

## Risks and decisions

- Endpoint configuration was split into method-specific fields (`send_endpoint`, `status_endpoint`) while keeping `endpoint(...)` for backward compatibility.
- SMS.RU may return money values as numbers; public API should preserve exact text (`String`) to avoid precision loss.
- Unknown `sms` map keys in status response should be treated predictably (explicit parse/transport error, not silent drop).
