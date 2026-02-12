# Changelog

## Unreleased

## 0.5.0 - 2026-02-12

### Added

- Add Milestone A account/auth support:
  - `StatusOnlyResponse`, `BalanceResponse`, `FreeUsageResponse`, `LimitUsageResponse`, `SendersResponse`
  - `SmsRuClient::check_auth()`
  - `SmsRuClient::get_balance()`
  - `SmsRuClient::get_free_usage()`
  - `SmsRuClient::get_limit_usage()`
  - `SmsRuClient::get_senders()`
  - `SmsRuClientBuilder` endpoint overrides:
    - `auth_check_endpoint(...)`
    - `my_balance_endpoint(...)`
    - `my_free_endpoint(...)`
    - `my_limit_endpoint(...)`
    - `my_senders_endpoint(...)`

- Add Milestone B stoplist support:
  - `StoplistText`, `AddStoplistEntry`, `RemoveStoplistEntry`, `StoplistResponse`
  - `SmsRuClient::add_stoplist_entry(...)`
  - `SmsRuClient::remove_stoplist_entry(...)`
  - `SmsRuClient::get_stoplist()`
  - `SmsRuClientBuilder` endpoint overrides:
    - `stoplist_add_endpoint(...)`
    - `stoplist_del_endpoint(...)`
    - `stoplist_get_endpoint(...)`

- Add Milestone C callback support:
  - `CallbackUrl`, `AddCallback`, `RemoveCallback`, `CallbacksResponse`
  - `SmsRuClient::add_callback(...)`
  - `SmsRuClient::remove_callback(...)`
  - `SmsRuClient::get_callbacks()`
  - `SmsRuClientBuilder` endpoint overrides:
    - `callback_add_endpoint(...)`
    - `callback_del_endpoint(...)`
    - `callback_get_endpoint(...)`

### Changed

- Keep API surface JSON-only for all implemented methods.
- Preserve unknown status codes consistently across all newly added decoding paths.
- Synchronize `SPEC.md` and `README.md` with the implemented endpoint surface.

### Release notes

- Release version: `0.5.0`.

## 0.4.0

### Added

- Add call authentication support (`callcheck/add`, `callcheck/status`):
  - Implement `StartCallAuth` and `CheckCallAuthStatus` domain request/response models.
  - Add transport encoding/decoding for call authentication requests and responses.
  - Update `SmsRuClient` to include call authentication endpoints:
    - `SmsRuClient::start_call_auth(...)`
    - `SmsRuClient::check_call_auth_status(...)`
  - Enhance domain models to support call-check IDs and status codes:
    - `CallCheckId`, `CallCheckStatusCode`, `KnownCallCheckStatusCode`
  - Add example usage in `examples/call_auth.rs`.
  - Add tests covering request encoding and response parsing.

## 0.3.0

### Added

- Add check cost support (`sms/cost`):
  - Implement `CheckCost` request and `CheckCostResponse` response structures in the domain layer.
  - Add `SmsRuClient::check_cost(...)` for checking SMS costs.
  - Introduce `CheckCostOptions` for optional parameters in cost requests.
  - Add transport encoding/decoding for cost requests and responses.
  - Add tests validating request limits and response parsing.
  - Update documentation with check-cost usage examples.

## 0.2.0

- Add typed `sms/status` support with `CheckStatus`, `CheckStatusResponse`, and `SmsStatusResult`.
- Add strong `SmsId` typing across public APIs (including `SendSmsResponse.sms[*].sms_id`).
- Add `SmsRuClient::check_status(...)` and method-specific builder endpoints:
  - `send_endpoint(...)`
  - `status_endpoint(...)`
- Add `examples/check_status.rs`.

## 0.1.0

- Initial release with domain, transport, and client layers.
- JSON-only `sms/send` support with strong request/response types.
