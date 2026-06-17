# Raskolnikov — Implementation Plan

**Version:** Alpha 0.1.0
**Total Phases:** 8

---

## Phase Overview

| Phase | Name | Depends On | Theme |
|-------|------|------------|-------|
| 1 | Scaffold & Configuration | — | Project skeleton, toolchain, CI, config |
| 2 | TUI Skeleton | 1 | Ratatui 3-panel layout, event loop, keybindings |
| 3 | AI Provider System | 1 | 8 providers, Provider trait, context management |
| 4 | Tool System | 1, 2 | 4 tools, executor, output parsing, wordlists |
| 5 | Agent Shell | 2, 3, 4 | Conversation loop, system prompt, confirmation flow |
| 6 | Session Logging & Export | 1, 5 | JSON log, transcripts, findings, `rsk sessions` |
| 7 | Error Handling & Security | 2, 3, 4, 5 | Unified errors, sanitisation, proxy, key safety |
| 8 | Polish & Packaging | 1–7 | README, man page, AUR, .deb, completions, spec audit |

---

## Dependency Graph

```
Phase 1 ──┬── Phase 2 ──┐
          ├── Phase 3 ──┤
          ├── Phase 4 ──┤
          └── Phase 6 ──┤
                        ▼
                    Phase 5 ──┐
                              ├── Phase 7
                              └── Phase 8
```

Phase 5 (Agent Shell) is the integration point — it depends on TUI, AI providers, and tools all being functional. Phases 2, 3, 4 can be built in parallel after Phase 1.

---

## Estimated Effort

| Phase | Files Created | Files Modified | User Stories | Acceptance Criteria |
|-------|---------------|----------------|--------------|-------------------|
| 1 | ~18 | 0 | 6 | 10 |
| 2 | 4 | 1 | 10 | 17 |
| 3 | 6 | 1 | 7 | 10 |
| 4 | 6 | 0 | 7 | 14 |
| 5 | 3 | 1 | 10 | 13 |
| 6 | 3 | 1 | 6 | 14 |
| 7 | 1 | 4 | 5 | 13 |
| 8 | 7 | 1 | 6 | 15 |
| **Total** | **~48** | **~9** | **57** | **106** |

---

## Key Architectural Decisions

1. **No SDK dependencies for AI providers** — raw `reqwest` HTTP for all providers
2. **Three binary names, one source** — Cargo `[[bin]]` entries, runtime detection via `current_exe()`
3. **All OpenAI-compatible providers share one module** — parameterised by base_url + API key
4. **Tools are trait objects** — registered at startup, iterated for availability check
5. **Session logs flushed per event** — crash recovery via partial log replay
6. **API keys from env only** — never in config, logs, or context
7. **Testing with mocks** — no real tools or AI providers in unit tests

---

## Phase Files

| File | Content |
|------|---------|
| [phase-1-scaffold.md](phase-1-scaffold.md) | Project skeleton, config, CI/CD, toolchain |
| [phase-2-tui.md](phase-2-tui.md) | Ratatui layout, event loop, keybindings |
| [phase-3-ai-providers.md](phase-3-ai-providers.md) | Provider trait, 8 providers, context management |
| [phase-4-tools.md](phase-4-tools.md) | Tool trait, executor, nmap/gobuster/nikto/sqlmap |
| [phase-5-agent-shell.md](phase-5-agent-shell.md) | Conversation loop, system prompt, confirmation flow |
| [phase-6-session-logging.md](phase-6-session-logging.md) | JSON log, transcripts, findings, session CLI |
| [phase-7-error-handling-security.md](phase-7-error-handling-security.md) | Error types, sanitisation, proxy, key safety |
| [phase-8-polish-packaging.md](phase-8-polish-packaging.md) | Docs, packaging, completions, spec audit |

---

## Quick Start (for implementers)

```bash
# Phase 1
git init
cargo init --name raskolnikov
# Add all files from phase-1-scaffold.md
cargo build && cargo test && cargo clippy && cargo fmt --check

# Phase 2-4 (parallel after Phase 1)
# Build TUI, AI providers, and tools independently

# Phase 5
# Wire everything together in the agent shell

# Phase 6-8
# Add logging, error handling, polish
```
