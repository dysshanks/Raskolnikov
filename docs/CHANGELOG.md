# Changelog

## [Unreleased]

### Added

- **9 more tool integrations** — ffuf (detection; parser already existed),
  smbclient, smbmap, wpscan, masscan, nbtscan, dnsx, subfinder, and impacket
  (GetNPUsers / GetUserSPNs / secretsdump). Each with a best-effort output
  parser feeding engagement context.
- **Discovered targets** — DNS/subdomain/host discovery results from dnsx,
  subfinder, nbtscan and masscan now surface in the `=== DISCOVERED TARGETS ===`
  prompt section and a `## Discovered Targets` section in `findings.md`.
- **9 new tool integrations** — hydra, whatweb, feroxbuster, netexec, john,
  nuclei, enum4linux-ng, httpx, and hashcat. Each with a structured output
  parser, registered in the startup tool check.
- **Credential findings** — `EngagementContext` now tracks credentials
  (service, host, user, password) parsed from hydra/netexec/john/hashcat
  output; exported as a `## Credentials` table in `findings.md`.
- **Central output parsing** — `src/tools/parse.rs` dispatches tool output by
  name into engagement context and returns tags for the findings bar.
- **Output sanitization** — `src/tools/sanitize.rs` strips ANSI CSI/OSC
  sequences and C0 control characters from tool output at the TUI boundary.
- **Unified error type** — `src/error.rs` with a `thiserror` enum; config uses
  `Result<T, crate::error::Error>`.
- **Gated `/update`** — self-update only runs from a git checkout.
- **Split documentation** — `docs/` reorganised into focused guides (USAGE,
  INSTALLATION, CONFIGURATION, PROVIDERS, SESSIONS, TOOLS, CLI, ARCHITECTURE,
  ROADMAP, SECURITY) with an index. Removed `spec-mvp.md` and `docs/plan/`.
- **Packaging** — Docker image, AUR PKGBUILD, and `.deb` control now declare
  the additional core tools.

### Changed

- **Command execution** — tool commands are tokenized with `shell-words`
  (quoting-aware) and empty commands are rejected; processes run in their own
  process group for reliable Ctrl+C interruption.
- **Nuclei CVE tags** are normalised to uppercase.
- **Tracing** is enabled via `RASKOLNIKOV_LOG` (RUST_LOG-style filter) instead
  of always-on file logging.
- **Module layout** — `src/tools/` grows `parse.rs` and `sanitize.rs`; the
  modules remain flat and match `docs/ARCHITECTURE.md`.

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
