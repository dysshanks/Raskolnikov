# Phase 3 — AI Provider System

**Objective:** Implement all 8 AI providers with a shared `Provider` trait. The provider system detects available providers at startup, resolves the default, and exposes a uniform `chat()` interface that the agent shell uses.

---

## Files to Create / Modify

| File | Status | Purpose |
|------|--------|---------|
| `src/ai/mod.rs` | Create | `Provider` trait, `Message`/`ProviderResponse` types, `resolve_provider()` function |
| `src/ai/ollama.rs` | Create | Ollama HTTP client (`/api/chat`) |
| `src/ai/openai.rs` | Create | OpenAI-compatible client (used by OpenAI, Groq, Llama API, Together) |
| `src/ai/anthropic.rs` | Create | Anthropic Messages API client |
| `src/ai/openrouter.rs` | Create | OpenRouter API client |
| `src/ai/nous.rs` | Create | Nous API client (Hermes models) |
| `src/config.rs` | Modify | Add provider/model fields, env var reading for API keys |

---

## Data Structures (src/ai/mod.rs)

```rust
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

pub struct Message {
    pub role: Role,
    pub content: String,
    pub name: Option<String>,  // tool name for tool role messages
}

pub struct ProviderResponse {
    pub content: String,
    pub finish_reason: String, // "stop", "length", "tool_calls"
}

pub struct ProviderConfig {
    pub name: &'static str,
    pub env_key: Option<&'static str>,   // env var name for API key
    pub default_model: &'static str,
    pub default_base_url: Option<&'static str>,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn config(&self) -> &ProviderConfig;
    async fn chat(&self, messages: &[Message]) -> Result<ProviderResponse>;
}
```

---

## Provider Resolution Order

```
1. CLI --provider flag                      (highest priority)
2. config.toml [ai].provider
3. AUTO-DETECT (first available):
   a. Ollama  — check http://localhost:11434/api/tags
   b. Anthropic   — check ANTHROPIC_API_KEY env var
   c. OpenAI      — check OPENAI_API_KEY env var
   d. OpenRouter  — check OPENROUTER_API_KEY env var
   e. Groq        — check GROQ_API_KEY env var
   f. Nous        — check NOUS_API_KEY env var
   g. Llama API   — check LLAMA_API_KEY env var
   h. Together    — check TOGETHER_API_KEY env var
```

`resolve_provider(config: &Config) -> Box<dyn Provider>` iterates this order and returns the first match.

---

## Provider Details

### Ollama

| Field | Value |
|-------|-------|
| Endpoint | `{host}/api/chat` (default `http://localhost:11434`) |
| Method | POST |
| Request body | `{"model": "{model}", "messages": [...], "stream": false}` |
| Response | `{"message": {"role": "assistant", "content": "..."}}` |
| Auth | None |
| Default model | `qwen3` |
| Config section | `[ollama] host = "http://localhost:11434"` |

Available models checked via `GET /api/tags` → `{"models": [{"name": "qwen3", ...}]}`.

### OpenAI (shared client for OpenAI, Groq, Llama API, Together)

| Field | Value |
|-------|-------|
| Endpoint | `{base_url}/chat/completions` |
| Method | POST |
| Request body | `{"model": "{model}", "messages": [...]}` |
| Response | `{"choices": [{"message": {"content": "..."}}]}` |
| Auth | `Authorization: Bearer {api_key}` |
| Default models per variant | See spec tables |

Config sections for each: `[openai]`, `[groq]`, `[llama_api]`, `[together]` each with their own `base_url`.

The `openai.rs` module is parameterised by `base_url` and `api_key` — instantiated 4 times by the provider resolution system.

### Anthropic

| Field | Value |
|-------|-------|
| Endpoint | `https://api.anthropic.com/v1/messages` |
| Method | POST |
| Headers | `x-api-key: {key}`, `anthropic-version: 2023-06-01` |
| Request body | Anthropic-specific format: system as top-level key, messages array with role+content |
| Response | `{"content": [{"type": "text", "text": "..."}]}` |
| Auth | `x-api-key` header |
| Default model | `claude-sonnet-4-6` |

Anthropic uses a different message format — system prompt is a separate top-level field, not a message with role "system". The provider must translate.

### OpenRouter

| Field | Value |
|-------|-------|
| Endpoint | `https://openrouter.ai/api/v1/chat/completions` |
| Method | POST |
| Headers | `Authorization: Bearer {key}`, `HTTP-Referer: https://github.com/raskolnikov-security/raskolnikov`, `X-Title: Raskolnikov` |
| Request body | OpenAI-compatible format |
| Response | OpenAI-compatible format |
| Default model | `meta-llama/llama-3.3-70b-instruct` |

### Nous

| Field | Value |
|-------|-------|
| Endpoint | `https://inference-api.nousresearch.com/v1/chat/completions` |
| Method | POST |
| Headers | `Authorization: Bearer {key}` |
| Request body | OpenAI-compatible format |
| Response | OpenAI-compatible format |
| Default model | `hermes-3-llama-3.1-70b` |

---

## Context Window Management

Implemented in this phase as a utility used by the agent shell (Phase 5):

1. Token counting: approximate via `content.len() / 4` (rough char-to-token ratio for most models)
2. Threshold: 80% of the model's reported or configured context window
3. When exceeded: summarise the oldest tool output blocks (not operator/agent messages)
4. Summarised block replaced with: `[Tool output summarised: nmap found 3 open ports...]`
5. Raw output always preserved on disk in session tools dir
6. Operator shown a one-line warning: `⚠ Context at 85% — older tool outputs summarised`

---

## User Stories

| ID | Title | Description |
|----|-------|-------------|
| US-3.1 | Ollama auto-detected | As a user with Ollama running, `rsk` detects it automatically and sets it as default provider |
| US-3.2 | Cloud provider via env var | As a user, setting `ANTHROPIC_API_KEY` makes Anthropic available without any config file |
| US-3.3 | Provider override via CLI | As a user, `rsk --provider openai --model gpt-4o` overrides auto-detection |
| US-3.4 | Provider override via config | As a user, setting `provider = "groq"` in `config.toml` persists my preference |
| US-3.5 | Provider unavailable | As a user, if a specified provider is unavailable, I see a clear error with available options |
| US-3.6 | Chat sends and receives | As the agent shell developer, I can call `provider.chat(messages)` and get a text response |
| US-3.7 | Context summarisation | As a user in a long session, I'm warned when context fills up and old tool outputs get summarised |

---

## Acceptance Criteria

| ID | Criterion | How to Verify |
|----|-----------|---------------|
| AC-3.1 | Provider resolution picks Ollama when running on localhost:11434 | Start Ollama, launch `rsk` without --provider, check log shows "Resolved provider: ollama" |
| AC-3.2 | Provider resolution picks Anthropic when ANTHROPIC_API_KEY is set (no Ollama) | Unset Ollama, set ANTHROPIC_API_KEY, launch |
| AC-3.3 | `--provider openai --model gpt-4o` forces OpenAI provider | Launch with flags |
| AC-3.4 | `config.toml` with `[ai] provider = "groq"` sets Groq as default | Create config, launch |
| AC-3.5 | Ollama `chat()` returns valid response with qwen3 | Write test calling the provider directly |
| AC-3.6 | OpenAI `chat()` returns valid response with gpt-4o-mini | Write test with mock HTTP server |
| AC-3.7 | Anthropic `chat()` translates message format correctly | Write test verifying request body structure |
| AC-3.8 | Context summarisation triggers at 80% threshold | Fill conversation with enough tokens, verify summarisation runs |
| AC-3.9 | Operator and agent messages are never summarised | Verify summarised content is only tool outputs |
| AC-3.10 | All 8 providers listed in `rsk help` or startup output | Launch, check provider list |

---

## Architecture Decision

**No SDK dependencies.** All providers use raw `reqwest` HTTP calls with serde for JSON. This keeps the binary small and avoids version conflicts. Each provider module defines its own request/response structs.

The `openai.rs` module is generic — it takes `base_url` and `api_key` as constructor parameters. Groq, Llama API, and Together all instantiate it with their respective config:

```rust
// In provider resolution
ProviderConfig {
    name: "groq",
    env_key: Some("GROQ_API_KEY"),
    default_model: "llama-3.3-70b-versatile",
    default_base_url: Some("https://api.groq.com/openai/v1"),
}
```

Anthropic gets a dedicated module because its API format is different (system prompt as top-level param, content blocks array).

---

## Order of Implementation

1. `src/ai/mod.rs` — data types, Provider trait, resolve_provider()
2. `src/ai/ollama.rs` — simplest, no auth needed
3. `src/ai/openai.rs` — reusable for 4 providers
4. `src/ai/anthropic.rs` — different format
5. `src/ai/openrouter.rs` — mostly OpenAI-compat with extra headers
6. `src/ai/nous.rs` — OpenAI-compat with specific defaults
7. Context window management utility
8. Wire into config.rs (env var keys, provider config sections)
9. Unit tests with wiremock for each provider
10. Manual test with real Ollama
