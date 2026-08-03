# Roadmap

## 0.1.0-alpha — It Works *(current)*

One agent. Four tools. A conversation. Everything logged. Local AI default.
Eight provider options.

## 0.2.0 — Agents & Memory

- Markdown agent system — switchable personas, built-in library
  (default, webhunter, domainhunter, researcher)
- AI-generated agents (`/agent create`)
- Skills system — Markdown playbooks, built-in library, save from session,
  AI-generate
- `/auto` mode — semi-autonomous, recon tools only
- `/loop` mode — continuous loop, passive tools, exit conditions
- Knowledge graph (SQLite) — persistent findings across sessions
- Cross-session target recall
- Replay engine (`rsk replay <id>`)

## 0.3.0 — Exploitation & Automation

- Teams system — Markdown multi-agent configs (sequential)
- Workflows system — YAML declarative pipelines
- Hook system — external scripts at session events
- Metasploit via MSFRPC
- Scope enforcement (CIDR/domain files)
- Multi-target engagement support

## 0.4.0 — Ecosystem

- True concurrent multi-agent (parallel model instances, per-agent provider
  config)
- MCP server support + community MCP integrations
- Agent marketplace, skill marketplace, tool plugin system
- Burp Suite headless, dnsx/subfinder/amass, ligolo-ng/chisel
- macOS first-class support
- Shell completions: bash, zsh, fish

## 0.5.0+ — Intelligence

- BloodHound CE integration
- Certipy (AD CS attacks)
- Full report generation (Markdown → PDF)
- Cross-engagement pattern recognition
- Auto-skill generation from repeated manual steps

## Never

- Web UI — terminal-only, permanent constraint
- Windows support — not planned
- Telemetry or phone-home of any kind
