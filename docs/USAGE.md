# Usage

`raskolnikov`, `rsk`, or `rk` with no arguments opens the agent shell. That is the
entire entry point — no subcommands required.

```bash
rsk
rsk --model deepseek-r1
rsk --provider anthropic
```

## Startup

On launch Raskolnikov:

1. Checks available security tools and prints `✓`/`✗` per tool.
2. Detects an AI provider (Ollama first, then configured API keys).
3. Creates a session directory under `sessions/`.
4. Opens the TUI.

The first launch walks you through provider and model selection.

## Interaction model

### Starting is just talking

Any input is valid. The agent extracts intent, target, and constraints:

```
> scan 10.0.0.1
> enumerate target.htb, start with ports then web
> I've already found 80 and 443 open, skip the port scan and enumerate web
> do a full engagement on 10.0.0.5, HackTheBox machine, goal is root
```

### The agent proposes, you approve

Before every tool execution the agent states what it intends to run and waits for
approval. Proposed commands end with "— run this?".

```
> yes
> go ahead
> skip nikto, go straight to gobuster
> wait — what wordlist are you using?
```

Operator approval is always required. There is no auto-execution mode.

### Redirection at any time

You can change direction mid-engagement. The full conversation history and all
findings remain in context.

### Asking questions mid-session

```
> what have we found so far?
> what flags is sqlmap going to use?
```

## TUI layout

The interface has three panels:

- **Tool output (left)** — live stdout/stderr streams while a tool runs.
- **Conversation (right)** — full chat history between you and the agent.
- **Findings bar (bottom)** — persistent strip of confirmed findings: ports,
  web paths, credentials, flags. Updates after each tool run.
- **Input bar** — single line, always focused when no tool is running.

## Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `PgUp` / `PgDn` | Scroll conversation |
| `Ctrl+Up` / `Ctrl+Down` | Scroll conversation (5 lines) |
| `Up` / `Down` | Command history (at the input) |
| `Tab` | Complete `/` command |
| `Ctrl+C` | Interrupt running tool, or prompt to end session at idle |
| `Ctrl+L` | Clear the conversation panel |
| `Ctrl+O` | Toggle the tool-output "island" |
| `Esc` | Clear input / close command menu |

## Slash commands

| Command | Action |
|---------|--------|
| `/findings <tag>` | Show findings, optionally filtered by tag |
| `/island` | Toggle the tool-output island |
| `/mouse` | Toggle mouse capture (enables terminal text selection) |
| `/quit` | End the session |
| `/update` | Pull the latest version from the git checkout |
| `/update tools` | Update the installed security tools |

Commands can be fuzzy-matched: type `/qu`, `/fin`, etc., and use `Up`/`Down` +
`Tab` to navigate and complete.

### Interrupting a running tool

`Ctrl+C` while a tool runs sends SIGTERM to the child process group; if the
process does not exit within 5 seconds, SIGKILL is sent. Partial output up to
the interrupt point is still saved to disk and logged.

`/quit` typed while a tool runs is queued and processed only after the current
tool exits or is interrupted.

## Session end

`Ctrl+C` at the idle prompt asks to confirm. Conversation and findings are
saved to the session directory on exit.
