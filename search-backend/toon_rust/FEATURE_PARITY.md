# Feature Parity: toon_rust vs TypeScript Reference

> **Status:** Complete
> **Last Updated:** 2026-01-20

## Summary

The `tru` (toon_rust) CLI achieves **full feature parity** with the TypeScript reference implementation. All encode/decode operations, CLI flags, and streaming behaviors match the spec.

## CLI Flags

| Flag | TypeScript | Rust | Status |
|------|------------|------|--------|
| `--encode`, `-e` | Yes | Yes | ✓ |
| `--decode`, `-d` | Yes | Yes | ✓ |
| `--output`, `-o FILE` | Yes | Yes | ✓ |
| `--delimiter CHAR` | Yes | Yes | ✓ |
| `--indent N` | Yes | Yes | ✓ |
| `--key-folding MODE` | Yes | Yes | ✓ |
| `--flatten-depth N` | Yes | Yes | ✓ |
| `--expand-paths MODE` | Yes | Yes | ✓ |
| `--no-strict` | Yes | Yes | ✓ |
| `--stats` | Yes | Yes | ✓ |
| Auto-detect by extension | Yes | Yes | ✓ |
| Stdin/stdout streaming | Yes | Yes | ✓ |

## Encode Features

| Feature | TypeScript | Rust | Status |
|---------|------------|------|--------|
| Primitives (null, bool, number, string) | Yes | Yes | ✓ |
| Arrays (homogeneous) | Yes | Yes | ✓ |
| Arrays (tabular) | Yes | Yes | ✓ |
| Arrays (mixed/nested) | Yes | Yes | ✓ |
| Objects | Yes | Yes | ✓ |
| Key folding (safe mode) | Yes | Yes | ✓ |
| Flatten depth control | Yes | Yes | ✓ |
| Custom delimiters | Yes | Yes | ✓ |
| Indentation control | Yes | Yes | ✓ |
| Deterministic output | Yes | Yes | ✓ |

## Decode Features

| Feature | TypeScript | Rust | Status |
|---------|------------|------|--------|
| Primitives | Yes | Yes | ✓ |
| Arrays | Yes | Yes | ✓ |
| Tabular arrays | Yes | Yes | ✓ |
| Objects | Yes | Yes | ✓ |
| Path expansion (safe mode) | Yes | Yes | ✓ |
| Strict validation | Yes | Yes | ✓ |
| Streaming events | Yes | Yes | ✓ |
| Error recovery | Yes | Yes | ✓ |

## Performance

Measured with hyperfine against the legacy TypeScript CLI (v2.1.0):

| Operation | TypeScript | Rust | Improvement |
|-----------|------------|------|-------------|
| Encode 336B | 82 ms | 3 ms | **27x faster** |
| Encode 144KB | 92 ms | 11 ms | **8x faster** |
| Encode 784KB | 105 ms | 24 ms | **4x faster** |
| Decode 379KB | 519 ms | 59 ms | **9x faster** |
| Startup | 66 ms | 1.1 ms | **60x faster** |
| Memory (784KB) | 68 MB | 8 MB | **8x less** |
| Binary size | 608KB + Node | 681KB standalone | No runtime |

## Test Coverage

| Test Type | Count | Status |
|-----------|-------|--------|
| Unit tests | 3 | ✓ Pass |
| CLI integration | 5 | ✓ Pass |
| Encode fixtures | 1 (many cases) | ✓ Pass |
| Decode fixtures | 1 (many cases) | ✓ Pass |
| JSON stream | 4 | ✓ Pass |
| Conformance (encode) | 1 (all fixtures) | ✓ Pass |
| Conformance (decode) | 1 (all fixtures) | ✓ Pass |

## Known Differences

None. The Rust implementation matches TypeScript behavior exactly for all tested cases.

## Optimizations Applied

1. **Pre-sized vectors** - Output vectors pre-allocated based on input size estimation
2. **Reference-based sibling tracking** - Avoid cloning strings for key comparison
3. **Direct string building** - Use `push`/`push_str` instead of `format!` in hot paths
4. **Reduced cloning in folding** - Minimize JsonValue clones during key chain traversal
5. **Single-buffer JSON stringify** - JSON output built into one pre-allocated buffer
6. **Parser pre-allocation** - Delimiter parsing pre-estimates Vec capacity
7. **Lazy content allocation** - Blank lines skip string allocation in scanner

## Architecture Compliance

| Requirement | Status |
|-------------|--------|
| No unsafe code | ✓ `#![forbid(unsafe_code)]` |
| Rust 2024 edition | ✓ |
| Clippy pedantic | ✓ No warnings |
| Streaming encode | ✓ Line-by-line |
| Event-based decode | ✓ JsonStreamEvent |
| Deterministic output | ✓ Byte-for-byte |
