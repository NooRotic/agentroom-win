# Proposed Architecture for `tru` (toon_rust)

## Using Rust Best Practices from beads_rust

> **Status:** Proposed Architecture  
> **Reference:** `EXISTING_TOON_RUST_STRUCTURE.md` (spec)  
> **Exemplar:** `/dp/beads_rust` conventions (edition, linting, release profile)

---

## Executive Summary

This document defines the Rust architecture for `tru`, the TOON reference port.
It mirrors the TypeScript implementation's behavior while using Rust best practices
and the conventions established in `beads_rust`.

Key principles:

1. **Spec-first**: code implements the spec doc only.
2. **Streaming-first**: line-by-line encode, event-based decode.
3. **Deterministic output**: stable, byte-for-byte output.
4. **No unsafe code**: `#![forbid(unsafe_code)]`.
5. **Minimal dependencies**: small, fast binary.

---

## Non-Negotiable Requirements

| Requirement | Description |
|------------|-------------|
| **Spec parity** | Must match TypeScript behavior exactly |
| **CLI parity** | Flags, defaults, and outputs match `toon` CLI |
| **Streaming decode** | Event stream matches reference semantics |
| **Key folding + expansion** | Safe-mode behavior identical |
| **Strict validation** | Counts, blank lines, indentation rules |
| **No unsafe code** | Enforced by lint config |

---

## High-Level System Map

```
[CLI Layer]
  -> clap derive + IO routing + stats + success/error formatting
  -> [Core Library]
     -> encode (normalize/folding) + decode (scanner/parser/events)
     -> [Shared Utils]
        -> string escaping + validation + JSON event formatter
```

---

## 1. Project Structure

### 1.1 Directory Layout

```
toon_rust/
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  build.rs
  src/
    main.rs                # CLI entry (thin wrapper)
    lib.rs                 # Public API re-exports
    error.rs               # Error types and formatting
    options.rs             # Encode/Decode options structs
    encode/
      mod.rs
      normalize.rs
      primitives.rs
      encoders.rs
      folding.rs
      replacer.rs
    decode/
      mod.rs
      scanner.rs
      parser.rs
      decoders.rs
      event_builder.rs
      expand.rs
      validation.rs
    cli/
      mod.rs
      args.rs            # clap derive
      conversion.rs      # encode/decode plumbing
      json_stream.rs     # stream JSON from events
      json_stringify.rs  # streaming stringify
    shared/
      mod.rs
      constants.rs
      string_utils.rs
      literal_utils.rs
      validation.rs
  tests/
    unit/
    integration/
    conformance/
```

### 1.2 Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `encode` | Normalize + encode JSON to TOON lines |
| `decode` | Parse TOON lines to event stream and/or values |
| `cli` | CLI arguments, IO, stats, success/failure formatting |
| `shared` | String escaping, literal parsing, validation helpers |
| `error` | Error taxonomy and user-facing messages |

---

## 2. Public API Shape (Library)

Expose the same top-level API as TypeScript:

- `encode(input, options) -> String`
- `encode_lines(input, options) -> Iterator<String>`
- `decode(input, options) -> JsonValue`
- `decode_from_lines(lines, options) -> JsonValue`
- `decode_stream_sync(lines, options) -> Iterator<JsonStreamEvent>`
- `decode_stream(source, options) -> AsyncStream<JsonStreamEvent>`

Rust types mirror `JsonValue` and `JsonStreamEvent`.

---

## 3. CLI Strategy

Use `clap` derive (aligned with beads_rust conventions).

Key CLI behaviors:

- Input detection: stdin or file path
- Mode detection: encode/decode flags or extension
- Streamed encode/decode outputs
- `--stats` requiring full output for token counting

Logging:

- Use `log`/`slog`-style or `tracing` per beads_rust pattern
- Diagnostics to stderr, JSON/TOON output to stdout

---

## 4. Error Handling

Define a structured error enum:

- `InvalidIndent`
- `InvalidDelimiter`
- `InvalidKeyFolding`
- `InvalidFlattenDepth`
- `InvalidExpandPaths`
- `DecodeError` (wraps Syntax/Type/Range errors)
- `EncodeError`
- `IoError`

Errors should render with user-friendly messages matching the reference CLI.

---

## 5. Dependency Strategy

Minimal deps, aligned with beads_rust:

- `clap` for CLI
- `serde_json` for JSON parsing and stringification (careful: must match stream output semantics)
- `thiserror` + `anyhow` for errors

Avoid heavy crates unless required for exact behavior.

---

## 6. Testing Strategy

1. **Unit tests** for normalization, encoding, decoding, and parsing rules.
2. **Golden tests** for known outputs from TypeScript reference.
3. **Conformance tests** from TOON spec repo fixtures.
4. **CLI integration tests** for flags, IO, and stats output.

All tests must pass under `cargo test` with strict clippy linting.
