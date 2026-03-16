# AgentRoom — Claude Code Instructions

## Repo

- **Owner repo**: `NooRotic/agentroom-win` — this is the user's own repo, safe to push
- **Upstream (read-only)**: `liuyixin-louis/agentroom` — DO NOT push here, user does not own it
- **Branch protection**: `main` requires PRs; always work on a feature branch and open a PR
- All feature branches merge into `main` via PR on GitHub

## Project Overview

Tauri desktop app (Rust backend + React/TypeScript frontend) that shows AI coding agents as animated pixel art characters in a virtual office. Supports Claude Code, Codex, and Gemini agents.

Key components:
- `src-tauri/src/` — Rust backend: file watcher, transcript parser, agent state manager, Tauri commands
- `src/` — React frontend: office canvas, session list, search, transcript viewer
- `search-backend/cass/` — vendored CASS search backend (Rust binary, SQLite + tantivy index)
- `search-backend/frankentui/` — vendored frankentui TUI framework
- `public/assets/` — pixel art sprites, tilesets, layout JSON

## Windows-Specific Rules

- Always use `USERPROFILE` fallback when `HOME` may be unset (native Windows processes)
- Path detection in Rust must use `path.components()` not string contains with `/` — Windows uses backslashes
- CASS binary is `cass.exe` on Windows; build with `cargo build --release` in `search-backend/cass/`
- frankentui native-backend items must be gated `#[cfg(all(unix, feature = "native-backend"))]`

## Architecture Notes

### Agent type → palette mapping
`officeState.ts` `AGENT_TYPE_PALETTES`:
- `claude-code` → palettes [0, 1]
- `codex` → palettes [2, 3]
- `gemini` → palettes [4, 5]

### Agent type brand colors (used in renderer + ToolOverlay)
- `claude-code` → `#D97941` (Anthropic orange)
- `gemini` → `#4B8DF8` (Google blue)
- `codex` → `#10A37F` (OpenAI teal)

### File watcher
`src-tauri/src/file_watcher.rs` — when `project_dir` is empty, watches all `~/.claude/projects/` subdirs recursively. When a specific workspace is passed (via `switchWatching`), watches only that project dir non-recursively.

### Session staleness
`src/App.tsx` auto-indexes CASS every 2 minutes. Manual reindex via the "Reindex" button in `StatusBar`. Sessions reload after both.

### Key data flow
JSONL file change → `file_watcher.rs` → `transcript_parser.rs` → `agent_state.rs` → Tauri event `agent-state-changed` → `useAgentEvents.ts` → `OfficeState` → canvas render

## Ideas & TODO

See `docs/ideas-todo.md` for a prioritized list of planned features with effort estimates.
