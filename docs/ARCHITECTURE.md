# Architecture

## Overview

Raskolnikov is a Rust binary that runs a Ratatui TUI loop. The TUI dispatches
user messages to an AI provider, the AI response is parsed for tool suggestions,
and approved tools are executed via tokio subprocess with interrupt support.
Every event is logged to a session directory.

## Data flow

```
User input (Enter)
  → App.submit_message()
    → Message::user() appended to history
    → processing = true

Main loop (every 100ms):
  if processing → App.process_ai()
    → builds system prompt + message history
    → sends to AI provider via HTTP
    → parses response for "— run this?" + code block
    → if tool suggested: App.state = AwaitingConfirm
    → if not: displays response, waits for next input

User types "yes" in AwaitingConfirm:
  → App.spawn_tool(command)
    → creates interrupt watch channel
    → spawns tokio task: executor::run_tool()
    → App.state = ToolRunning

Tool finishes:
  → App.check_tool_completion() via oneshot::Receiver
    → displays stdout/stderr in tool_output panel
    → sends result back as Message::tool() to message history
    → processing = true (AI interprets results)

Session saved on quit:
  → Transcript::write() → conversation.md
  → FindingsExport::write() → findings.md
  → logger.session_end()
```

## Module layout

```
src/
  main.rs          CLI dispatch, config load, TUI launch
  lib.rs           Top-level pub mod declarations
  config.rs        Config struct, TOML load/save, API keys, data dirs
  ai/
    mod.rs         Provider trait, Message types, resolve_provider()
    anthropic.rs   Anthropic Claude API
    ollama.rs      Ollama local API
    openai.rs      OpenAI-compatible API (reused by Groq, Together, etc.)
    openrouter.rs  OpenRouter API
    nous.rs        Nous Research API
  tui/
    mod.rs         Startup: tool checks, provider resolve, session init
    app.rs         App state machine, run loop, AI processing, tool execution
    layout.rs      Ratatui render: header, tool panel, conversation, input bar
    input.rs       Standalone key handler (legacy)
  tools/
    mod.rs         ToolInfo, check_tool(), check_all_tools()
    executor.rs    run_tool() via tokio::process with interrupt watch
    nmap.rs        Nmap XML parser + command builder
    gobuster.rs    Gobuster/ffuf JSON parser
    nikto.rs       Nikto output parser
    sqlmap.rs      Sqlmap output parser
  session/
    logger.rs      JSON-lines session.log writer
    transcript.rs  Markdown conversation.md writer
    findings.rs    Markdown findings.md writer
  agent/
    shell.rs       AgentShell: context + prompt builder wrapper
    context.rs     EngagementContext: ports, paths, findings, targets
    prompt.rs      PromptBuilder: master prompt template construction
```

## State machine

```
Idle ──user input──→ processing ──AI response──→ Idle
                                           └──tool suggested──→ AwaitingConfirm
                                                                   │
                                                          yes ──→ ToolRunning
                                                           no ──→ Idle

ToolRunning ──finishes──→ processing (AI interprets)
           └─Ctrl+C──→ Interrupted ──any key──→ Idle

Idle ──Ctrl+C──→ ConfirmQuit ──Y──→ exit
                            └─N──→ Idle
```

## Key design decisions

- **No re-exports** — modules reference each other by full path
  (`crate::tools::nmap::NmapPort`). Keeps dependencies explicit.
- **`event::poll()` with 100ms timeout** — allows interleaving key events with
  async AI processing and tool completion checks.
- **`oneshot` channel for tool results** — tool runs in a spawned tokio task;
  completion signal delivered via `oneshot::Receiver::try_recv()`.
- **`watch` channel for interrupt** — `Ctrl+C` sends `true` via watch sender;
  executor's `select!` picks up the change and kills the child.
- **Session per invocation** — each `rsk` run creates a new
  `YYYY-MM-DDTHH-MM-SS` session directory under `sessions/`.
