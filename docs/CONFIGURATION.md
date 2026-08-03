# Configuration

Configuration is stored in TOML at `~/.config/raskolnikov/config.toml`
(override with `RASKOLNIKOV_CONFIG`). All fields use `#[serde(default)]` —
any key may be omitted.

Merge order: built-in defaults → `config.toml` → environment variables →
CLI flags.

## Example

```toml
[cli]
alias = ""               # optional custom binary alias (creates symlink)

[ai]
provider = "auto"        # auto | ollama | anthropic | openai | openrouter |
                         # groq | nous | llama-api | together
model = "qwen3"
context_window = 131072  # context limit; 80% triggers summarisation

[ollama]
host = "http://localhost:11434"

[anthropic]
base_url = "https://api.anthropic.com"

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
mouse = true             # mouse capture; set to false to enable terminal text selection

[network]
proxy          = ""      # HTTP proxy for AI API calls
proxy_https    = ""      # HTTPS proxy for AI API calls
no_proxy       = ["localhost", "127.0.0.1"]
timeout_secs   = 60      # per-read timeout for AI requests; 0 = no timeout

[colors]
accent    = "red"
surface   = "dark_gray"
highlight = "red"

use_tmp_data_dir = false # store session data under /tmp instead of ~/.local/share
```

## Defaults

| Key | Default |
|-----|---------|
| `ai.provider` | `auto` |
| `ai.model` | `qwen3` |
| `ai.context_window` | `131072` |
| `ollama.host` | `http://localhost:11434` |
| `tools.prefer_ffuf` | `false` |
| `tools.nmap_timing` | `4` |
| `tools.sqlmap_level` | `2` |
| `tools.sqlmap_risk` | `1` |
| `ui.stream_output` | `true` |
| `ui.mouse` | `true` |
| `network.timeout_secs` | `60` |
| `colors.accent` | `red` |

Wordlist resolution order (first match wins):

```
/usr/share/wordlists/dirbuster/directory-list-2.3-medium.txt
/usr/share/seclists/Discovery/Web-Content/common.txt
/usr/share/seclists/Discovery/Web-Content/raft-medium-words.txt
/usr/share/wordlists/dirb/common.txt
/usr/share/wordlists/dirb/big.txt
```

Falls back to a minimal built-in wordlist (500 entries) if none are found.

## API keys

API keys are read from environment variables only — never stored in
`config.toml`, session logs, or AI context:

```bash
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
OPENROUTER_API_KEY=sk-or-...
GROQ_API_KEY=gsk_...
NOUS_API_KEY=sk-nous-...
LLAMA_API_KEY=...
TOGETHER_API_KEY=...
```

## Data directories

Session data defaults to `~/.local/share/raskolnikov/sessions/`. Override with
`RASKOLNIKOV_DATA`, or set `use_tmp_data_dir = true` to use a temporary
directory (useful for throwaway/air-gapped sessions).

## CLI

Configuration can be managed from the command line — see [CLI.md](CLI.md).
