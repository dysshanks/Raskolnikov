pub mod anthropic;
pub mod nous;
pub mod ollama;
pub mod openai;
pub mod openrouter;

use crate::config;
use async_trait::async_trait;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub name: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
        }
    }

    pub fn tool(content: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            name: Some(name.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: String,
    pub finish_reason: String,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn chat(
        &self,
        messages: &[Message],
    ) -> Result<ProviderResponse, Box<dyn std::error::Error>>;

    async fn chat_stream(
        &self,
        messages: &[Message],
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<ProviderResponse, Box<dyn std::error::Error>> {
        let resp = self.chat(messages).await?;
        let _ = tx.send(resp.content.clone());
        Ok(resp)
    }
}

static HTTP_TIMEOUT_SECS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(60);

/// Sets the per-read timeout used by the shared HTTP client.
///
/// A value of 0 disables the timeout entirely. This must be called before any
/// provider request is made (it only affects client creation).
pub fn set_http_timeout(secs: u64) {
    HTTP_TIMEOUT_SECS.store(secs, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn http_client() -> reqwest::Client {
    use std::sync::OnceLock;
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            let secs = HTTP_TIMEOUT_SECS.load(std::sync::atomic::Ordering::Relaxed);
            let mut builder = reqwest::Client::builder();
            // A read timeout (not a total request timeout) so long streaming
            // responses are never cut off; only a stalled connection is.
            if secs > 0 {
                builder = builder.read_timeout(std::time::Duration::from_secs(secs));
            }
            builder.build().expect("Failed to create HTTP client")
        })
        .clone()
}

/// Rough token estimation (chars / 4)
pub fn estimate_tokens(text: &str) -> u32 {
    (text.len() / 4) as u32
}

/// Returns whether summarisation is needed and the total estimated tokens
pub fn check_context(messages: &[Message], context_window: u32) -> (bool, u32, f64) {
    let total: u32 = messages.iter().map(|m| estimate_tokens(&m.content)).sum();
    let ratio = total as f64 / context_window as f64;
    (ratio >= 0.80, total, ratio)
}

/// Summarise old tool messages by replacing verbose outputs with a short note.
/// Never touches system, user, or assistant messages. The last `keep_recent`
/// messages are left untouched so the model still sees the freshest output.
pub fn summarise_context(messages: &mut [Message], keep_recent: usize) -> u32 {
    let mut count = 0;
    let boundary = messages.len().saturating_sub(keep_recent);
    for (i, msg) in messages.iter_mut().enumerate() {
        if i >= boundary {
            break;
        }
        if let Role::Tool = msg.role {
            let lines: Vec<&str> = msg.content.lines().collect();
            if lines.len() > 10 {
                let summary = lines.iter().take(3).copied().collect::<Vec<_>>().join("\n");
                msg.content = format!(
                    "[Tool output summarised — {} lines]\n{}\n[...]",
                    lines.len(),
                    summary
                );
                count += 1;
            }
        }
    }
    count
}

#[derive(Debug, Clone)]
pub enum ProviderKind {
    Ollama(ollama::OllamaProvider),
    Anthropic(anthropic::AnthropicProvider),
    OpenAi(openai::OpenAiProvider),
    OpenRouter(openrouter::OpenRouterProvider),
    Nous(nous::NousProvider),
}

#[async_trait]
impl Provider for ProviderKind {
    fn name(&self) -> &'static str {
        match self {
            ProviderKind::Ollama(p) => p.name(),
            ProviderKind::Anthropic(p) => p.name(),
            ProviderKind::OpenAi(p) => p.name(),
            ProviderKind::OpenRouter(p) => p.name(),
            ProviderKind::Nous(p) => p.name(),
        }
    }

    async fn chat(
        &self,
        messages: &[Message],
    ) -> Result<ProviderResponse, Box<dyn std::error::Error>> {
        match self {
            ProviderKind::Ollama(p) => p.chat(messages).await,
            ProviderKind::Anthropic(p) => p.chat(messages).await,
            ProviderKind::OpenAi(p) => p.chat(messages).await,
            ProviderKind::OpenRouter(p) => p.chat(messages).await,
            ProviderKind::Nous(p) => p.chat(messages).await,
        }
    }

    async fn chat_stream(
        &self,
        messages: &[Message],
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<ProviderResponse, Box<dyn std::error::Error>> {
        match self {
            ProviderKind::Ollama(p) => p.chat_stream(messages, tx).await,
            ProviderKind::Anthropic(p) => p.chat_stream(messages, tx).await,
            ProviderKind::OpenAi(p) => p.chat_stream(messages, tx).await,
            ProviderKind::OpenRouter(p) => p.chat_stream(messages, tx).await,
            ProviderKind::Nous(p) => p.chat_stream(messages, tx).await,
        }
    }
}

pub fn resolve_provider(config: &config::Config) -> Option<ProviderKind> {
    let provider_name = &config.ai.provider;
    let keys = config::ApiKeys::from_env();

    match provider_name.as_str() {
        "auto" => {
            if let Some(key) = keys.anthropic {
                return Some(ProviderKind::Anthropic(anthropic::AnthropicProvider::new(
                    &config.anthropic.base_url,
                    &key,
                    &config.ai.model,
                )));
            }
            if let Some(key) = keys.openai {
                return Some(ProviderKind::OpenAi(openai::OpenAiProvider::new(
                    &config.openai.base_url,
                    &key,
                    &config.ai.model,
                    "openai",
                )));
            }
            if let Some(key) = keys.groq {
                return Some(ProviderKind::OpenAi(openai::OpenAiProvider::new(
                    &config.groq.base_url,
                    &key,
                    &config.ai.model,
                    "groq",
                )));
            }
            if let Some(key) = keys.openrouter {
                return Some(ProviderKind::OpenRouter(
                    openrouter::OpenRouterProvider::new(&key, &config.ai.model),
                ));
            }
            if let Some(key) = keys.nous {
                return Some(ProviderKind::Nous(nous::NousProvider::new(
                    &key,
                    &config.ai.model,
                )));
            }
            if let Some(key) = keys.llama {
                return Some(ProviderKind::OpenAi(openai::OpenAiProvider::new(
                    &config.llama_api.base_url,
                    &key,
                    &config.ai.model,
                    "llama-api",
                )));
            }
            if let Some(key) = keys.together {
                return Some(ProviderKind::OpenAi(openai::OpenAiProvider::new(
                    &config.together.base_url,
                    &key,
                    &config.ai.model,
                    "together",
                )));
            }
            Some(ProviderKind::Ollama(ollama::OllamaProvider::new(
                &config.ollama.host,
                &config.ai.model,
            )))
        }
        "ollama" => Some(ProviderKind::Ollama(ollama::OllamaProvider::new(
            &config.ollama.host,
            &config.ai.model,
        ))),
        "anthropic" => keys.anthropic.map(|key| {
            ProviderKind::Anthropic(anthropic::AnthropicProvider::new(
                &config.anthropic.base_url,
                &key,
                &config.ai.model,
            ))
        }),
        "openai" => keys.openai.map(|key| {
            ProviderKind::OpenAi(openai::OpenAiProvider::new(
                &config.openai.base_url,
                &key,
                &config.ai.model,
                "openai",
            ))
        }),
        "openrouter" => keys.openrouter.map(|key| {
            ProviderKind::OpenRouter(openrouter::OpenRouterProvider::new(&key, &config.ai.model))
        }),
        "groq" => keys.groq.map(|key| {
            ProviderKind::OpenAi(openai::OpenAiProvider::new(
                &config.groq.base_url,
                &key,
                &config.ai.model,
                "groq",
            ))
        }),
        "nous" => keys
            .nous
            .map(|key| ProviderKind::Nous(nous::NousProvider::new(&key, &config.ai.model))),
        "llama-api" => keys.llama.map(|key| {
            ProviderKind::OpenAi(openai::OpenAiProvider::new(
                &config.llama_api.base_url,
                &key,
                &config.ai.model,
                "llama-api",
            ))
        }),
        "together" => keys.together.map(|key| {
            ProviderKind::OpenAi(openai::OpenAiProvider::new(
                &config.together.base_url,
                &key,
                &config.ai.model,
                "together",
            ))
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_constructors() {
        let msg = Message::system("test");
        assert!(matches!(msg.role, Role::System));

        let msg = Message::user("hello");
        assert!(matches!(msg.role, Role::User));

        let msg = Message::assistant("hi");
        assert!(matches!(msg.role, Role::Assistant));

        let msg = Message::tool("output", "nmap");
        assert!(matches!(msg.role, Role::Tool));
        assert_eq!(msg.name, Some("nmap".to_string()));
    }

    #[test]
    fn test_http_timeout_default_is_60() {
        assert_eq!(
            HTTP_TIMEOUT_SECS.load(std::sync::atomic::Ordering::Relaxed),
            60
        );
    }

    #[test]
    fn test_set_http_timeout_updates_global() {
        set_http_timeout(120);
        assert_eq!(
            HTTP_TIMEOUT_SECS.load(std::sync::atomic::Ordering::Relaxed),
            120
        );
        set_http_timeout(60);
    }

    #[test]
    fn test_summarise_keeps_recent_tool_messages() {
        let mut messages = vec![
            Message::tool(
                "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11",
                "nmap",
            ),
            Message::assistant("ok"),
            Message::tool(
                "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11",
                "sqlmap",
            ),
        ];
        let count = summarise_context(&mut messages, 2);
        assert_eq!(count, 1);
        assert!(messages[0].content.contains("summarised"));
        assert!(!messages[2].content.contains("summarised"));
    }

    #[test]
    fn test_summarise_all_when_under_recent_boundary() {
        let mut messages = vec![
            Message::tool("x\nx\nx\nx\nx\nx\nx\nx\nx\nx\nx", "nmap"),
            Message::tool("y\ny\ny\ny\ny\ny\ny\ny\ny\ny\ny", "gobuster"),
        ];
        let count = summarise_context(&mut messages, 0);
        assert_eq!(count, 2);
    }
}
