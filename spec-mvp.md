# Raskolnikov — MVP Specification

**Version: Alpha 0.1.0**
**Status: Experimental — APIs, commands, and architecture may change without notice**

-----

## Table of Contents

1. [Overview](#overview)
1. [Philosophy](#philosophy)
1. [Core Concept](#core-concept)
1. [The Agent Shell](#the-agent-shell)
1. [Interaction Model](#interaction-model)
1. [Tool Integrations](#tool-integrations)
1. [Error Handling](#error-handling)
1. [AI Provider System](#ai-provider-system)
1. [TUI Design](#tui-design)
1. [Session & Logging](#session--logging)
1. [Installation](#installation)
1. [First Launch](#first-launch)
1. [CLI Reference](#cli-reference)
1. [Configuration](#configuration)
1. [Security Considerations](#security-considerations)
1. [Technology Stack](#technology-stack)
1. [Project Structure](#project-structure)
1. [Alpha Scope & Roadmap](#alpha-scope--roadmap)
1. [Future Systems](#future-systems)
1. [License](#license)

-----

## Overview

Raskolnikov is a terminal-native, markdown-driven AI security operating environment.

You run `rsk`. A persistent agent shell opens. You talk to it in plain English. It
reasons, plans, runs tools, interprets output, and responds to whatever you say next —
whether that’s approving a step, changing direction, or asking a question mid-session.

It is not a chatbot. It is not a scanner wrapper. It is not another AI CLI.

It is a security agent operating system you have a conversation with.

**Target environments:** CTF labs (HackTheBox, TryHackMe) and real-world penetration testing.

**Target users:** Security professionals, students, and researchers who live in the terminal.

-----

## Philosophy

### Terminal-Only

Raskolnikov is a terminal system. Permanent constraint, not a temporary limitation.

- Ratatui TUI is the only UI layer — ever
- No web UI, no browser dashboard, no GUI, no Electron wrapper
- No roadmap items pointing toward any graphical interface

### Open Source First

- Repository: `<repo-url>`
- Apache-2.0 license
- No proprietary core, no cloud-gated features, no paywalls
- All design decisions documented and open for community discussion

### Linux First

**Primary (fully supported, CI-tested):**

- Arch Linux *(primary development target)*
- Kali Linux
- BlackArch
- Ubuntu 22.04+
- Debian 12+
- Parrot OS

**Secondary (best-effort):** macOS
**Not supported:** Windows — not planned, not on the roadmap

### Local AI First

- No API key required by default
- No account required
- No cloud dependency
- No telemetry of any kind
- Works fully air-gapped with a local Ollama model

Cloud providers are opt-in, never required.

### Markdown-First Intelligence

All human and AI knowledge artifacts in Raskolnikov are Markdown. This is a design
rule, not a convention.

|Type                |Format         |
|--------------------|---------------|
|Agent definitions   |Markdown       |
|Skill playbooks     |Markdown       |
|Team configurations |Markdown       |
|Session transcripts |Markdown       |
|Findings exports    |Markdown       |
|System configuration|TOML only      |
|Automation workflows|YAML only      |
|Session event logs  |JSON lines only|

Markdown is human-readable, human-editable, git-diffable, and shareable as plain text.
No special syntax. No tooling required to contribute an agent or skill.

### Operator Control

Raskolnikov never executes a tool without explicit operator approval in default mode.
The agent proposes; the human decides. This default is non-negotiable.

-----

## Disclaimer

Raskolnikov is a tool for authorised security testing. Users are solely
responsible for ensuring their use complies with all applicable laws.
The developers assume no liability for misuse.

-----

## Core Concept

### What this replaces

```
CLI tool with AI assistance
  └── single agent
  └── tool execution wrapper
  └── one engagement at a time
  └── no persistent memory
  └── no reusable knowledge
```

### Raskolnikov

```
Terminal-native AI operating environment
  └── markdown-native knowledge ecosystem
      └── agents   — specialist AI personas        (0.2.0)
      └── skills   — reusable playbooks            (0.2.0)
      └── teams    — multi-agent configurations    (0.3.0)
      └── workflows — declarative automation       (0.3.0)
  └── persistent engagement memory                 (0.2.0)
  └── evolving capability
      └── AI-generated agents and skills           (0.2.0)
      └── community marketplace                    (0.4.0)
```

The shift is from a procedural security CLI to a knowledge-driven agent operating system
that accumulates capability over time. Alpha 0.1.0 lays the foundation.

-----

## The Agent Shell

`raskolnikov`, `rsk`, or `rk` with no arguments opens the shell. That is the entire
entry point. No subcommands required.

```bash
rsk
rsk --model deepseek-r1
rsk --provider anthropic
```

```
$ rsk

Checking tools...
  ✓ nmap 7.94  ✓ gobuster 3.6.0  ✓ nikto 2.1.6  ✓ sqlmap 1.7.8

Checking AI providers...
  ✓ Ollama — qwen3

┌──────────────────────────────────────────────────────────────────────┐
│  RASKOLNIKOV  alpha 0.1.0                          model: qwen3      │
├─────────────────────────────┬────────────────────────────────────────┤
│                             │                                        │
│  TOOL OUTPUT                │  CONVERSATION                          │
│                             │                                        │
│                             │  Ready. Type anything.                 │
│                             │                                        │
├─────────────────────────────┴────────────────────────────────────────┤
│  FINDINGS  ·  none yet                                               │
├──────────────────────────────────────────────────────────────────────┤
│  > _                                                                 │
└──────────────────────────────────────────────────────────────────────┘
```

A custom CLI alias can be set in config:

```toml
[cli]
alias = "ghost"   # now 'ghost' launches raskolnikov
```

-----

## Interaction Model

### Starting is just talking

Any input is valid. Vague, specific, short, or long.

```
> scan 10.0.0.1
> enumerate target.htb, start with ports then web
> I've already found 80 and 443 open, skip the port scan and enumerate web
> check this box for SQLi, the login form at /login looked suspicious
> do a full engagement on 10.0.0.5, HackTheBox machine, goal is root
```

The agent extracts intent, target, and constraints from whatever was written. If it
genuinely cannot proceed, it asks one clarifying question — not a form.

### The agent responds in natural language

```
agent  nmap found three open ports: SSH on 22, Apache on 80, and MySQL on 3306.
       MySQL being directly exposed is unusual — often indicates misconfiguration.

       I want to run nikto against port 80 to check for known web vulnerabilities,
       then enumerate directories with gobuster. After that we can look at whether
       MySQL accepts unauthenticated connections.

       nikto -h http://10.0.0.1 — run this?
```

### Confirming is just responding

```
> yes
> go ahead
> skip nikto, go straight to gobuster
> wait — what wordlist are you using?
> stop, I want to try something manually first
```

No magic keywords. Plain English throughout.

### Redirection at any point

The operator can change direction mid-engagement at any time. The full conversation
history and all findings remain in context.

```
agent  gobuster found /admin (302) and /uploads (200). Want me to investigate further?

> forget the web stuff for now, check if MySQL on 3306 accepts unauthenticated connections

agent  Sure. I'll use nmap's mysql-empty-password script.
       nmap -p 3306 --script mysql-empty-password 10.0.0.1 — run this?
```

### Asking questions mid-session

```
> what have we found so far?

agent  Ports: 22 (SSH/OpenSSH 8.9), 80 (HTTP/Apache 2.4.52), 3306 (MySQL 8.0.33)
       Web paths: /admin (302 → /admin/login), /uploads (200, listing enabled)
       Flags: MySQL exposed to network, /uploads world-readable
       Not yet tested: SQLi, Apache CVEs, MySQL auth

> what flags is sqlmap going to use?

agent  --crawl=2 to discover forms, --batch so it never prompts, --level=2 and
       --risk=1 to stay thorough without being aggressive. Want me to adjust anything?
```

### Operator always has final say

Before every tool execution, the agent states what it intends to run and waits for
approval. This cannot be bypassed in 0.1.0.

### Session end

`ctrl+c` or `/quit` at the idle prompt prompts:

```
End session? Conversation and findings will be saved. [Y/n]
```

### Interrupting a running tool

`ctrl+c` while a tool is running immediately sends SIGTERM to the
child process. If the process does not exit within 5 seconds, SIGKILL
is sent. The tool's partial output up to the interrupt point is still
saved to disk and logged.

`/quit` typed in the input bar while a tool is running is queued and
processed only after the current tool exits or is interrupted.

-----

## Agent Prompt

The master prompt defines Raskolnikov's identity, rules, and available tools.
It is injected as the system message on every conversation turn. Its structure
is fixed for 0.1.0 and will become agent-customisable in 0.2.0.

### Structure

```
You are Raskolnikov, a security agent running in a terminal.
You are assisting a security operator with penetration testing.

=== AVAILABLE TOOLS ===
{tool_names}

=== RULES ===
1. Always explain your reasoning before proposing a tool.
2. Always state the exact command you want to run in a code block.
3. Never execute a tool without operator approval.
4. Wait for "yes" or "go ahead" before proceeding.
5. If the operator says "no" or changes direction, adapt.
6. If you want to run a tool, end your message with " — run this?"

=== CURRENT CONTEXT ===
{ports, findings, session state}

Respond naturally. Be concise but informative.
```

### Behaviour rules

|Rule|Rationale|
|----|---------|
|Explain before acting|Operator must understand intent before approving|
|Exact command in code block|Operator reviews flags and arguments before approval|
|No auto-execution|Operator approval is always required in 0.1.0|
|End tool proposals with " — run this?"|Clear, parseable signal that approval is needed|
|Adapt on redirection|Operator may change focus mid-engagement at any time|
|Keep responses concise|Operator is a terminal user — brevity is respect|

### Context injection

On each turn, the prompt is populated with:
- **Available tools** — list of tools detected on startup (e.g. `nmap, gobuster, nikto, sqlmap`)
- **Current findings** — ports, services, web paths, flags discovered so far
- **Session state** — empty until findings are added

The context section is regenerated every turn to reflect the latest
engagement state. All past conversation history (operator messages, agent
responses, tool output) is passed separately as the chat history — not
embedded in the system prompt.

### Design constraints

- **No tool-specific instructions in the prompt.** Tools are described
  by name only. The AI model is expected to know common security tools
  from its training data. Tool-specific flag documentation is a 0.2.0
  feature (Skills system).
- **No operator identity or credentials.** The prompt must never contain
  API keys, usernames, or session identifiers.
- **Single persona.** The agent is always "Raskolnikov" in 0.1.0. Agent
  switching arrives in 0.2.0.
- **Prompt length** must stay under 1k tokens in 0.1.0. The conversation
  history carries the weight of the engagement.

-----

## Tool Integrations

Alpha 0.1.0 supports exactly four tools. All must be installed on the operator's system —
Raskolnikov does not bundle them.

### Availability check

On every startup:

```
tools  ✓ nmap 7.94  ✓ gobuster 3.6.0  ✓ nikto 2.1.6  ✓ sqlmap 1.7.8
```

Missing tools generate a warning but do not prevent startup. The agent skips steps
requiring unavailable tools and notifies the operator.

-----

### nmap

**Purpose:** Port scanning, service detection, OS fingerprinting, NSE scripts
**Install:** `sudo apt install nmap` / `sudo pacman -S nmap`

**Alpha flags:**

```
-sV                   service version detection
-sC                   default NSE scripts
-T4                   aggressive timing (configurable)
-oX -                 XML output, parsed internally
-p-                   full port range (when operator requests thorough scan)
--script <name>       targeted NSE scripts against specific services
```

**Output:** XML parsed into structured port/service list. Raw XML saved to
`session/tools/nmap_output.xml`. Parsed data injected into conversation context.

**Example invocations the agent may use:**

```bash
nmap -sV -sC -T4 10.0.0.1
nmap -p- -sV -T3 10.0.0.1
nmap -p 3306 --script mysql-empty-password 10.0.0.1
nmap -p 80,443 --script http-headers 10.0.0.1
```

-----

### gobuster / ffuf

**Purpose:** Directory, file, and DNS brute-forcing
**Install:** `sudo apt install gobuster` / `go install github.com/ffuf/ffuf/v2@latest`

**Preference:** gobuster by default; falls back to ffuf if absent or if
`prefer_ffuf = true` in config.

**Wordlist resolution order:**

```
/usr/share/wordlists/dirbuster/directory-list-2.3-medium.txt
/usr/share/seclists/Discovery/Web-Content/common.txt
/usr/share/seclists/Discovery/Web-Content/raft-medium-words.txt
/usr/share/wordlists/dirb/common.txt
/usr/share/wordlists/dirb/big.txt
```

Falls back to a minimal built-in wordlist (500 entries) if none found. Agent warns
the operator and suggests installing SecLists.

**Alpha flags (gobuster):**

```
dir -u <url> -w <wordlist> -q -t 20
-x php,html,txt         added when PHP/web stack detected
-b 404,403              exclude status codes
```

**Alpha flags (ffuf):**

```
-u <url>/FUZZ -w <wordlist> -mc 200,301,302,403 -t 20 -of json -o -
```

**Output:** Status code + path per line. Paths returning 200/301/302/403 added to
engagement context. Admin-pattern paths (`/admin`, `/login`, `/dashboard`) flagged immediately.

-----

### nikto

**Purpose:** Web server fingerprinting and known vulnerability detection
**Install:** `sudo apt install nikto` / `sudo pacman -S nikto`

**Alpha flags:**

```
-h <host>
-p <port>             if non-standard port
-Format txt
-nointeractive
-Plugins @@DEFAULT
```

**Output:** Lines prefixed `+` extracted as findings. CVE references extracted where
present. Findings added to engagement context.

-----

### sqlmap

**Purpose:** SQL injection detection
**Install:** `sudo apt install sqlmap` / `pip install sqlmap`

**Alpha flags:**

```
-u <url>
--crawl=2             discover forms to depth 2
--batch               never prompt — Raskolnikov handles all confirmation
--level=2
--risk=1              conservative default
--output-dir          session tools directory
--forms               test forms on the page
--threads=4
```

`--batch` is always set. sqlmap never interrupts the session. Raskolnikov’s own
confirmation step gates execution before sqlmap runs at all.

**Output:** Injection point confirmations extracted (parameter, type, payload).
Confirmed injections added to findings with parameter and technique.

-----

### Future Tools (post-alpha)

**0.2.0**

- `hydra` — credential attacks (SSH, FTP, HTTP-form, MySQL, SMB)
- `enum4linux-ng` — SMB/Windows enumeration, auto-triggered on SMB detection
- `whatweb` — web technology fingerprinting
- `feroxbuster` — recursive directory enumeration

**0.3.0**

- `metasploit` — module-based exploitation via MSFRPC
- `john` / `hashcat` — offline hash cracking
- `crackmapexec` / `netexec` — Windows/AD enumeration and credential validation
- `kerbrute` — Kerberos user enumeration
- `impacket` — GetNPUsers, GetUserSPNs, secretsdump

**0.4.0**

- `nuclei` — template-based vulnerability scanning
- `dnsx` / `subfinder` — subdomain and DNS enumeration
- `burpsuite` (headless REST API) — web proxy and active scanner
- `ligolo-ng` / `chisel` — tunnelling and pivoting
- `feroxbuster` escalated — recursive with advanced filtering

**Under consideration (no version assigned)**

|Tool                     |Purpose                               |
|-------------------------|--------------------------------------|
|`arjun`                  |HTTP parameter discovery              |
|`ghauri`                 |Modern SQLi alternative               |
|`trufflehog` / `gitleaks`|Secrets scanning                      |
|`responder`              |LLMNR/NBT-NS poisoning (LAN)          |
|`evil-winrm`             |WinRM post-exploitation shell         |
|`bloodhound-ce`          |AD attack path visualisation          |
|`certipy`                |AD Certificate Services attacks       |
|`coercer`                |NTLM coercion                         |
|`httpx`                  |Fast HTTP probing at scale            |
|`waybackurls` / `gau`    |Historical URL OSINT                  |
|`gospider`               |Web crawler and endpoint discovery    |
|`semgrep`                |Static analysis (when source in scope)|

**Tool plugin system (0.4.0):** Custom tools added as a Markdown + TOML pair without
modifying core. Community-shareable as git repos.

-----

## Error Handling

|Scenario                          |Behaviour                                    |
|----------------------------------|---------------------------------------------|
|Tool not found at startup         |Warning logged, agent skips steps needing it |
|Tool crashes mid-execution        |Non-zero exit logged, partial output saved   |
|Tool runs longer than 30 minutes  |SIGTERM sent, SIGKILL after 5s if no response|
|AI provider unreachable           |Connection error displayed, retry once       |
|AI provider returns invalid JSON  |Log warning, request text-only fallback      |
|Session directory unwritable      |Error on startup, refuses to launch          |
|Config file corrupt or missing    |Use defaults, warn on first boot             |

-----

## AI Provider System

### Provider resolution order

On startup, Raskolnikov detects available providers in this order:

1. Ollama (local — checked via `http://localhost:11434`)
1. `ANTHROPIC_API_KEY`
1. `OPENAI_API_KEY`
1. `OPENROUTER_API_KEY`
1. `GROQ_API_KEY`
1. `NOUS_API_KEY`
1. `LLAMA_API_KEY`
1. `TOGETHER_API_KEY`

First detected becomes default unless overridden in `config.toml` or via `--provider`.

### Conversation history format

The full session is passed to the model on every turn as a standard chat history:

```
system      You are Raskolnikov, a security agent running in a terminal...
            [engagement context, available tools, current findings]

user        scan 10.0.0.1

assistant   I'll start with nmap to discover open ports...
            nmap -sV -sC -T4 10.0.0.1 — run this?

user        yes

tool        [nmap output]

assistant   Found three open ports: SSH 22, HTTP 80, MySQL 3306...
```

Every message, every tool result, every piece of reasoning stays in context for the
duration of the session.

-----

### Ollama (Default)

No account, no API key, no network required after model pull.

```bash
ollama pull qwen3
rsk config provider ollama
rsk config model qwen3
```

**Recommended models:**

|Model          |Pull                       |Context|Notes                                    |
|---------------|---------------------------|-------|-----------------------------------------|
|`qwen3`        |`ollama pull qwen3`        |32k    |**Recommended**                          |
|`nous-hermes3` |`ollama pull nous-hermes3` |128k   |Strong tool-call reasoning, large context|
|`nous-hermes2` |`ollama pull nous-hermes2` |8k     |Solid Hermes baseline, lightweight       |
|`qwen2.5-coder`|`ollama pull qwen2.5-coder`|32k    |Good for command/flag construction       |
|`deepseek-r1`  |`ollama pull deepseek-r1`  |64k    |Chain-of-thought reasoning, slower       |
|`mistral`      |`ollama pull mistral`      |8k     |Lightweight fallback                     |
|`llama3.3`     |`ollama pull llama3.3`     |128k   |Strong general baseline                  |
|`phi4`         |`ollama pull phi4`         |16k    |Strong reasoning for size                |

**Hardware requirements:**

|Model size|Min RAM|GPU                          |
|----------|-------|-----------------------------|
|7–8B      |8 GB   |Optional                     |
|14B       |16 GB  |Recommended                  |
|30B+      |32 GB  |Required for acceptable speed|

**Local alternatives to Ollama:**

- **llama.cpp server** — more quantisation control, OpenAI-compatible endpoint
- **LM Studio** — GUI model management, exposes local OpenAI-compatible server
- **Jan** — similar to LM Studio

All three work via the `openai` provider with `base_url` pointed at localhost.

-----

### Anthropic

```bash
export ANTHROPIC_API_KEY=sk-ant-...
rsk config provider anthropic
rsk config model claude-sonnet-4-6
```

|Model              |Notes                         |
|-------------------|------------------------------|
|`claude-sonnet-4-6`|**Recommended — best balance**|
|`claude-opus-4-6`  |Most capable, higher cost     |
|`claude-haiku-4-5` |Fastest, lowest cost          |

Best for: highest reasoning quality, 200k context, long complex sessions.

-----

### NousResearch (Nous API)

Hermes models fine-tuned for agentic behaviour and structured tool-call reasoning.
Accessed directly via the Nous API — not via OpenRouter.

```bash
export NOUS_API_KEY=sk-nous-...
rsk config provider nous
rsk config model hermes-3-llama-3.1-70b
```

|Model                    |Context|Notes                                 |
|-------------------------|-------|--------------------------------------|
|`hermes-3-llama-3.1-405b`|128k   |Most capable Hermes                   |
|`hermes-3-llama-3.1-70b` |128k   |**Recommended — quality/cost balance**|
|`hermes-3-llama-3.1-8b`  |128k   |Lightweight, fast, good for sub-agents|
|`hermes-2-pro-llama-3-8b`|8k     |Solid function-calling baseline       |

**Why Hermes for Raskolnikov:**
Hermes models are fine-tuned on instruction-following and tool-use datasets. This means:
more reliable tool selection, better flag/argument construction, stronger persona adherence
when agent Markdown is injected into the system prompt.

Hermes 3 supports 128k context — full session history fits without summarisation for
most engagements.

For local use: `ollama pull nous-hermes3` (no Nous API key required).

-----

### OpenAI

```bash
export OPENAI_API_KEY=sk-...
rsk config provider openai
rsk config model gpt-4o
```

|Model        |Notes                         |
|-------------|------------------------------|
|`gpt-4o`     |**Recommended**               |
|`gpt-4o-mini`|Lightweight, low cost         |
|`o3-mini`    |Strong reasoning at lower cost|

-----

### OpenRouter

Routes to many providers via a single API key. **Hermes models are not accessed via
OpenRouter in Raskolnikov** — use the Nous API directly for all Hermes models.

```bash
export OPENROUTER_API_KEY=sk-or-...
rsk config provider openrouter
rsk config model meta-llama/llama-3.3-70b-instruct
```

|Model string                       |Notes                |
|-----------------------------------|---------------------|
|`anthropic/claude-sonnet-4-6`      |Claude via OpenRouter|
|`meta-llama/llama-3.3-70b-instruct`|Llama 3.3 70B        |
|`deepseek/deepseek-r1`             |R1 reasoning model   |
|`mistralai/mistral-large`          |Large Mistral        |
|`google/gemini-pro-1.5`            |1M context           |

-----

### Groq

Very fast token generation on open models. Best for real-time feel and (future)
multi-agent workloads where sub-agents need rapid turnaround.

```bash
export GROQ_API_KEY=gsk_...
rsk config provider groq
rsk config model llama-3.3-70b-versatile
```

|Model                          |Context|Notes                               |
|-------------------------------|-------|------------------------------------|
|`llama-3.3-70b-versatile`      |128k   |**Recommended on Groq**             |
|`llama-3.1-8b-instant`         |128k   |Very fast, lighter quality          |
|`deepseek-r1-distill-llama-70b`|128k   |Distilled reasoning on Groq hardware|
|`mixtral-8x7b-32768`           |32k    |MoE, fast                           |

-----

### Llama API (Meta)

Meta’s official hosted Llama API. Llama models without local hardware and without
routing through a third party.

```bash
export LLAMA_API_KEY=...
rsk config provider llama-api
rsk config model llama3.3-70b
```

-----

### Together AI

Wide range of open models, competitive pricing.

```bash
export TOGETHER_API_KEY=...
rsk config provider together
rsk config model meta-llama/Llama-3.3-70B-Instruct-Turbo
```

Notable: `NousResearch/Hermes-3-Llama-3.1-70B` is available on Together as an
alternative to the direct Nous API.

-----

### Provider Comparison

|Provider   |Cost |Speed        |Privacy   |Context   |Best For                                  |
|-----------|-----|-------------|----------|----------|------------------------------------------|
|Ollama     |Free |GPU-dependent|Full local|Up to 128k|Default, privacy-first, air-gapped        |
|Nous API   |Paid |Fast         |API       |128k      |Hermes direct — best tool-call reliability|
|Groq       |Cheap|Very fast    |API       |128k      |Fast open models, real-time feel          |
|Anthropic  |Paid |Fast         |API       |200k      |Highest reasoning quality, long sessions  |
|OpenAI     |Paid |Fast         |API       |128k      |GPT family                                |
|OpenRouter |Paid |Varies       |API       |Varies    |Non-Hermes model switching                |
|Llama API  |Paid |Fast         |API       |128k      |Meta models direct                        |
|Together AI|Paid |Fast         |API       |Varies    |Open model variety                        |

### Context window management

The conversation history grows throughout a session. Raskolnikov’s strategy:

1. Full history passed while within 80% of the model’s context limit
1. At 80%, older tool outputs are summarised and replaced in history
1. Operator warned when summarisation occurs
1. Raw outputs always preserved on disk in `session/tools/`
1. Operator and agent messages are never summarised

**Minimum recommended:** 32k tokens
**Recommended:** 128k+ (covers most full sessions without summarisation)

-----

## TUI Design

Terminal-only. Ratatui is the sole UI layer. No web fallback. No exceptions.

```
┌────────────────────────────────────────────────────────────────────────┐
│  RASKOLNIKOV  alpha 0.1.0                              model: qwen3    │
├──────────────────────────────┬─────────────────────────────────────────┤
│                              │                                         │
│  TOOL OUTPUT                 │  CONVERSATION                           │
│                              │                                         │
│  [live stdout/stderr         │  you    scan 10.0.0.1                  │
│   streams here while         │                                         │
│   a tool is running]         │  agent  Starting with nmap.             │
│                              │         nmap -sV -sC -T4 10.0.0.1      │
│  PORT   STATE  SERVICE       │         — run this?                     │
│  22/tcp open   ssh           │                                         │
│  80/tcp open   http          │  you    yes                             │
│  3306   open   mysql         │                                         │
│                              │  agent  [running nmap...]               │
│                              │                                         │
├──────────────────────────────┴─────────────────────────────────────────┤
│  FINDINGS  ·  22/tcp ssh  ·  80/tcp Apache 2.4.52  ·  3306/tcp mysql  │
├────────────────────────────────────────────────────────────────────────┤
│  > _                                                                   │
└────────────────────────────────────────────────────────────────────────┘
```

### Panels

**Tool Output (left):** Live streaming stdout/stderr while a tool runs. Scrollable.
Cleared between tool runs (all output saved to disk). Dimmed when no tool is running.

**Conversation (right):** Full chat history between operator and agent. Scrollable —
older messages accessible above. Agent responses stream incrementally. Tool invocations
shown inline as part of the agent’s message.

**Findings bar:** Persistent strip of confirmed findings across the whole session.
Updates after each tool run. Key info only: ports, paths, flagged vulnerabilities.

**Input bar:** Single line, always focused when no tool is running. `Enter` sends.
When a tool is running, the input bar remains active but input is queued. The
message is delivered to the agent after the current tool completes. Only one
queued message is kept — typing a second overwrites the first.

### Keybindings

|Key            |Action                            |
|---------------|----------------------------------|
|`Enter`        |Send message                      |
|`PgUp` / `PgDn`|Scroll active panel               |
|`Tab`          |Switch scroll focus between panels|
|`ctrl+c`       |Prompt to end session             |
|`ctrl+l`       |Clear tool output panel           |

-----

## Session & Logging

Every session logged automatically. No opt-in required.

### Directory structure

```
~/.local/share/raskolnikov/
└── sessions/
    └── 2025-06-16T14-22-01/
        ├── session.log          JSON lines — full event log
        ├── conversation.md      Markdown transcript
        ├── findings.md          structured findings export
        └── tools/
            ├── nmap_output.xml
            ├── nikto_output.txt
            ├── gobuster_output.txt
            └── sqlmap_output.txt
```

### session.log (JSON lines)

```json
{"ts":"2025-06-16T14:22:01Z","type":"session_start","model":"qwen3"}
{"ts":"2025-06-16T14:22:05Z","type":"operator","content":"scan 10.0.0.1"}
{"ts":"2025-06-16T14:22:07Z","type":"agent","content":"Starting with nmap..."}
{"ts":"2025-06-16T14:22:09Z","type":"tool_start","tool":"nmap","cmd":"nmap -sV -sC -T4 10.0.0.1"}
{"ts":"2025-06-16T14:23:41Z","type":"tool_end","tool":"nmap","exit_code":0,"duration_s":92}
{"ts":"2025-06-16T14:23:43Z","type":"agent","content":"Found 3 open ports..."}
{"ts":"2025-06-16T14:23:44Z","type":"operator","content":"yes"}
```

### conversation.md (Markdown transcript)

```markdown
# Session: 2025-06-16T14:22:01
**Target:** 10.0.0.1  **Model:** qwen3

---

**[14:22:05] you**
scan 10.0.0.1

**[14:22:07] agent**
Starting with nmap to discover open ports.
`nmap -sV -sC -T4 10.0.0.1` — run this?

**[14:22:09] you**
yes

**[14:22:09] tool: nmap** *(92s)*
PORT     STATE SERVICE VERSION
22/tcp   open  ssh     OpenSSH 8.9
80/tcp   open  http    Apache 2.4.52
3306/tcp open  mysql   MySQL 8.0.33

**[14:23:43] agent**
Found 3 open ports...
```

### findings.md (auto-generated on session end)

```markdown
# Findings: 10.0.0.1
**Date:** 2025-06-16  **Model:** qwen3

## Open Ports
| Port | Service | Version |
|---|---|---|
| 22/tcp | SSH | OpenSSH 8.9 |
| 80/tcp | HTTP | Apache 2.4.52 |
| 3306/tcp | MySQL | MySQL 8.0.33 |

## Web Paths
| Path | Status | Notes |
|---|---|---|
| /admin | 302 | Redirects to /admin/login |
| /uploads | 200 | Directory listing enabled |

## Flags
- MySQL exposed directly to network
- /uploads world-readable
```

### Findings deduplication

Findings are deduplicated by key + value (e.g. port/protocol, path/status).
If nmap reports port 80/tcp and nikto also references port 80/tcp, the
finding appears once in the findings bar and in `findings.md`.

### Abrupt termination recovery

On abrupt termination (SIGKILL, power loss, terminal crash), partial
session data is recoverable: `session.log` is flushed to disk after every
event. `conversation.md` and `findings.md` are written only on clean
shutdown — they may be absent or incomplete if the session was not ended
normally.

### Session cleanup

Sessions are kept indefinitely by default. Operators can prune old
sessions with:

```bash
rsk sessions prune --keep 30    # keep sessions from the last 30 days
rsk sessions prune --keep 10    # keep the 10 most recent sessions
```

### Session commands

```bash
rsk sessions                  list all sessions
rsk sessions show <id>        print conversation.md
rsk sessions findings <id>    print findings.md
rsk sessions log <id>         dump raw JSON log
rsk sessions prune <flags>    remove old sessions
```

-----

## Installation

### Arch Linux

```bash
yay -S raskolnikov
```

AUR package installs the binary, man page, shell completions, and `rsk`/`rk` symlinks.

### Kali / Debian / Ubuntu

Signed `.deb` is the recommended install method:

```bash
wget <repo-url>/releases/download/v0.1.0/raskolnikov_0.1.0_amd64.deb
wget <repo-url>/releases/download/v0.1.0/raskolnikov_0.1.0_amd64.deb.sig
gpg --verify raskolnikov_0.1.0_amd64.deb.sig raskolnikov_0.1.0_amd64.deb
sudo dpkg -i raskolnikov_0.1.0_amd64.deb
```

Script install (fallback — not recommended for a security tool):

```bash
curl -fsSL https://raskolnikov.sh/install | bash
```

### Build from source

Requires Rust stable 1.75+.

```bash
git clone <repo-url>
cd raskolnikov
cargo build --release
sudo install -m755 target/release/raskolnikov /usr/local/bin/raskolnikov
sudo ln -s /usr/local/bin/raskolnikov /usr/local/bin/rsk
sudo ln -s /usr/local/bin/raskolnikov /usr/local/bin/rk
```

-----

## First Launch

```
$ rsk

Checking tools...
  ✓ nmap 7.94  ✓ gobuster 3.6.0  ✓ nikto 2.1.6  ✓ sqlmap 1.7.8

Checking AI providers...
  ✓ Ollama detected

No default model configured.

Available models:
  1. qwen3          [recommended]
  2. nous-hermes3
  3. deepseek-r1
  4. mistral

Select default model [1]:

✓ Config written to ~/.config/raskolnikov/config.toml
✓ Sessions directory created at ~/.local/share/raskolnikov/sessions/

Ready.
```

On subsequent launches the TUI opens immediately with no prompts.

-----

## CLI Reference

```
USAGE
  raskolnikov [options]
  rsk [options]
  rk [options]

OPTIONS
  --version             Print version and exit
  --model <name>        Override default model for this session
  --provider <name>     Override default provider for this session

SUBCOMMANDS
  rsk sessions                  list past sessions
  rsk sessions show <id>        print conversation transcript
  rsk sessions findings <id>    print findings summary
  rsk sessions log <id>         dump raw JSON log
  rsk sessions prune <flags>    remove old sessions
  rsk config                    show current config
  rsk config provider <p>       set AI provider
  rsk config model <m>          set default model
  rsk config set <key> <val>    set an arbitrary config value
  rsk tools                     check tool availability and versions
  rsk help                      show help
```

-----

## Configuration

```toml
# ~/.config/raskolnikov/config.toml

[cli]
alias = ""              # optional custom binary alias

[ai]
provider = "ollama"     # ollama | anthropic | openai | openrouter |
                        # groq | nous | llama-api | together
model    = "qwen3"

[ollama]
host = "http://localhost:11434"

[openai]
base_url = "https://api.openai.com/v1"   # override for local compat servers

[nous]
base_url = "https://inference-api.nousresearch.com/v1"

[groq]
base_url = "https://api.groq.com/openai/v1"

[llama_api]
base_url = "https://api.llama.com/v1"

[together]
base_url = "https://api.together.xyz/v1"

[tools]
prefer_ffuf    = false
nmap_timing    = 4
sqlmap_level   = 2
sqlmap_risk    = 1

[wordlists]
paths = [
  "/usr/share/wordlists/dirbuster/directory-list-2.3-medium.txt",
  "/usr/share/seclists/Discovery/Web-Content/common.txt",
  "/usr/share/wordlists/dirb/common.txt",
]

[ui]
stream_output = true

[network]
proxy          = ""     # HTTP proxy for AI API calls
proxy_https    = ""     # HTTPS proxy for AI API calls
no_proxy       = ["localhost", "127.0.0.1"]
```

All API keys are read from environment variables — never stored in `config.toml`.

```bash
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
OPENROUTER_API_KEY=sk-or-...
GROQ_API_KEY=gsk_...
NOUS_API_KEY=sk-nous-...
LLAMA_API_KEY=...
TOGETHER_API_KEY=...
```

-----

## Security Considerations

### Command injection prevention

All tool commands are constructed by the AI and parameterised by the agent.
Raskolnikov validates basic structure — target strings are sanitised to
prevent shell metacharacter injection — but does not fully sandbox
subprocess execution. Operators should review proposed commands before
approving them (always-on confirmation is the default).

### API key safety

API keys are read from environment variables only. They are never stored in
`config.toml`, never written to session logs, never included in AI context
messages, and never transmitted anywhere except to the configured provider.

### Session data confidentiality

Session logs, tool output, and findings are stored at `~/.local/share/raskolnikov/`
with `0600` file permissions. The operator is responsible for secure disposal
of session data when it is no longer needed.

### Network isolation

Raskolnikov makes no outbound network requests except to the configured AI
provider(s). No telemetry. No crash reporting. No phone-home. No automatic
update checks. Fully air-gapped operation is supported with Ollama.

### Reporting vulnerabilities

Security issues should be reported privately to the maintainers via the
repository's security advisory process — not via public issues.

-----

## Technology Stack

|Component          |Choice                   |Reason                                        |
|-------------------|-------------------------|----------------------------------------------|
|Language           |Rust (stable 1.75+)      |Static binary, memory safety, fast startup    |
|TUI                |Ratatui                  |Terminal-native, composable, async-friendly   |
|Async runtime      |Tokio                    |Process spawning, streaming I/O, concurrent UI|
|AI — local         |Ollama HTTP API          |Local-first, no auth, open model support      |
|AI — cloud         |Direct HTTP per provider |No SDK dependency, minimal binary size        |
|Serialization      |serde + serde_json       |Session logging, config, API payloads         |
|Config             |toml                     |Human-writable                                |
|Knowledge artifacts|Markdown (pulldown-cmark)|Agents, transcripts, findings                 |
|Process execution  |tokio::process           |Async subprocess, streaming stdout/stderr     |

-----

## Project Structure

```
raskolnikov/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── SPEC.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── src/
│   ├── main.rs                  entrypoint — parse flags, launch TUI
│   ├── config.rs                config loading and defaults
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── shell.rs             conversation loop
│   │   ├── context.rs           EngagementContext — findings, history
│   │   └── prompt.rs            system prompt builder
│   ├── tools/
│   │   ├── mod.rs               Tool trait + availability check
│   │   ├── executor.rs          subprocess runner, output streaming
│   │   ├── nmap.rs              XML parser
│   │   ├── gobuster.rs
│   │   ├── nikto.rs
│   │   └── sqlmap.rs
│   ├── ai/
│   │   ├── mod.rs               Provider trait
│   │   ├── ollama.rs
│   │   ├── anthropic.rs
│   │   ├── openai.rs            (also handles groq, llama-api, together — OAI-compat)
│   │   ├── openrouter.rs
│   │   └── nous.rs
│   ├── session/
│   │   ├── mod.rs
│   │   ├── logger.rs            JSON lines writer
│   │   ├── transcript.rs        conversation.md writer
│   │   └── findings.rs          findings.md exporter
│   └── tui/
│       ├── mod.rs
│       ├── app.rs               TUI state
│       ├── layout.rs            three-panel layout
│       └── input.rs             input handler
└── packaging/
    ├── PKGBUILD
    ├── raskolnikov.deb.spec
    └── raskolnikov.1
```

-----

## Alpha Scope & Roadmap

### Alpha 0.1.0 — In Scope

The goal of 0.1.0 is a working agent shell: conversation, four tools, everything logged.

|Feature                                   |      |
|------------------------------------------|------|
|Agent shell — `rsk`, `rk`, `raskolnikov`  |✓     |
|Short aliases `rsk` and `rk`              |✓     |
|Custom alias via config                   |✓     |
|Free-form natural language interaction    |✓     |
|Mid-session redirection                   |✓     |
|Single default agent (hardcoded)          |✓     |
|Per-step operator confirmation — always on|✓     |
|nmap + XML parsing                        |✓     |
|gobuster / ffuf                           |✓     |
|nikto                                     |✓     |
|sqlmap                                    |✓     |
|Live tool output streaming                |✓     |
|Full conversation history in model context|✓     |
|JSON lines session log                    |✓     |
|`conversation.md` transcript              |✓     |
|`findings.md` export on session end       |✓     |
|Ollama (default)                          |✓     |
|Anthropic, OpenAI, OpenRouter, Groq       |Opt-in|
|Nous API (Hermes direct)                  |Opt-in|
|Llama API, Together AI                    |Opt-in|
|Tool availability check on startup        |✓     |
|AUR package (Arch)                        |✓     |
|Signed `.deb` (Kali/Debian/Ubuntu)        |✓     |
|Build from source                         |✓     |

### Out of Scope for 0.1.0

|Feature                                                     |Target   |
|------------------------------------------------------------|---------|
|Agent switching, built-in agent library, AI agent generation|0.2.0    |
|Skills system                                               |0.2.0    |
|`/auto` and `/loop` autonomous modes                        |0.2.0    |
|Knowledge graph and cross-session memory                    |0.2.0    |
|Replay engine                                               |0.2.0    |
|hydra, enum4linux-ng, whatweb, feroxbuster                  |0.2.0    |
|Teams and workflows systems                                 |0.3.0    |
|Hook system                                                 |0.3.0    |
|Metasploit, john/hashcat, crackmapexec, kerbrute, impacket  |0.3.0    |
|Scope enforcement, multi-target                             |0.3.0    |
|Concurrent multi-agent, MCP, marketplaces, plugin system    |0.4.0    |
|nuclei, Burp Suite, dnsx/subfinder                          |0.4.0    |
|BloodHound CE, Certipy                                      |0.5.0+   |
|macOS first-class, shell completions                        |0.4.0    |
|Web UI                                                      |**Never**|
|Windows support                                             |**Never**|

-----

### Roadmap

#### Alpha 0.1.0 *(current)* — It Works

One agent. Four tools. A conversation. Everything logged. Local AI default. Eight provider options.

#### Alpha 0.2.0 — Agents & Memory

- Markdown agent system — switchable personas, built-in library (default, webhunter, domainhunter, researcher)
- AI-generated agents (`/agent create`)
- Skills system — Markdown playbooks, built-in library, save from session, AI-generate
- `/auto` mode — semi-autonomous, recon tools only
- `/loop` mode — continuous loop, passive tools, exit conditions
- SQLite knowledge graph — persistent findings across sessions
- Cross-session target recall
- Replay engine (`rsk replay <id>`)
- New tools: hydra (gated), enum4linux-ng, whatweb, feroxbuster

#### Beta 0.3.0 — Exploitation & Automation

- Teams system — Markdown multi-agent configs (sequential)
- Workflows system — YAML declarative pipelines
- Hook system — external scripts at session events
- Metasploit via MSFRPC
- John / hashcat, crackmapexec / netexec, kerbrute, impacket
- Scope enforcement (CIDR/domain files)
- Multi-target engagement support

#### 0.4.0 — Ecosystem

- True concurrent multi-agent (parallel model instances, per-agent provider config)
- MCP server support + community MCP integrations
- Agent marketplace, skill marketplace, tool plugin system
- nuclei, Burp Suite headless, dnsx/subfinder/amass, ligolo-ng/chisel
- macOS first-class support
- Shell completions: bash, zsh, fish

#### 0.5.0+ — Intelligence

- BloodHound CE integration
- Certipy (AD CS attacks)
- Full report generation (Markdown → PDF)
- Cross-engagement pattern recognition
- Auto-skill generation from repeated manual steps

-----

## Future Systems

Brief descriptions of major systems designed but not yet built.

### Agent System (0.2.0)

Agents are Markdown documents defining a persona, expertise, and behaviour. Stored in
`~/.config/raskolnikov/agents/`. Active agent injected into system prompt on every turn.
Switchable mid-session with context preserved. AI-generatable via `/agent create`.

### Skills System (0.2.0)

Skills are Markdown documents describing reusable step sequences. Stored in
`~/.config/raskolnikov/skills/`. Operator loads a skill (`/skill use web-recon`),
agent follows the playbook while still reasoning and adapting. Saveable from session
history. AI-generatable.

### Teams System (0.3.0)

Teams are Markdown documents defining multi-agent configurations. In 0.3.0, agents
run sequentially with context handoff. In 0.4.0, true concurrent parallel execution.

### Workflows System (0.3.0)

YAML declarative automation pipelines. Unlike skills (model-driven), workflows are
deterministic — fixed step sequences for repeatable, consistent engagements.

### Knowledge Graph (0.2.0)

SQLite graph of targets, findings, and relationships. Enables cross-session memory
and attack path reasoning.

```
Host → Port → Service → Version
           → Vulnerability → CVE
Host → Credential
Host → Attack Path → Steps
```

### Autonomous Modes (0.2.0)

**`/auto`:** Semi-autonomous — executes recon steps without per-step confirmation.
Operator can interrupt anytime. Destructive tools always gated.

**`/loop`:** Continuous — agent runs until goal complete, stuck, or timed out.
Passive/non-destructive tools only by default.

### Hook System (0.3.0)

External scripts triggered at session events (`on_finding`, `on_tool_end`,
`on_session_end`, etc.). Environment variables carry session context to hooks.
Use cases: Slack notifications, Obsidian export, scope enforcement, CI pipelines.

### MCP Support (0.4.0)

Model Context Protocol integration. Community-built MCP servers extend Raskolnikov’s
tooling without core modifications: mcp-metasploit, mcp-burp, mcp-cvesearch,
mcp-scope, mcp-notes, and more.

### Multi-Agent (0.4.0)

Parallel model instances with specialised roles: Orchestrator, WebHunter, DomainHunter,
Researcher, Reporter. Per-agent provider config. Operator still sees one interface.

-----

## License

**Apache-2.0**

- Commercial use permitted
- Enterprise adoption friendly
- Patent protection clause
- Encourages community contributions

Full license text in `LICENSE`.

-----

## Contributing

Most valuable contributions for alpha:

1. **Bug reports** — especially on non-Arch distros
1. **Tool output parsing** — nmap XML edge cases, sqlmap output variants
1. **System prompt tuning** — better agent behaviour on smaller local models
1. **Wordlist path detection** — common paths across distros

See `CONTRIBUTING.md` in the repository.

-----

*Raskolnikov alpha 0.1.0 — a markdown-native security agent operating environment.*
