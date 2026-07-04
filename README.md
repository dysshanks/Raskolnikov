# Raskolnikov

Terminal-native, markdown-driven AI security operating environment for authorised
penetration testing and CTF competitions.

You run `rsk`. A persistent agent shell opens. You talk to it in plain English.
It reasons, plans, runs tools (nmap, gobuster, nikto, sqlmap), interprets output,
and responds — whether you approve a step, change direction, or ask a question
mid-session.

**Status: Alpha 0.1.0 — experimental, APIs and architecture may change.**

---

## Features

- **Conversational agent shell** — describe your target in natural language, the
  agent plans and suggests tool commands, you approve or redirect.
- **8 AI providers** — Ollama (local), Anthropic, OpenAI, Groq, OpenRouter, Nous,
  Llama API, Together. API keys from environment variables only.
- **Tool integrations** — nmap, gobuster/ffuf, nikto, sqlmap parsing with
  structured output extraction.
- **Ratatui TUI** — three-panel layout (tool output, conversation, findings),
  keyboard-driven, no mouse required.
- **Session logging** — every session is written as JSON-lines (`session.log`)
  plus markdown transcript (`conversation.md`) and findings (`findings.md`).
- **Configurable** — TOML-based config at `~/.config/raskolnikov/config.toml`,
  overridable via CLI flags.
- **Interrupt-safe** — Ctrl+C during tool execution sends SIGKILL via tokio
  watch channel; clean session save on quit.
- **No telemetry, no phone-home, no web UI.**

## Installation

### From source

```bash
git clone <repo-url>
cd raskolnikov
cargo build --release
sudo cp target/release/raskolnikov /usr/local/bin/
sudo ln -s raskolnikov /usr/local/bin/rsk
sudo ln -s raskolnikov /usr/local/bin/rk
```

Dependencies at build time: Rust 1.80+ with `rustfmt` and `clippy` components.
Runtime dependencies: `nmap`, `gobuster` (or `ffuf`), `nikto`, `sqlmap` — each
tool is optional; missing tools are reported at startup.

## Quick start

```bash
# Configure your AI provider (defaults to Ollama at localhost:11434)
export ANTHROPIC_API_KEY=sk-ant-...

# Or use a local model via Ollama
# (no env var needed; just ensure ollama is running)

# Start the agent shell
rsk
```

Type a target description, for example:

```
scan 10.10.10.10
```

The agent will reason, suggest an nmap command, and wait for approval before
running it. Type `yes` to proceed, or ask questions / change direction.

### Key bindings

| Key | Action |
|---|---|
| Type + Enter | Send message to agent |
| Tab | Switch panel focus (tool output / conversation) |
| PageUp / PageDown | Scroll focused panel |
| Ctrl+C | Interrupt running tool / confirm quit |
| Ctrl+L | Clear tool output panel |
| Esc | Clear input |
| `/quit` | End session and save |

## CLI reference

```
raskolnikov [--model <model>] [--provider <provider>] [--version]
            [<command>]

Commands:
  sessions  List and manage past sessions
  config    View or modify configuration
  tools     Check tool availability and versions

Sessions subcommands:
  list                  List all sessions
  show <id>             Show conversation transcript
  findings <id>         Show findings summary
  log <id>              Dump raw JSON session log
  prune [--keep <N>]    Remove sessions older than N days (default: 30)

Config subcommands:
  show                  Show current configuration
  provider <name>       Set AI provider
  model <name>          Set default model
  set <key> <value>     Set an arbitrary config key

Config keys for `set`:
  provider, model, ollama_host, nmap_timing, prefer_ffuf, sqlmap_level,
  sqlmap_risk, stream_output, proxy, proxy_https
```

## Configuration

Config file location (first found wins):

1. `$RASKOLNIKOV_CONFIG`
2. `~/.config/raskolnikov/config.toml`
3. `/etc/raskolnikov/config.toml`

Example:

```toml
[ai]
provider = "ollama"
model = "qwen3"

[ollama]
host = "http://localhost:11434"

[tools]
nmap_timing = 4
prefer_ffuf = false
```

API keys are read from environment variables only — see `.env.example`.

Session data is stored at `$RASKOLNIKOV_DATA` (default:
`~/.local/share/raskolnikov/`).

## Project structure

```
src/
  main.rs            Entrypoint — CLI dispatch and TUI launch
  lib.rs             Top-level module declarations
  config.rs          TOML config, API keys, data dirs
  ai/                AI provider implementations (Ollama, Anthropic, OpenAI, …)
  tui/               Ratatui TUI — App, layout, input handling
  tools/             Tool integrations and executor
  session/           Logger, transcript, findings export
  agent/             Agent shell, engagement context, prompt builder
```

See `docs/spec-mvp.md` for the full specification.

## License

Apache 2.0.

## Disclaimer

This tool is for authorised security testing and CTF competitions only. Users
are solely responsible for compliance with all applicable laws. The developers
assume no liability for misuse.
