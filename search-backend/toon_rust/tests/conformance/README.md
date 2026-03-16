# Conformance Harness (Skeleton)

This directory documents the planned conformance harness for `toon`.

Goals:

- Run spec fixtures from the TOON spec repository.
- Compare Rust output with expected values from fixtures.
- Provide clear logging and per-case failure context.

Fixture sources:

1. External spec repo (preferred):
   - `legacy_spec/tests/fixtures`
2. Vendored copy (current):
   - `tests/fixtures/spec`

Planned usage:

```
TOON_SPEC_FIXTURES=legacy_spec/tests/fixtures \
cargo test --features conformance -- --ignored
```

Notes:

- The harness is currently a skeleton in `tests/conformance.rs`.
- All conformance tests are `#[ignore]` until the core library is implemented.
