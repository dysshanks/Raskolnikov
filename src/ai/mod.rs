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

pub(crate) fn http_client() -> reqwest::Client {
    use std::sync::OnceLock;
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client")
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
/// Never touches system, user, or assistant messages.
pub fn summarise_context(messages: &mut [Message]) -> u32 {
    let mut count = 0;
    for msg in messages.iter_mut() {
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
}
