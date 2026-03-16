# TOON Rust Integration Guide (toon_rust / tru)

This guide documents the public API, CLI behavior, error patterns, and recommended integration patterns for the TOON Rust implementation.

## Project Purpose and Architecture (Short)

- Purpose: Spec-first TOON encoder/decoder in Rust with deterministic output and strict validation.
- Binary: `tru` (src/main.rs) is a thin CLI wrapper that calls `toon_rust::cli::run()`. It is named `tru` to avoid conflicting with coreutils `tr`.
- Library: `toon_rust` exposes encode/decode functions and JSON event types for integration.
- Pipeline: encode uses normalize -> optional replacer -> key folding -> emit lines. Decode scans lines -> parses tokens -> builds events -> builds JSON tree (with optional path expansion).

Key modules:
- `src/encode/*`: normalization, key folding, emit lines, replacer hooks
- `src/decode/*`: scanning, parsing, validation, event building, path expansion
- `src/cli/*`: CLI args, streaming JSON chunk output, conversions

## Installation

### CLI (binary)

```
# Build and run locally
cargo build --release
./target/release/tru --help

# Or install from git
cargo install --git https://github.com/Dicklesworthstone/toon_rust --bin tru
```

### Library (Cargo.toml)

Preferred while crates.io versioning is in flux:

```
[dependencies]
toon_rust = { git = "https://github.com/Dicklesworthstone/toon_rust" }
```

For local integration:

```
[dependencies]
toon_rust = { path = "../toon_rust" }
```

## Public API Reference

### Core Functions

- `encode(input, options) -> String`
  - Encodes a JSON value into TOON.
  - `input` implements `Into<JsonValue>` (including `serde_json::Value`).

- `encode_lines(input, options) -> Vec<String>`
  - Same as `encode`, but returns line vector (no final join).

- `encode_stream_events(input, options) -> Vec<JsonStreamEvent>`
  - Emits JSON stream events equivalent to decoding TOON output.

- `try_decode(input, options) -> Result<JsonValue>`
  - Fallible decode, returns `Result`.

- `decode(input, options) -> JsonValue`
  - Panics on error. Use `try_decode` for non-panicking path.

- `try_decode_from_lines(lines, options) -> Result<JsonValue>`
- `decode_from_lines(lines, options) -> JsonValue`

- `try_decode_stream_sync(lines, options) -> Result<Vec<JsonStreamEvent>>`
- `decode_stream_sync(lines, options) -> Vec<JsonStreamEvent>`

- `try_decode_stream(lines, options) -> Result<Vec<JsonStreamEvent>>` (async wrapper)
- `decode_stream(lines, options) -> Vec<JsonStreamEvent>` (async wrapper)

### Core Types

- `JsonValue`
  - `Primitive(JsonPrimitive)`
  - `Array(Vec<JsonValue>)`
  - `Object(Vec<(String, JsonValue)>)`

- `JsonPrimitive` is `StringOrNumberOrBoolOrNull`:
  - `String(String)`
  - `Number(f64)`
  - `Bool(bool)`
  - `Null`

- `JsonStreamEvent`
  - `StartObject` / `EndObject`
  - `StartArray { length: usize }` / `EndArray`
  - `Key { key: String, was_quoted: bool }`
  - `Primitive { value: JsonPrimitive }`

### Options

- `EncodeOptions`
  - `indent: Option<usize>` (default 2)
  - `delimiter: Option<char>` (default ',')
  - `key_folding: Option<KeyFoldingMode>` (default Off)
  - `flatten_depth: Option<usize>` (default usize::MAX)
  - `replacer: Option<EncodeReplacer>`

- `DecodeOptions`
  - `indent: Option<usize>` (default 2)
  - `strict: Option<bool>` (default true)
  - `expand_paths: Option<ExpandPathsMode>` (default Off)

- `DecodeStreamOptions`
  - `indent: Option<usize>`
  - `strict: Option<bool>`

- `KeyFoldingMode`: `Off | Safe`
- `ExpandPathsMode`: `Off | Safe`

### Error Handling

- Library errors are `ToonError::Message { message: String }` and `Result<T>` alias.
- `decode()` and `decode_from_lines()` panic on errors (they call the fallible versions and unwrap).
- CLI prints error to stderr and exits with code 1.

## CLI Reference (tru)

Auto-detection:
- `.json` -> encode
- `.toon` -> decode
- No input or `-` -> stdin, defaults to encode unless `--decode` is set

Flags:
- `-o, --output <FILE>`: output file (stdout if omitted)
- `-e, --encode`: force encode
- `-d, --decode`: force decode
- `--delimiter <,|\t|\|>`: default ','
- `--indent <0..=16>`: default 2
- `--no-strict`: disable strict decoding checks
- `--key-folding <off|safe>`: encode-only
- `--flatten-depth <N>`: encode-only
- `--expand-paths <off|safe>`: decode-only
- `--stats`: encode-only token estimate (prints stats to stderr)

Examples:

```
tru input.json                  # encode to TOON
tru input.toon                  # decode to JSON
tru input.json -o output.toon
cat data.json | tru --encode
cat data.toon | tru --decode
tru input.json --stats
```

## Integration Patterns

### 1) Basic Encode/Decode

```rust
use toon_rust::{encode, decode};
use toon_rust::options::{EncodeOptions, DecodeOptions, KeyFoldingMode, ExpandPathsMode};

let value: serde_json::Value = serde_json::json!({"user": {"id": 1, "name": "Ada"}});

let toon = encode(value.clone(), Some(EncodeOptions {
    indent: None,
    delimiter: None,
    key_folding: Some(KeyFoldingMode::Safe),
    flatten_depth: None,
    replacer: None,
}));

let decoded = decode(&toon, Some(DecodeOptions {
    indent: None,
    strict: None,
    expand_paths: Some(ExpandPathsMode::Safe),
}));
```

### 2) OutputFormat Enum Pattern

Use a format selector (flag + env) so tools can emit JSON or TOON without rework.

```rust
#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Json,
    Toon,
}

impl OutputFormat {
    fn from_env(default: OutputFormat) -> OutputFormat {
        match std::env::var("TOON_FORMAT").as_deref() {
            Ok("toon") => OutputFormat::Toon,
            Ok("json") => OutputFormat::Json,
            _ => default,
        }
    }
}
```

```rust
fn render_payload(value: serde_json::Value, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string()),
        OutputFormat::Toon => toon_rust::encode(value, None),
    }
}
```

### 3) Stats Pattern (mirrors tru)

Token estimate heuristic used by CLI:

```rust
fn estimate_tokens(text: &str) -> usize {
    let char_estimate = text.chars().filter(|c| !c.is_whitespace()).count() / 4;
    let word_estimate = text.split_whitespace().count();
    char_estimate.max(word_estimate).max(1)
}
```

Use with JSON input to report savings:

```rust
let json_text = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string());
let toon_text = toon_rust::encode(value, None);
let json_tokens = estimate_tokens(&json_text);
let toon_tokens = estimate_tokens(&toon_text);
let diff = json_tokens.saturating_sub(toon_tokens);
```

### 4) Streaming-Style Integration

The library exposes stream events (`JsonStreamEvent`) and the CLI provides a JSON chunk builder:

- `decode_stream_sync` -> events
- `cli::json_stream_from_events` -> JSON chunks

This enables large outputs without building a full JSON string in one allocation.

## Performance Notes (per README)

Encode benchmarks (hyperfine, 10 runs):

| Input Size | Node.js (toon) | Rust (tru) | Speedup |
| --- | --- | --- | --- |
| 336 B | 82 ms | 3 ms | 27x |
| 144 KB (1.5K rows) | 92 ms | 11 ms | 8x |
| 784 KB (5K rows) | 105 ms | 24 ms | 4x |

Decode benchmarks:

| Input Size | Node.js (toon) | Rust (tru) | Speedup |
| --- | --- | --- | --- |
| 379 KB TOON | 519 ms | 59 ms | 9x |

Memory and size notes:
- Startup time: 66 ms (Node) vs 1.1 ms (Rust)
- Memory (784 KB encode): 68 MB (Node) vs 8 MB (Rust)
- Binary size: 681 KB standalone (Rust) vs Node runtime + 608 KB

Token reduction estimates (typical JSON patterns):
- Simple object: ~40% fewer tokens
- Array of strings: ~57% fewer tokens
- Tabular data: ~60% fewer tokens
- Nested config: ~50% fewer tokens

These numbers are from the repo README; re-run locally for tool-specific payloads.

## Error Handling Patterns

- Use `try_decode` / `try_decode_from_lines` in production paths.
- Keep strict mode on by default; allow opt-out with `--no-strict` or config.
- Common strict-mode errors:
  - Count mismatch in list/tabular arrays
  - Blank lines inside arrays
  - Extra rows/items beyond declared length
  - Invalid indentation or tab usage

## Test Templates

### 1) Roundtrip Test

```rust
#[test]
fn roundtrip_encode_decode() {
    let input = serde_json::json!({"a": 1, "b": [true, false]});
    let toon = toon_rust::encode(input.clone(), None);
    let out = toon_rust::decode(&toon, None);
    assert_eq!(toon_rust::JsonValue::from(input), out);
}
```

### 2) Golden Test (insta)

```rust
#[test]
fn toon_snapshot() {
    let input = serde_json::json!({"user": {"id": 1, "name": "Ada"}});
    let toon = toon_rust::encode(input, None);
    insta::assert_snapshot!(toon);
}
```

### 3) Property Test (proptest)

```rust
proptest::proptest! {
    #[test]
    fn encode_decode_is_lossless(value in proptest::collection::vec(0u8..=255, 0..100)) {
        let json = serde_json::Value::Array(
            value.into_iter().map(|v| serde_json::Value::from(v as i64)).collect()
        );
        let toon = toon_rust::encode(json.clone(), None);
        let out = toon_rust::decode(&toon, None);
        assert_eq!(toon_rust::JsonValue::from(json), out);
    }
}
```

## Troubleshooting Checklist

- Decode errors: try `try_decode` and log the error string.
- Strict-mode failures: retry with `DecodeOptions { strict: Some(false), .. }`.
- Dotted paths: use `ExpandPathsMode::Safe` to expand `a.b.c` into nested objects.
- Mixed arrays: TOON chooses list or tabular based on structure; verify array is uniform.
