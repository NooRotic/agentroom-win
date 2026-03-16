# Log Search - Spec + UX Flow

This spec defines the user experience, behavior, and test requirements for
**Live Log Search + Filter** in the demo showcase (`bd-1b5h.1`).

---

## 1) Goals
- Fast, deterministic filtering of log output with clear visual feedback.
- Zero flicker in inline mode (stable UI while logs continue above).
- Consistent keyboard-first interaction (no mouse required).
- Correct, non-destructive highlighting (never corrupt ANSI/markup).

## 2) Non-Goals
- Full-text index persistence (no on-disk index).
- Multi-pane search history UI.
- Regex capture group rendering or replacements.

---

## 3) UX States

### 3.1 Closed (Default)
- Log viewer shows full log stream.
- No search chrome visible.

### 3.2 Active (Query Input)
- Search bar visible above the log viewer panel.
- Query updates filter results in real time.
- Results count + mode toggles visible on the right.

### 3.3 Empty Query
- Shows full log list.
- Inline hint text: "Type to filter ( / to search, Esc to close )".

### 3.4 No Results
- Show an explicit empty-state line: "No matches - Esc to close, / to edit."
- Still renders the search bar so the user can adjust query.

---

## 4) Input Model & Keybindings

### 4.1 Open / Close
- **Open search:** `/`
- **Close search:** `Esc` (restores full log)

### 4.2 Toggles (Active Search Only)
- **Regex mode:** `r` (toggles Literal <-> Regex)
- **Case sensitivity:** `c` (toggles Case Sensitive <-> Insensitive)
- **Context lines:** `n` (cycles 0 -> 1 -> 2 -> 5 -> 0)

### 4.3 Editing
- Standard text input with backspace/delete.
- Cursor within query field only (no multi-line query).

### 4.4 Help Overlay (Required)
- Help overlay must list:
  - `/` Open search
  - `Esc` Close search
  - `r` Toggle regex
  - `c` Toggle case
  - `n` Cycle context lines

---

## 5) Search Semantics

### 5.1 Matching
- **Literal mode:** substring match on rendered text.
- **Regex mode:** Rust regex engine; invalid regex shows error state and falls
  back to no matches until fixed.
- **Case sensitivity:** applies to both literal and regex.

### 5.2 Filtering Output
- When query is non-empty, only matching lines are shown.
- Context lines expand the view around each match (above + below).
- Context windows **merge** when overlapping (no duplicate lines).

### 5.3 Highlighting Rules
- All matching spans are highlighted in the log list.
- If navigation is added later, the "current" match uses a stronger highlight.
- Highlighting must preserve existing markup/ANSI (no style corruption).

---

## 6) Capability Tiers (Graceful Degradation)

- **Tier 0 (Off):** Extremely constrained budget -> search disabled; show hint.
- **Tier 1 (Lite):** Literal-only, no highlights, no context lines.
- **Tier 2 (Full):** Regex + highlights + context lines.

Tier selection should be deterministic based on render budget and log size.

---

## 7) Failure Modes (Ledger)
- **Regex parse errors:** shown inline; no crashes.
- **Highlight corruption:** prevents ANSI/style leakage.
- **Performance regressions:** filter latency > budget.

Evidence ledger fields:
- decision: {tier, literal/regex, case, context}
- reason: {budget, log_size, regex_error}
- observed latency + match count

---

## 8) Performance & Optimization Protocol
- Baseline filter latency (p50/p95/p99) with `hyperfine`.
- Profile CPU + allocations; identify top 5 hotspots.
- Opportunity matrix; implement only Score ≥ 2.0 (Impact×Confidence/Effort).
- Isomorphism proof: ordering + deterministic output checksums.

---

## 9) Tests (Required)

### Unit Tests
- Literal vs regex correctness.
- Case sensitivity toggle.
- Context line merging + no duplicates.
- Regex error state handling.

### Property Tests
- Determinism: same input stream + query -> same output.
- Idempotence: applying filter twice yields identical results.

### Snapshot Tests
- Empty query
- Matches (with highlights)
- No results

### PTY E2E
- Open `/` -> type -> toggle `r`/`c`/`n` -> `Esc`.
- JSONL logs with: query, mode, case, context, match_count, latency_ms.

---

## 10) Integration Notes
- Search UI belongs to the **LogViewer demo screen**.
- Search state lives in that screen's model (no global singleton).
- Help overlay entries are per-screen (`chrome::HelpEntry`).
