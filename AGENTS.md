# Agent guidelines (repo-wide)

These instructions apply to all files in this repository.

## Rust quality bar

- Prefer idiomatic, stable Rust; keep the public API small and coherent.
- Run `cargo fmt` and keep code `clippy`-clean (no `allow` unless justified).
- Avoid panics in library code: no `unwrap`, `expect`, or `todo!` in non-test code.
- Favor explicit error types and clear invariants over implicit behavior.
- Keep changes minimal and well-scoped; don’t refactor unrelated code.

## Domain-driven design (DDD)

- Model the SMS.RU domain explicitly: auth, recipients, message content, sender, scheduling, delivery results, and status codes.
- Separate layers:
  - **domain**: pure types + validation + invariants (no I/O)
  - **transport**: HTTP request/response + serialization details
  - **service/client**: orchestrates calls, maps transport ↔ domain
- Keep SMS.RU wire-format quirks contained to the transport layer.

## Type-driven development (strong internal types)

- Use strong types internally (newtypes) to prevent invalid states:
  - `ApiId`, `Login`, `Password`
  - `PhoneNumber`, `SenderId`
  - `MessageText` (UTF-8, length constraints)
  - `UnixTimestamp`, `TtlMinutes`
  - `PartnerId`
  - `StatusCode` (known + unknown preserved)
- Represent alternative request shapes with enums (e.g., one message to many vs per-recipient messages).
- Validate early at construction time; keep constructors fallible where needed.
- Keep conversions to/from `String` explicit; avoid “stringly typed” APIs.

## Error handling

- Provide a single crate error type (e.g., `SmsRuError`) with variants for:
  - transport/HTTP failures
  - API-level failures (top-level status != OK)
  - parse/validation failures
- Preserve SMS.RU codes and optional texts; never discard unknown codes.

## Serialization and HTTP

- Default to JSON responses (`json=1`) per `SPEC.md`.
- Prefer `serde` for JSON mapping; avoid ad-hoc JSON traversal.
- Prefer `reqwest` as transport; keep it behind a small abstraction if tests need mocking.

## Testing

- Add unit tests for domain validation and request serialization.
- Avoid real network calls in tests; use mocking or local fixtures.
- Keep tests deterministic and fast.
