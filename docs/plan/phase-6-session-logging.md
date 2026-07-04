# Phase 6 — Session Logging & Export

**Objective:** Implement automatic session logging (JSON lines), Markdown transcript export, findings export with deduplication, and the `rsk sessions` CLI subcommand tree.

---

## Files to Create / Modify

| File | Status | Purpose |
|------|--------|---------|
| `src/session/mod.rs` | Rewrite | Re-export logger, transcript, findings, commands |
| `src/session/logger.rs` | Create | JSON lines writer, flushed per event |
| `src/session/transcript.rs` | Create | `conversation.md` writer |
| `src/session/findings.rs` | Create | `findings.md` writer + dedup |
| `src/main.rs` | Modify | Route `rsk sessions *` subcommands to session module |

---

## Event Types (session.log)

```json
{"ts":"2025-06-16T14:22:01Z","type":"session_start","model":"qwen3","provider":"ollama"}
{"ts":"2025-06-16T14:22:05Z","type":"operator","content":"scan 10.0.0.1"}
{"ts":"2025-06-16T14:22:07Z","type":"agent","content":"Starting with nmap to discover open ports..."}
{"ts":"2025-06-16T14:22:09Z","type":"tool_start","tool":"nmap","args":"nmap -sV -sC -T4 10.0.0.1"}
{"ts":"2025-06-16T14:23:41Z","type":"tool_end","tool":"nmap","exit_code":0,"duration_s":92}
{"ts":"2025-06-16T14:23:43Z","type":"finding","source":"nmap","content":"Port 22/tcp: SSH OpenSSH 8.9"}
{"ts":"2025-06-16T14:23:44Z","type":"operator","content":"yes"}
{"ts":"2025-06-16T14:23:44Z","type":"agent","content":"Found 3 open ports..."}
{"ts":"2025-06-16T14:25:00Z","type":"context_summary","trigger":"80% threshold","summarised":3}
{"ts":"2025-06-16T14:30:00Z","type":"session_end","duration_s":480}
```

Every event is flushed to disk immediately via `BufWriter::flush()` for crash recovery.

---

## Directory Structure

```
~/.local/share/raskolnikov/
└── sessions/
    └── 2025-06-16T14-22-01/
        ├── session.log          JSON lines — every event
        ├── conversation.md      Markdown — human-readable transcript
        ├── findings.md          Markdown — structured findings summary
        └── tools/
            ├── nmap_output.xml
            ├── nikto_output.txt
            ├── gobuster_output.txt
            └── sqlmap_output.txt
```

The session directory name is the ISO 8601 timestamp at session start with colons replaced by hyphens: `2025-06-16T14-22-01`.

---

## conversation.md Format

```markdown
# Session: 2025-06-16T14:22:01
**Target:** 10.0.0.1  **Model:** qwen3  **Provider:** ollama

---

**[14:22:05] you**
scan 10.0.0.1

**[14:22:07] agent**
Starting with nmap to discover open ports.
`nmap -sV -sC -T4 10.0.0.1` — run this?

**[14:22:09] you**
yes

**[14:22:09] tool: nmap** *(92s)*
```
PORT     STATE SERVICE VERSION
22/tcp   open  ssh     OpenSSH 8.9
80/tcp   open  http    Apache 2.4.52
3306/tcp open  mysql   MySQL 8.0.33
```

**[14:23:43] agent**
Found 3 open ports...

**[14:23:44] finding**
Port 3306/tcp: MySQL exposed directly to network
```

Only written on clean session end. On abrupt termination, file may be absent.

---

## findings.md Format

```markdown
# Findings: 10.0.0.1
**Date:** 2025-06-16  **Model:** qwen3  **Provider:** ollama

## Open Ports
| Port | Protocol | Service | Version |
|------|----------|---------|---------|
| 22/tcp | TCP | SSH | OpenSSH 8.9 |
| 80/tcp | TCP | HTTP | Apache 2.4.52 |
| 3306/tcp | TCP | MySQL | MySQL 8.0.33 |

## Web Paths
| Path | Status | Notes |
|------|--------|-------|
| /admin | 302 | Redirects to /admin/login |
| /uploads | 200 | Directory listing enabled |

## Flags
- MySQL exposed directly to network
- /uploads world-readable
```

---

## Deduplication Rules

Before writing findings.md, run dedup:

| Finding Type | Dedup Key |
|-------------|-----------|
| Port | `{port}/{protocol}` |
| Web Path | `{path}` |
| Flag / Vulnerability | `{description}` (case-insensitive) |
| CVE | `{CVE-ID}` |

Latest occurrence wins (in case of updated information).

---

## Session CLI Subcommands

These run outside the TUI — print to stdout and exit.

```bash
rsk sessions                          # list all session dirs with date + model
  2025-06-16T14-22-01  qwen3         10.0.0.1
  2025-06-15T09-12-33  claude-sonnet  scanme.org

rsk sessions show 2025-06-16T14-22-01    # cat conversation.md

rsk sessions findings 2025-06-16T14-22-01  # cat findings.md

rsk sessions log 2025-06-16T14-22-01      # cat session.log

rsk sessions prune --keep 30    # delete sessions older than 30 days
rsk sessions prune --keep 10    # keep only 10 most recent
```

### Prune Implementation

- `--keep 30`: filter sessions by directory timestamp, delete dirs where age > 30 days
- `--keep 10`: sort by timestamp, delete all but the 10 most recent
- Dry-run mode: `--dry-run` flag shows what would be deleted without deleting
- Confirmation prompt before deletion when run interactively

---

## Crash Recovery

- `session.log`: flushed after every event — always complete up to the last event before the crash
- `conversation.md` and `findings.md`: only written on clean `session_end` event
- On next launch, any incomplete session directories (no `session_end` event in log) are flagged:
  ```
  ⚠ Found incomplete session: 2025-06-16T14-22-01 (abrupt termination)
     session.log available with partial data
  ```
- Recovery command: `rsk sessions recover <id>` attempts to rebuild conversation.md and findings.md from the partial log

---

## User Stories

| ID | Title | Description |
|----|-------|-------------|
| US-6.1 | Everything logged automatically | As a user, every session is logged without me needing to opt in or remember |
| US-6.2 | Human-readable transcript | As a user, I can read `conversation.md` in any text editor |
| US-6.3 | Findings exported | As a user, `findings.md` gives me a clean summary of discoveries |
| US-6.4 | List past sessions | As a user, `rsk sessions` shows me all my past sessions |
| US-6.5 | View old transcripts | As a user, `rsk sessions show <id>` prints a past conversation |
| US-6.6 | Clean up old sessions | As a user, `rsk sessions prune --keep 30` removes old sessions |

---

## Acceptance Criteria

| ID | Criterion | How to Verify |
|----|-----------|---------------|
| AC-6.1 | Session directory created on launch at `~/.local/share/raskolnikov/sessions/<ts>/` | Launch `rsk`, check directory |
| AC-6.2 | `session.log` contains valid JSON lines with all events | Run a few turns, inspect log |
| AC-6.3 | `session.log` is flushed after every event (crash-safe) | Kill the process with SIGKILL, check last log line |
| AC-6.4 | `conversation.md` written on clean exit | Start and end session, verify file |
| AC-6.5 | `findings.md` written on clean exit with dedup | Run multiple tools reporting same port, verify single entry |
| AC-6.6 | `rsk sessions` lists all session dirs with metadata | Run command |
| AC-6.7 | `rsk sessions show <id>` prints valid Markdown | Run command, verify output |
| AC-6.8 | `rsk sessions findings <id>` prints findings | Run command |
| AC-6.9 | `rsk sessions log <id>` prints raw JSON lines | Run command |
| AC-6.10 | `rsk sessions prune --keep 30` removes sessions older than 30 days | Create old test dirs, run prune |
| AC-6.11 | `rsk sessions prune --keep 10 --dry-run` shows what would be deleted | Run dry-run, verify no deletion |
| AC-6.12 | Tool output files saved in `session/tools/` directory | Run a tool, check directory |
| AC-6.13 | Raw nmap XML saved to `session/tools/nmap_output.xml` | Run nmap, check file |
| AC-6.14 | Incomplete sessions flagged on next launch | Simulate abrupt termination, re-launch |

---

## Order of Implementation

1. `src/session/logger.rs` — JSON lines writer with auto-flush
2. Integrate logger into agent shell (hook into operator/agent/tool events)
3. `src/session/transcript.rs` — conversation.md writer
4. `src/session/findings.rs` — findings.md writer + dedup
5. Wire transcript + findings into session end handler
6. `rsk sessions` CLI subcommands (list, show, findings, log)
7. `rsk sessions prune` with --keep and --dry-run
8. Crash recovery: flag incomplete sessions, `rsk sessions recover`
