# Sessions

Every session is logged automatically. No opt-in required.

## Directory structure

```
~/.local/share/raskolnikov/
└── sessions/
    └── 2025-06-16T14-22-01/
        ├── session.log          JSON lines — full event log
        ├── conversation.md      Markdown transcript
        ├── findings.md          structured findings export
        └── tools/
            ├── nmap_output.xml
            ├── gobuster_output.txt
            ├── hydra_output.txt
            └── ...
```

Each `rsk` run creates a new `YYYY-MM-DDTHH-MM-SS` session directory.
Override the data root with `RASKOLNIKOV_DATA` or `use_tmp_data_dir`.

## session.log (JSON lines)

```json
{"ts":"2025-06-16T14:22:01Z","type":"session_start","model":"qwen3"}
{"ts":"2025-06-16T14:22:05Z","type":"operator","content":"scan 10.0.0.1"}
{"ts":"2025-06-16T14:22:07Z","type":"agent","content":"Starting with nmap..."}
{"ts":"2025-06-16T14:22:09Z","type":"tool_start","tool":"nmap","cmd":"nmap -sV -sC -T4 10.0.0.1"}
{"ts":"2025-06-16T14:23:41Z","type":"tool_end","tool":"nmap","exit_code":0,"duration_s":92}
{"ts":"2025-06-16T14:23:43Z","type":"agent","content":"Found 3 open ports..."}
{"ts":"2025-06-16T14:23:44Z","type":"operator","content":"yes"}
```

The log is flushed to disk after every event, so partial sessions survive
abrupt termination (SIGKILL, power loss, terminal crash).

## conversation.md (Markdown transcript)

```markdown
# Session: 2025-06-16T14:22:01
**Target:** 10.0.0.1  **Model:** qwen3

---

**[14:22:05] you**
scan 10.0.0.1

**[14:22:07] agent**
Starting with nmap to discover open ports.
`nmap -sV -sC -T4 10.0.0.1` — run this?

**[14:22:09] tool: nmap** *(92s)*
PORT     STATE SERVICE VERSION
22/tcp   open  ssh     OpenSSH 8.9
80/tcp   open  http    Apache 2.4.52
3306/tcp open  mysql   MySQL 8.0.33
```

## findings.md (written on session end)

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

## Credentials
| Service | Host | User | Password |
|---|---|---|---|
| ssh | 10.0.0.1 | root | toor |

## Flags
- MySQL exposed directly to network
- /uploads world-readable
```

Findings are deduplicated by key + value. If nmap reports port 80/tcp and
nikto also references port 80/tcp, the finding appears once.

Tool output parsing feeds the findings: nmap ports, gobuster/ffuf/feroxbuster
paths, nikto issues, sqlmap injections, hydra/netexec credentials, john/hashcat
cracked hashes, whatweb technologies, nuclei CVEs, enum4linux users/shares,
httpx probes.

## Session commands

```bash
rsk sessions                  list all sessions
rsk sessions show <id>        print conversation.md
rsk sessions findings <id>    print findings.md
rsk sessions log <id>         dump raw JSON log
rsk sessions prune --keep 30  keep sessions from the last 30 days
rsk sessions prune --keep 10r keep the 10 most recent sessions
rsk sessions recover <id>     rebuild conversation.md + findings.md from session.log
```

Sessions are kept indefinitely by default. Prune old sessions with
`rsk sessions prune`.

## Recovery

`conversation.md` and `findings.md` are written only on clean shutdown. If a
session was killed, they may be absent or incomplete. `rsk sessions recover
<id>` rebuilds both from the `session.log` event stream.
