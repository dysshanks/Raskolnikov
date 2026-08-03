# CLI

## Synopsis

```
raskolnikov [options]
rsk [options]
rk [options]

raskolnikov sessions <action> [options]
raskolnikov config [action] [key value]
raskolnikov tools
```

`rsk` and `rk` are symlinks to `raskolnikov`. A custom alias can also be set
via `[cli] alias` in config.

## Options

| Option | Description |
|--------|-------------|
| `--version` | Print version and exit |
| `--model <name>` | Override the default AI model for this session |
| `--provider <name>` | Override the default provider for this session |

Provider values: `auto`, `ollama`, `anthropic`, `openai`, `openrouter`,
`groq`, `nous`, `llama-api`, `together`.

## Subcommands

### sessions

| Command | Description |
|---------|-------------|
| `sessions list` | List all saved sessions (marks incomplete ones) |
| `sessions show <id>` | Print the `conversation.md` transcript |
| `sessions findings <id>` | Print the `findings.md` summary |
| `sessions log <id>` | Dump the raw JSON session log |
| `sessions prune --keep N` | Keep sessions from the last N days |
| `sessions prune --keep Nr` | Keep the N most recent sessions |
| `sessions recover <id>` | Rebuild `conversation.md` + `findings.md` from the log |

### config

| Command | Description |
|---------|-------------|
| `config show` | Display current configuration as TOML |
| `config provider <name>` | Set the AI provider |
| `config model <name>` | Set the default model |
| `config set <key> <value>` | Set an arbitrary config key |

Supported `config set` keys: `provider`, `model`, `ollama_host`, `nmap_timing`,
`prefer_ffuf`, `sqlmap_level`, `sqlmap_risk`, `stream_output`, `alias`,
`context_window`, `proxy`, `proxy_https`, `no_proxy`, `timeout_secs`,
`use_tmp_data_dir`.

### tools

Check availability and versions of all security tools. See [TOOLS.md](TOOLS.md)
for the full list.

## Exit codes

`0` on success. Non-zero with an error message when a subcommand fails (e.g.
unknown session ID or invalid config key).
