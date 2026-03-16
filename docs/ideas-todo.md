# AgentRoom — Ideas & TODO

Brainstormed 2026-03-16. Rough priority order within each section.

---

## Visual Polish

- [ ] **Tool icons in speech bubble** — show a small emoji matching the active tool instead of just a status dot
  - 🔍 Grep/Glob, ✏️ Edit/Write, 🖥️ Bash, 🌐 WebFetch/WebSearch, 📖 Read
  - Tool name already arrives in `tool_name` on `tool_start` events — map to emoji in renderer
- [ ] **Animated desk monitors** — when an agent goes active, the monitor on their desk shows a scrolling effect
  - Matrix effect already exists (`matrixEffect` on `Character`); repurpose for monitor sprites
- [ ] **Completion sparkle** — small pixel burst above character when a turn ends (task finished)
  - Similar to matrix effect, short-lived particle system centered on character
- [ ] **Agent nameplate** — permanent small label above each character showing project folder name
  - `ch.folderName` is already set; render it below the agent-type dot unconditionally (not just on hover)

---

## Office Life

- [ ] **Water cooler gathering** — idle agents pathfind toward the cooler instead of just sitting
  - Needs cooler tile coords; add an "idle gather point" concept to `OfficeState`
- [ ] **Sub-agent proximity seating** — Task sub-agents spawn at a desk adjacent to their parent
  - Currently picks closest free seat; tighten the distance filter when `isSubagent === true`
- [ ] **Per-agent-type audio** — distinct sound cue when each agent type goes active vs. idle
  - Extend `notificationSound.ts`; `playDoneSound()` is already called on `turn_end`

---

## Sidebar / Sessions

- [ ] **Session activity sparklines** — tiny inline chart next to each session row showing activity over the day
  - Derive from session `startedAt` + tool event timestamps if available in CASS
- [ ] **Jump to agent from session** — keyboard shortcut / button that focuses the office camera on the character matching the selected session
  - `handleClick` in App.tsx already does the reverse (agent → session); invert it
- [ ] **"Last active X min ago" timestamp** — show relative time on each session row
  - `session.startedAt` exists; add a formatted relative-time display
- [ ] **Open workspace button** — one-click to open the project folder in Explorer or VS Code
  - Use Tauri `shell.open()` on `session.workspace`; add icon button to project group header

---

## Info Panels

- [ ] **Live token burn rate** — rolling tokens/min graph in TokenPanel, not just totals
  - Sample token counts on a timer; keep a sliding window of (timestamp, delta) pairs
- [ ] **Per-project agent count badge** — show `N active / M total` on each project group header in the session list
- [ ] **OS desktop notifications** — system-level notification when any agent hits a permission bubble
  - Tauri `notification` plugin; trigger on `permission` status event

---

## Quality of Life

- [ ] **Persist focused project** — remember last focused project across restarts via `localStorage`
- [ ] **Middle-mouse pan** — pan the office canvas with middle-mouse button in addition to current method
  - Add `mousedown` handler for `button === 1` in `OfficeCanvas.tsx`
- [ ] **Follow active agent mode** — toggle that keeps the camera centered on whichever agent is currently typing
  - `officeState.cameraFollowId` already exists but is unused; wire it up in the game loop
- [ ] **System tray integration** — minimize to tray; badge showing count of currently active agents
  - Tauri `system-tray` plugin

---

## Effort Notes

| Item | Estimate |
|------|----------|
| Agent nameplate (always-on label) | ~1 hr |
| Last active timestamp | ~1 hr |
| Open workspace button | ~1 hr |
| Follow active agent mode | ~2 hrs (cameraFollowId plumbing) |
| Middle-mouse pan | ~1 hr |
| Tool icons in bubble | ~2 hrs |
| Per-agent-type audio | ~2 hrs |
| OS desktop notifications | ~2 hrs |
| Completion sparkle | ~3 hrs |
| Sub-agent proximity seating | ~2 hrs |
| Token burn rate graph | ~3 hrs |
| Session sparklines | ~4 hrs |
| Water cooler gathering | ~4 hrs |
| System tray | ~4 hrs |
