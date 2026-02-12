# Changelog

## Unreleased

- Add typed `sms/cost` support with:
  - `CheckCost`, `CheckCostOptions`
  - `CheckCostResponse`, `SmsCostResult`
  - `SmsRuClient::check_cost(...)`
  - `SmsRuClientBuilder::cost_endpoint(...)`
  - `examples/check_cost.rs`.

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
