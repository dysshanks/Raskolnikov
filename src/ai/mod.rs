pub mod anthropic;
pub mod nous;
pub mod ollama;
pub mod openai;
pub mod openrouter;

use crate::config;
use async_trait::async_trait;

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
}

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
