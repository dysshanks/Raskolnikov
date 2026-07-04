# Phase 2 — TUI Skeleton

**Objective:** Build the Ratatui terminal UI with the 3-panel layout, event loop, and all keybindings. At this stage the TUI renders static placeholder content — no dynamic data flows yet.

---

## Files to Create / Modify

| File | Status | Purpose |
|------|--------|---------|
| `src/tui/mod.rs` | Create | Re-export `App`, `run_tui()` |
| `src/tui/app.rs` | Create | TUI state machine + event loop |
| `src/tui/layout.rs` | Create | Constraint-based 3-panel layout |
| `src/tui/input.rs` | Create | Crossterm event polling + keybinding dispatch |

---

## Dependencies (Cargo.toml — additions)

Already included in Phase 1: `ratatui`, `crossterm`, `tokio`.

---

## User Stories

| ID | Title | Description |
|----|-------|-------------|
| US-2.1 | TUI renders on launch | As a user, running `rsk` opens the Ratatui TUI immediately — no flicker, no startup delay |
| US-2.2 | Three panels visible | As a user, I see Tool Output (left), Conversation (right), and a bottom bar with Findings + input |
| US-2.3 | Input bar accepts text | As a user, I can type into the input bar at the bottom |
| US-2.4 | Enter sends message | As a user, pressing Enter sends the current input (queues it if tool running) |
| US-2.5 | Panel scrolling | As a user, I can scroll the active panel with PgUp/PgDn |
| US-2.6 | Tab switches focus | As a user, pressing Tab cycles scroll focus between the two main panels |
| US-2.7 | Ctrl+L clears tool output | As a user, pressing Ctrl+L clears the tool output panel |
| US-2.8 | Ctrl+C prompts to end session | As a user, pressing Ctrl+C at idle shows a confirmation dialog to end the session |
| US-2.9 | Ctrl+C kills running tool | As a user, pressing Ctrl+C while a tool is running sends SIGTERM to the child process |
| US-2.10 | Quit command | As a user, typing `/quit` in the input bar triggers the end-session prompt |

---

## Acceptance Criteria

| ID | Criterion | How to Verify |
|----|-----------|---------------|
| AC-2.1 | TUI opens fullscreen and shows the RASKOLNIKOV header with version | Launch `rsk` |
| AC-2.2 | Left panel header shows "TOOL OUTPUT" | Visual inspection |
| AC-2.3 | Right panel header shows "CONVERSATION" | Visual inspection |
| AC-2.4 | Bottom bar shows "FINDINGS" label on left, `> _` input prompt on right | Visual inspection |
| AC-2.5 | Typing in input bar displays characters | Type "hello world" |
| AC-2.6 | Pressing Enter clears input bar and adds message to conversation panel (right) | Type a message, press Enter |
| AC-2.7 | PgUp scrolls active panel up, PgDn scrolls down | Fill conversation panel with content, test scroll |
| AC-2.8 | Tab switches which panel PgUp/PgDn controls | Press Tab, verify focus indicator moves |
| AC-2.9 | Ctrl+L clears left panel content | Type some placeholder text in tool panel, press Ctrl+L |
| AC-2.10 | Ctrl+C at idle shows "End session? [Y/n]" overlay | Press Ctrl+C |
| AC-2.11 | Pressing Y or Enter on the prompt ends the session and exits TUI | Confirm prompt |
| AC-2.12 | Pressing N on the prompt dismisses it, session continues | Cancel prompt |
| AC-2.13 | Input bar shows queued indicator when tool is running | Simulate a running tool, type a message — see visual cue |
| AC-2.14 | `/quit` in input bar triggers same prompt as Ctrl+C | Type `/quit`, press Enter |
| AC-2.15 | Tool output panel is dimmed/empty when no tool is running | Visual inspection |
| AC-2.16 | Terminal is restored to clean state on exit | Quit the app, terminal should show shell prompt cleanly |
| AC-2.17 | Resizing terminal re-renders layout proportionally | Resize terminal window, panels adjust |

---

## Technical Design

### App State Machine

```
         ┌──────────────────────────────────────────────┐
         │                                              │
         v                                              │
    ┌──────────┐     tool proposed     ┌───────────────┐│
    │   Idle   │ ───────────────────>  │ AwaitingConfirm││
    └──────────┘                       └───────┬───────┘│
         ^                                     │        │
         │                          operator   │        │
         │                          approves   │        │
         │                                     v        │
         │                              ┌──────────┐    │
         │                              │ToolRunning│    │
         │                              └─────┬────┘    │
         │                      tool          │         │
         │                    completes       │         │
         └────────────────────────────────────┘         │
                                                        │
         ┌─────────────┐                                │
         │ Interrupted │ <────── Ctrl+C while running    │
         └─────────────┘                                │
                   │                                    │
                   └────────────────────────────────────┘
```

- **Idle**: Input bar focused. No tool running. Operator types and sends messages.
- **AwaitingConfirmation**: Agent proposed a tool. Input expects "yes"/"no"/modified command.
- **ToolRunning**: A subprocess is executing. Input bar accepts queued messages.
- **Interrupted**: User pressed Ctrl+C during tool run. Tool got SIGTERM. Awaiting user direction.

### Layout Constraints

```
┌─────────────────────────────────────────────────────────┐
│  RASKOLNIKOV  alpha 0.1.0              model: qwen3     │  ← 1 line header
├────────────────────────────┬────────────────────────────┤
│                            │                            │
│  TOOL OUTPUT               │  CONVERSATION              │  ← main area
│                            │                            │  (remaining height - 2)
│                            │                            │
│  [lines of tool output]    │  you  scan 10.0.0.1        │
│                            │                            │
│                            │  agent Starting with nmap  │
├────────────────────────────┴────────────────────────────┤
│  FINDINGS  ·  22/tcp ssh  ·  80/tcp http                │  ← 1 line findings bar
├─────────────────────────────────────────────────────────┤
│  > _                                                     │  ← 1 line input bar
└─────────────────────────────────────────────────────────┘
```

Ratatui `Constraint` ratios:
- Header: `Length(1)`
- Main panels: `Min(0)` — takes remaining space
- Findings bar: `Length(1)`
- Input bar: `Length(1)`

Main panels split horizontally with `Percentage(50)` / `Percentage(50)`.

### Rendering

- Standard Ratatui `Terminal::draw()` with double-buffering
- Re-render on input events + every 100ms tick for live tool output
- Active panel gets a highlight border or colour change
- Tool output panel dimmed (lower opacity colour) when no tool running

### Keybinding Dispatch

```
Enter         → App::submit_input()
PgUp          → App::scroll_active_panel(-1)
PgDn          → App::scroll_active_panel(1)
Tab           → App::cycle_focus()
Ctrl+C        → App::handle_interrupt()
Ctrl+L        → App::clear_tool_panel()
```

### Input Queuing

- `queued_message: Option<String>` on App state
- When a tool is running and user presses Enter, store in `queued_message`
- Show visual indicator: `> _ [1 queued]`
- New queued message overwrites previous
- When tool finishes, inject queued message into the conversation flow

### End-Session Prompt

- Overlay centred dialog: `End session? Conversation and findings will be saved. [Y/n]`
- Y or Enter → clean exit
- N or Esc → dismiss overlay
- Technically a separate state or a modal flag in App

---

## Order of Implementation

1. `src/tui/app.rs` — state machine, App struct with all state fields
2. `src/tui/layout.rs` — Constraint-based layout function
3. `src/tui/input.rs` — Crossterm event loop, keybinding dispatch
4. `src/tui/mod.rs` — `run_tui()` function that wires it together
5. Modify `src/main.rs` — replace stub with `tui::run().await`
6. Test manually: launch, verify layout, test all keybindings
