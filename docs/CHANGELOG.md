# Changelog

## [Unreleased]

### Fixed

- **Conversation scrolling** — now line-based (wrapped line counts) instead of
  element-based, so long multi-line agent responses are no longer clipped and
  `PgUp` / `PgDn` / mouse-wheel scroll correctly from the bottom.
- **Text selection / copy** — new `/mouse` command and `ui.mouse` config option
  to toggle mouse capture; with it off, text in the conversation can be
  selected and copied with the mouse (keyboard scrolling still works).
- **AI streaming cut off mid-response** — the shared HTTP client used a 30s
  *total* request timeout, so long local-model streams (and requests queued
  behind a busy Ollama model) died with `error decoding response body`. Now a
  per-read timeout (`network.timeout_secs`, default 60) is used instead, so
  streams run for as long as the model keeps producing tokens.

## [0.1.0-alpha] — 2025-06-17

### Added

- **Agent shell** — conversational interface with natural-language target
  description, AI reasoning, tool suggestion, and operator approval flow.
- **TUI** — Ratatui-based three-panel layout: tool output, conversation,
  findings bar. Keyboard-driven (Tab focus, PageUp/Down scroll, Ctrl+C
  interrupt, Ctrl+L clear).
- **AI providers** — Ollama (local), Anthropic Claude, OpenAI, Groq,
  OpenRouter, Nous, Llama API, Together. Pluggable via `#[async_trait]`
  `Provider` trait. API keys from environment variables only.
- **Tool integrations** — nmap XML parsing, gobuster/ffuf JSON parsing,
  nikto output parsing, sqlmap output parsing. Command builder for nmap.
- **Session logging** — JSON-lines `session.log` with structured events
  (session_start, operator, agent, tool_start, tool_end, session_end).
  Markdown transcript (`conversation.md`) and findings (`findings.md`).
- **CLI** — `rsk sessions {list,show,findings,log,prune}`,
  `rsk config {show,provider,model,set}`, `rsk tools`.
- **Config** — TOML-based with `#[serde(default)]`, env var overrides,
  `$RASKOLNIKOV_CONFIG` / `$RASKOLNIKOV_DATA` paths. Example config and
  `.env.example` included.
- **Prompt builder** — master prompt with engagement context injection,
  available tools list, behaviour rules.
- **Engagement context** — discovered ports, web paths, findings with
  deduplication.
- **Interrupt handling** — Ctrl+C during tool execution sends SIGKILL
  via tokio watch channel; session state preserved.
- **Agent configuration** — `CLAUDE.md`, `.cursorrules`, `opencode.json`,
  `.github/copilot-instructions.md` — agent rules for AI coding tools.

### Changed

- (none — initial release)

### Fixed

- (none — initial release)

### Security

- API keys never logged or stored in config files.
- Session files created with restricted permissions (0600).
- Shell arguments passed via `Command::arg()` — no shell injection vector.
- No telemetry, no phone-home, no third-party requests without user consent.
