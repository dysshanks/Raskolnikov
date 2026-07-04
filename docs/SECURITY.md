# Security

## Reporting vulnerabilities

This project is for authorised security testing and CTF competitions. If you
find a security vulnerability in Raskolnikov itself (not in the tools it wraps),
please report it privately by contacting the repository owner.

Do not file public GitHub issues for security vulnerabilities.

## Security design

- **API keys** are read from environment variables only. Never stored in config
  files, logs, or session data.
- **Shell injection** is prevented by passing arguments via
  `std::process::Command::arg()` rather than shell strings. User input is never
  concatenated into shell commands.
- **Session files** are created with restricted permissions (0600).
- **No telemetry** — Raskolnikov does not phone home, collect usage data, or
  make network requests except to the AI provider configured by the user.
- **No third-party dependencies** beyond the AI provider SDKs and standard
  penetration testing tools the user has explicitly installed.

## Operational security for users

- Raskolnikov runs external tools (nmap, gobuster, nikto, sqlmap) with the
  permissions of the invoking user. Be mindful of what you scan and from where.
- Session data (transcripts, findings, raw logs) is written to
  `~/.local/share/raskolnikov/sessions/`. Protect this directory if it contains
  sensitive engagement data.
- The tool does not validate that you have authorisation to scan a target. That
  is your responsibility.
