# Macro Recorder - Spec + UX Flow

This spec defines the UX, timing model, safety rules, and test requirements for
**Input Macro Recorder + Scenario Runner** (bd-2lus.1).

---

## 1) Goals
- Record user input events with timestamps and replay deterministically.
- Provide a clear, keyboard-first control flow (record, stop, play, loop).
- Normalize timing so playback is stable across machines.
- Support curated scenario macros with descriptions.

## 2) Non-Goals
- Networked macro sharing.
- Persisted global history across sessions (unless explicitly added later).
- Editing macros in-place (future enhancement).

---

## 3) Scope & Safety Rules
- **Scope**: macros are recorded at the demo showcase app level (not per widget).
- **Safety**: playback never triggers destructive commands; only in-app actions.
- **Isolation**: macro execution must not bypass the one-writer rule.

---

## 4) UX States

### 4.1 Idle (Default)
- Recorder UI shows: [Record] [Play] [Loop] [Speed]
- No macro loaded.

### 4.2 Recording
- UI shows live duration + event count.
- Status line: "Recording... (Esc to stop)"

### 4.3 Stopped (Macro Ready)
- Macro is available for playback.
- UI shows duration + event count.

### 4.4 Playing
- UI shows progress + speed multiplier.
- If playback fails, enter Error state.

### 4.5 Error
- UI shows error message + last failed event index.
- Recovery: Esc -> Idle (macro remains saved unless invalid).

---

## 5) Controls & Keybindings
- `r` Toggle Record / Stop
- `p` Play / Pause
- `l` Toggle Loop
- `+` / `-` Adjust playback speed (0.25x - 4.0x)
- `Esc` Stop playback or exit recording

Help overlay must list these bindings.

---

## 6) Timing Model

### 6.1 Normalized Timing
- Record absolute timestamps (monotonic, ms).
- Normalize on replay using recorded deltas (not wall clock).

### 6.2 Deterministic Mode
- Optional fixed-step playback (e.g., 16ms tick grid).
- Ensures deterministic E2E tests and stable replays.

### 6.3 Drift Handling
- On replay, compute drift = |actual - scheduled| per event.
- If drift exceeds threshold, log warning but continue.

---

## 7) Data Model

### 7.1 Macro Format
```json
{
  "id": "macro-001",
  "created_ts": 1730000000000,
  "events": [
    {"t": 0, "event": "Key('g')"},
    {"t": 52, "event": "Key('g')"},
    {"t": 120, "event": "Key('d')"}
  ]
}
```

### 7.2 Scenario Runner
- Scenario = macro + description + expected outcome
- Scenario list displayed in a dedicated panel

---

## 8) Failure Modes (Ledger)
- Missing events: corrupted macro data.
- Timing drift > threshold.
- Playback canceled due to resize/focus loss.
- Input parsing errors.

Evidence ledger fields:
- decision: {record, stop, play, loop, error}
- reason: {user_input, timer, validation_failure}
- event_index, drift_ms

---

## 9) Performance & Optimization Protocol
- Baseline p50/p95/p99 replay latency with `hyperfine`.
- Profile CPU + allocations; identify top 5 hotspots.
- Opportunity matrix; implement only Score >= 2.0 (Impact x Confidence / Effort).
- Isomorphism proof: deterministic ordering + golden checksums.

---

## 10) Test Plan

### Unit Tests
- State machine: record -> stop -> play -> idle transitions.
- Serialization/deserialization roundtrip.
- Deterministic replay under fixed-step mode.

### Property Tests
- Determinism: same macro -> same event sequence.
- Timing normalization does not reorder events.

### Snapshot Tests
- Idle state
- Recording state
- Playback state
- Error state

### PTY E2E
- Record -> stop -> replay
- JSONL logs with macro id, event count, replay drift, outcome

---

## 11) JSONL Logging Fields (Required)
- ts_ms
- macro_id
- event_index
- event_type
- scheduled_ms
- actual_ms
- drift_ms
- state (recording, stopped, playing, error)
- outcome (ok, canceled, failed)

---

## 12) Integration Notes
- Recorder lives in demo showcase runtime layer.
- Macro storage is in-memory for now; optional serialization later.
- Playback must respect inline/alt-screen mode without flicker.

