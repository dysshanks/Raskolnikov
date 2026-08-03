# Security

## Reporting vulnerabilities

This project is for authorised security testing and CTF competitions. If you
find a security vulnerability in Raskolnikov itself (not in the tools it wraps),
please report it privately by contacting the repository owner.

Do not file public GitHub issues for security vulnerabilities.

## Security design

- **API keys** are read from environment variables only. Never stored in config
  files, logs, or session data.
- **Shell injection** is prevented on two fronts: commands are parsed with
  `shell-words` tokenization (quoting-aware, no shell evaluation) before being
  passed via `Command::args()`, and never concatenated into a shell string.
  Empty commands are rejected outright.
- **Process isolation** — each tool runs in its own process group; interrupting
  sends SIGTERM to the whole group, with SIGKILL after 5s if ignored.
- **Terminal injection** — tool output is sanitized at the TUI boundary: ANSI
  CSI/OSC sequences and C0 control characters are stripped before rendering, so
  tool output cannot inject escape sequences into your terminal.
- **Session files** are created with restricted permissions (0600).
- **`/update` is gated** — the self-update command only runs from a git
  checkout (requires a `.git` directory), preventing accidental updates from
  non-git installations.
- **No telemetry** — Raskolnikov does not phone home, collect usage data, or
  make network requests except to the AI provider configured by the user.

## Operational security for users

- Raskolnikov runs external tools (nmap, gobuster, nikto, sqlmap, hydra, and
  more) with the permissions of the invoking user. Be mindful of what you scan
  and from where.
- Session data (transcripts, findings, raw logs) is written to
  `~/.local/share/raskolnikov/sessions/`. Protect this directory if it contains
  sensitive engagement data.
- The tool does not validate that you have authorisation to scan a target. That
  is your responsibility.
