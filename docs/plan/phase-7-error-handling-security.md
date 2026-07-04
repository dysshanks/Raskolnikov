# Phase 7 — Error Handling & Security

**Objective:** Wire the error handling table through the entire application, implement command injection prevention, proxy support, and ensure API keys are never leaked.

---

## Files to Create / Modify

| File | Status | Purpose |
|------|--------|---------|
| `src/error.rs` | Create | Unified error types for the whole application |
| `src/config.rs` | Modify | Add `[network]` proxy section, target sanitisation config |
| `src/tools/executor.rs` | Modify | Add target sanitisation before spawning |
| `src/tui/app.rs` | Modify | Add error display components (toast/notification) |
| `src/agent/shell.rs` | Modify | Wrap all fallible operations in error handling |
| `src/session/logger.rs` | Modify | Ensure API keys stripped from any logged content |

---

## Unified Error Types (src/error.rs)

```rust
#[derive(Debug, thiserror::Error)]
pub enum RaskError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("AI provider error: {0}")]
    Provider(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Tool interrupted by operator")]
    ToolInterrupted,

    #[error("Tool timed out after {0} seconds")]
    ToolTimeout(u64),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}
```

Using `thiserror` for ergonomic error derivation (add to Cargo.toml).

---

## Error Handling Table

| Scenario | Implementation |
|----------|----------------|
| Tool not found at startup | `ToolRegistry::check_all_available()` returns `None`, agent skips, warning logged |
| Tool crashes mid-execution | `ToolRunResult.exit_code != 0` → log + display "tool {name} exited with code {n}" |
| Tool runs > 30 min | `tokio::select` timeout branch → SIGTERM → SIGKILL after 5s → `RaskError::ToolTimeout` |
| AI provider unreachable | `reqwest` timeout/connection error → retry once after 3s → display error in TUI |
| AI provider returns invalid JSON | Catch serde parse error → log warning → request plain text response |
| Session dir unwritable | Check permissions at startup → `RaskError::Session` → refuse to launch |
| Config corrupt or missing | `toml::de::Error` → use defaults, warn on first boot |
| Target contains shell metacharacters | Sanitise before passing to tool command → log warning |

---

## Command Injection Prevention

### Target Sanitisation

Before any tool command is built, the target string is sanitised:

```rust
pub fn sanitise_target(raw: &str) -> Result<String, RaskError> {
    let forbidden = [';', '`', '$', '|', '&', '>', '<', '(', ')', '{', '}', '\n', '\r'];
    if raw.contains(forbidden) {
        return Err(RaskError::Parse(format!(
            "Target contains forbidden characters: {:?}",
            raw.chars().filter(|c| forbidden.contains(c)).collect::<Vec<_>>()
        )));
    }
    // Also strip leading/trailing whitespace
    Ok(raw.trim().to_string())
}
```

### Command Construction

Tool commands are constructed using `std::process::Command` (via `tokio::process::Command`) with array-form arguments — never via shell string:

```rust
// SAFE: array form, no shell interpretation
Command::new("nmap")
    .arg("-sV")
    .arg("-sC")
    .arg("-T4")
    .arg(&sanitised_target);
```

### What is NOT prevented (alpha scope)

- The AI could propose a tool with dangerous flags (e.g., `nmap --script=exploit`). This is gated by operator confirmation.
- The AI could propose running an unexpected tool. This is gated by the tool registry (only 4 known tools).
- Full sandboxing / containerisation is out of scope for 0.1.0.

---

## API Key Safety

### Storage

- API keys are read from environment variables only
- Never stored in `config.toml`
- Never written to `session.log`
- Never included in AI context/system prompt messages
- Never logged in tracing/debug output

### Implementation

```rust
pub struct ApiKeys {
    pub anthropic: Option<String>,
    pub openai: Option<String>,
    pub openrouter: Option<String>,
    pub groq: Option<String>,
    pub nous: Option<String>,
    pub llama: Option<String>,
    pub together: Option<String>,
}

impl ApiKeys {
    pub fn from_env() -> Self {
        Self {
            anthropic: std::env::var("ANTHROPIC_API_KEY").ok(),
            openai: std::env::var("OPENAI_API_KEY").ok(),
            // ...
        }
    }
}
```

### Key Redaction in Logs

The logger has a list of redacted patterns. Any log line matching an API key pattern has the value replaced with `[REDACTED]`:

```rust
// In logger::write_event
fn redact_keys(raw: &str) -> String {
    let patterns = [
        r"sk-ant-[a-zA-Z0-9_-]{10,}",
        r"sk-[a-zA-Z0-9]{10,}",
        r"gsk_[a-zA-Z0-9]{10,}",
        // etc.
    ];
    // Replace matches with [REDACTED]
}
```

---

## Network Proxy

### Config Section

```toml
[network]
proxy = "http://proxy.example.com:8080"
proxy_https = "https://proxy.example.com:8080"
no_proxy = ["localhost", "127.0.0.1", ".internal.corp.com"]
```

### Implementation

When building the `reqwest::Client`:

```rust
let mut builder = reqwest::Client::builder()
    .no_proxy();  // start clean

if let Some(proxy_url) = &config.network.proxy {
    if let Ok(proxy) = reqwest::Proxy::http(proxy_url) {
        if !config.network.no_proxy.is_empty() {
            proxy = proxy.no_proxy(&reqwest::NoProxy::from_string(
                &config.network.no_proxy.join(",")
            ));
        }
        builder = builder.proxy(proxy);
    }
}
```

Also respect the standard `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY` environment variables as fallback if `[network]` section is not configured.

---

## TUI Error Display

Errors are shown as in-line notifications in the TUI:

```
┌─────────────────────────────────────────────────────┐
│  ⚠ Tool 'nikto' exited with code 1. See logs.      │  ← toast notification
├─────────────────────────────────────────────────────┤
│                                                     │
│  [conversation continues...]                        │
│                                                     │
```

Toast notifications auto-dismiss after 10 seconds or on next user input. Critical errors (session dir unwritable) block startup entirely and print to stderr before exit.

---

## User Stories

| ID | Title | Description |
|----|-------|-------------|
| US-7.1 | Tool crash doesn't crash Raskolnikov | As a user, if sqlmap crashes, Raskolnikov logs the error and continues the session |
| US-7.2 | Connection error shown gracefully | As a user, if my AI provider is unreachable, I see a clear message — not a panic |
| US-7.3 | Proxy support for API calls | As a user behind a corporate proxy, I can configure proxy settings and AI calls work |
| US-7.4 | API keys never exposed | As a user, I can verify that my API keys never appear in logs, transcripts, or the TUI |
| US-7.5 | Shell metacharacters blocked | As a user, if I (or the AI) try to inject shell metacharacters in a target, the command is rejected |

---

## Acceptance Criteria

| ID | Criterion | How to Verify |
|----|-----------|---------------|
| AC-7.1 | Missing tool shows warning on startup, doesn't crash | Uninstall a tool, launch |
| AC-7.2 | Tool crash during session shows error notification, session continues | Run a tool that exits non-zero |
| AC-7.3 | AI provider timeout shows error after retry | Set wrong base_url, observe |
| AC-7.4 | Invalid JSON from provider shows warning, fallback requested | Mock provider returning bad JSON |
| AC-7.5 | Session dir unwritable shows clear error, exits | `chmod 000` on sessions dir, launch |
| AC-7.6 | Config file parse error uses defaults + warning | Write invalid TOML, launch |
| AC-7.7 | Target with `;` is rejected | Send a target containing `;`, verify rejection |
| AC-7.8 | Target with `` ` `` is rejected | Same with backtick |
| AC-7.9 | API key not present in session.log | Grep log for key patterns |
| AC-7.10 | API key not present in conversation.md | Grep transcript for key patterns |
| AC-7.11 | `[network] proxy` used for reqwest calls | Set proxy, check HTTP traffic (integration test) |
| AC-7.12 | `HTTP_PROXY` env var used as fallback | Set env var, no config, verify proxy used |
| AC-7.13 | 30-min tool timeout kills process | (Mock timeout value to seconds for testing) |

---

## Order of Implementation

1. `src/error.rs` — unified error types with thiserror
2. Target sanitisation utility function
3. Integrate sanitisation into executor (before command build)
4. API key loading from env (already partially done in Phase 1/3 — audit and harden)
5. Key redaction in logger
6. Network proxy support in reqwest client builder
7. TUI toast notification component for errors
8. Wire error handling into agent shell (wrap all operations)
9. Test each error scenario
