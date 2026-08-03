# Architecture

## Overview

Raskolnikov is a Rust binary that runs a Ratatui TUI loop. The TUI dispatches
user messages to an AI provider, the AI response is parsed for tool
suggestions, and approved tools are executed via tokio subprocess with
process-group interrupt support. Every event is logged to a session directory.

## Data flow

```
User input (Enter)
  → App.submit_message()
    → Message::user() appended to history
    → processing = true

Main loop (event::poll() with 50ms timeout):
  if processing → App.process_ai()
    → builds system prompt + message history
    → sends to AI provider via HTTP (streaming)
    → parses response for "— run this?" + code block
    → if tool suggested: App.state = AwaitingConfirm
    → if not: displays response, waits for next input

User types "yes" in AwaitingConfirm:
  → App.spawn_tool(command)
    → shell-words tokenization, empty-command guard
    → creates interrupt watch channel
    → spawns tokio task: executor::run_tool() in a new process group
    → App.state = ToolRunning

Tool finishes:
  → App.check_tool_completion() via oneshot::Receiver
    → displays stdout/stderr in tool_output panel (sanitized)
    → parse_tool_output() extracts ports/paths/credentials into context
    → sends result back as Message::tool() to message history
    → processing = true (AI interprets results)

Session saved on quit:
  → Transcript::write() → conversation.md
  → FindingsExport::write() → findings.md  (ports, paths, credentials)
  → logger.session_end()
```

## Module layout

```
src/
  main.rs          CLI dispatch (clap), config load, sessions/config/tools
                   subcommands, tracing init, TUI launch
  lib.rs           Top-level pub mod declarations
  config.rs        Config struct, TOML load/save, defaults, data dirs
  error.rs         thiserror Error enum; config::Result<T> alias
  ai/
    mod.rs         Provider trait, Message types, resolve_provider(),
                   summarise_context()
    anthropic.rs   Anthropic Claude API
    ollama.rs      Ollama local API
    openai.rs      OpenAI-compatible API (reused by Groq, Together, Llama API)
    openrouter.rs  OpenRouter API
    nous.rs        Nous Research API
  agent/
    shell.rs       AgentShell: system prompt + tool context assembly
    context.rs     EngagementContext: ports, paths, findings, credentials,
                   add_finding/add_port/add_credential with dedup
    prompt.rs      PromptBuilder: master prompt template construction
  tools/
    mod.rs         ToolInfo, check_tool(), check_all_tools(), parse module
    executor.rs    run_tool() via tokio::process with process-group kill
    parse.rs       parse_tool_output() — central dispatch by tool name
    sanitize.rs    ANSI/C0 control-character sanitizer for TUI output
    nmap.rs        Nmap XML parser + command builder
    gobuster.rs    Gobuster/ffuf parser + command builder + wordlist lookup
    nikto.rs       Nikto output parser
    sqlmap.rs      Sqlmap output parser + command builder
    hydra.rs       Hydra parser (login/password extraction)
    whatweb.rs     Whatweb technology parser
    feroxbuster.rs Feroxbuster JSONL parser
    netexec.rs     Netexec credential parser (nxc alias)
    john.rs        John the Ripper --show parser
    nuclei.rs      Nuclei template/CVE parser
    enum4linux.rs  Enum4linux-ng users/shares parser
    httpx.rs       Httpx probe parser
    hashcat.rs     Hashcat --show parser
  session/
    logger.rs      JSON-lines session.log writer
    transcript.rs  Markdown conversation.md writer
    findings.rs    Markdown findings.md writer (ports, paths, credentials)
    recover.rs     Rebuild conversation.md + findings.md from session.log
  tui/
    mod.rs         Startup: tool checks, provider resolve, first-launch
                   provider/model prompts, session init
    app.rs         App state machine, run loop, AI processing, tool execution,
                   slash commands, findings bar
    layout.rs      Ratatui render: header, tool panel, conversation, input bar
    input_handler.rs  Key handling, command fuzzy-match/completion, history
    tool_handler.rs   spawn_tool, check_tool_completion, tool output
```

## State machine

```
Idle ──user input──→ processing ──AI response──→ Idle
                                            └──tool suggested──→ AwaitingConfirm
                                                                   │
                                                          yes ──→ ToolRunning
                                                           no ──→ Idle

ToolRunning ──finishes──→ processing (AI interprets)
           └─Ctrl+C──→ Interrupted ──Enter──→ Idle

Idle ──Ctrl+C──→ ConfirmQuit ──Y──→ exit
                            └─N──→ Idle

Idle ──/update──→ Updating (git pull)
```

## Key design decisions

- **No re-exports** — modules reference each other by full path
  (`crate::tools::nmap::NmapPort`). Keeps dependencies explicit.
- **`event::poll()` with 50ms timeout** — allows interleaving key events with
  async AI processing and tool completion checks.
- **`oneshot` channel for tool results** — tool runs in a spawned tokio task;
  completion signal delivered via `oneshot::Receiver::try_recv()`.
- **`watch` channel for interrupt** — `Ctrl+C` sends `true` via a watch sender;
  the executor's `select!` picks it up and terminates the child process group.
- **Process groups** — each tool spawns with `process_group(0)`, so Ctrl+C
  (SIGTERM to the group) kills child processes too; SIGKILL after 5s if
  ignored.
- **`shell-words` tokenization** — user/AI-supplied command strings are split
  into argv respecting quoting before `Command::args()`, preventing shell
  injection; empty commands are rejected.
- **Central output parsing** — `parse_tool_output(tool, output, context)`
  dispatches by tool name and routes parsed findings into `EngagementContext`,
  returning tags for the findings bar.
- **Sanitization at the TUI boundary only** — tool output is stripped of
  ANSI CSI/OSC sequences and C0 control characters before rendering in the
  terminal. Raw bytes on disk (conversation.md, session.log) stay untouched.
- **Optional tools** — every tool is checked at startup; missing tools show
  `✗` and are skipped by the agent, but never block startup.
- **Session per invocation** — each `rsk` run creates a new
  `YYYY-MM-DDTHH-MM-SS` session directory under `sessions/`.
- **Config as `#[serde(default)]` structs** — every config field has a
  standalone `fn default_*()`; merge order is built-in defaults → config.toml
  → env vars → CLI flags.
- **Error handling** — `thiserror` enums in `src/error.rs`, aliased as
  `config::Result<T>`; production code never uses `unwrap()`.
