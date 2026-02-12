# smsru Roadmap

This plan tracks implementation milestones for the typed SMS.RU client.

## Completed

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

## Next

- [ ] Add optional decimal helper parsing for money-like fields (`balance`, `cost`, `total_cost`) behind a Cargo feature.
- [ ] Add integration-style tests with mock HTTP server fixtures (no live SMS.RU calls).
- [ ] Expand examples for per-recipient `CheckCost::per_recipient(...)`.
- [ ] Evaluate a blocking client feature for non-async applications.
