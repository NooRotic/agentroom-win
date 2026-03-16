# Plan: Port TOON Reference Implementation to Rust

> **Project:** toon_rust  
> **Binary Name:** `tru`  
> **Status:** Planning Phase (Spec-first)  
> **Reference Repo:** `legacy_toon/` (TypeScript reference implementation)

---

## Executive Summary

This plan defines a **spec-first** Rust port of the TOON reference implementation. The goal is to deliver a
single native binary (`tru`) and a Rust library that **matches the TypeScript behavior exactly**, including
streaming encode/decode, key folding, path expansion, strict validation, and CLI UX. The output must remain
fully compatible with the TOON v3.0 specification and the existing CLI contract.

This is not a translation. We extract the behavioral spec from the reference implementation and tests,
then implement from that spec, preserving edge cases and output semantics.

---

## Background: Legacy TOON

The legacy TypeScript implementation provides:

- **Core library** (legacy reference): encode/decode for JSON <-> TOON
- **Streaming decode** via event streams (no full in-memory object required)
- **Key folding** (safe, depth-limited)
- **Path expansion** (safe, optional, non-streaming)
- **CLI** (legacy reference) with stdin/stdout streaming and token stats

This Rust port must preserve:

- **Exact parsing rules** (quoting, escaping, strict mode)
- **Exact encoding decisions** (tabular array heuristics, folding rules)
- **JSON event stream semantics**
- **CLI flag behaviors and error messaging expectations**

---

## Goals (Must-Have)

1. **Spec fidelity**
   - Behaviors match the reference implementation for all supported features.
   - Decode/encode output is deterministic and lossless for JSON data model.

2. **Streaming-first**
   - Encoding yields lines incrementally.
   - Decode supports a streaming event interface.

3. **CLI parity**
   - CLI flags, defaults, and behaviors match the reference tool.
   - Auto-detection by file extension, stdin behavior, output formatting, and stats.

4. **Compatibility with TOON v3.0**
   - Format syntax is unchanged.
   - Conformance tests from spec repo must pass.

---

## Non-Goals (Explicit)

We do **not** port:

- Web docs, playgrounds, or marketing pages
- Monorepo tooling (pnpm, tsdown, automd, eslint, etc.)
- Editor plugins / VSCode extensions
- Benchmarks site generation (but keep core fixtures if needed for tests)
- Any JS-specific packaging logic (npm publishing, etc.)

---

## Source of Truth (Spec + Reference)

Primary sources for behavior:

1. **Reference implementation code** in `legacy_toon/packages/toon/`
2. **Reference CLI behavior** in `legacy_toon/packages/cli/`
3. **Reference tests** (encode/decode/stream/replacer) in `legacy_toon/packages/toon/test/`
4. **TOON spec repo** (external) for language-agnostic conformance fixtures

---

## Architecture Principles (Rust Port)

- **Spec-first**: implement from `EXISTING_TOON_RUST_STRUCTURE.md` only
- **No line-by-line translation**
- **Minimal dependencies** to keep binary small
- **Streaming decode built on explicit event stream**
- **Clear separation** between library and CLI

---

## Phase Plan

### Phase 1 - Spec Extraction (This Phase)

- Extract full behavior from reference implementation into `EXISTING_TOON_RUST_STRUCTURE.md`
- Enumerate all options, edge cases, and strict-mode validation rules
- Capture CLI behavior and output semantics
- Identify conformance fixtures and test sources

**Deliverables**
- Fully populated spec doc
- Proposed architecture doc
- Explicit non-goals and exclusions captured

### Phase 2 - Architecture + Scaffolding

- Finalize module structure in `PROPOSED_ARCHITECTURE.md`
- Align `Cargo.toml` with beads_rust style (edition, linting, release profile)
- Establish error taxonomy, logging, and public API shape

### Phase 3 - Core Library

- Implement encoding (including folding + delimiter rules)
- Implement decoding + streaming event model
- Implement normalization + replacer behavior
- Implement expandPaths (safe)

### Phase 4 - CLI Parity

- Implement CLI flags and validation logic
- Streaming IO paths for encode/decode
- Token stats (optional feature gate if needed)
- Error handling and exit codes

### Phase 5 - Conformance + QA

- Unit tests for encoding/decoding primitives and edge cases
- Golden tests for CLI outputs and streaming behavior
- Conformance tests against spec fixtures and legacy outputs
- Benchmark checks (optional)

---

## Conformance Testing Plan

We will build a conformance harness that:

1. Runs **legacy_toon** on fixture inputs to produce canonical outputs.
2. Runs **tru** on the same fixtures.
3. Compares output byte-for-byte.

Fixtures will cover:

- Encode of primitives, arrays, objects, mixed arrays, tabular arrays
- Decode strict vs non-strict modes
- Key folding and path expansion round-trips
- Quoting/escaping edge cases
- Streaming decode event sequences

### Spec Fixture Source

The TOON spec repo is cloned locally at:

```
legacy_spec/
  tests/
    README.md
    fixtures/
    fixtures.schema.json
```

We will use:

- `legacy_spec/tests/fixtures/encode/*.json`
- `legacy_spec/tests/fixtures/decode/*.json`

Each fixture defines input/output pairs and validation expectations.

Vendored copy (for CI and offline use):

- `tests/fixtures/spec/encode/*.json`
- `tests/fixtures/spec/decode/*.json`

---

## Success Criteria

**Functional**
- All reference tests pass or are matched by Rust equivalents
- All spec fixtures pass
- CLI behavior matches `toon` (inputs, outputs, errors)

**Quality**
- `cargo test` passes
- `cargo clippy --all-targets -- -D warnings` passes
- `cargo fmt --check` passes

**Performance**
- Streaming operations do not require full string buffering
- Binary size optimized using beads_rust release profile

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Spec drift vs reference | Tie tests to legacy outputs + spec fixtures |
| Streaming edge cases | Mirror event stream semantics exactly |
| Key folding conflicts | Explicit collision rules documented + tested |
| Path expansion conflicts | Strict vs non-strict behavior tested |

---

## Immediate Next Actions

1. Finalize spec extraction in `EXISTING_TOON_RUST_STRUCTURE.md`
2. Finalize module architecture in `PROPOSED_ARCHITECTURE.md`
3. Align project metadata (`Cargo.toml`, linting) with beads_rust
4. Start conformance test harness plan
