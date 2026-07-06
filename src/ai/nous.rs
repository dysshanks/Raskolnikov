use super::{Message, Provider, ProviderResponse};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct NousProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl NousProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: super::http_client(),
        }
    }
}

#[async_trait]
impl Provider for NousProvider {
    fn name(&self) -> &'static str {
        "nous"
    }

    async fn chat(
        &self,
        messages: &[Message],
    ) -> Result<ProviderResponse, Box<dyn std::error::Error>> {
        let url = "https://inference-api.nousresearch.com/v1/chat/completions";

        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        super::Role::System => "system",
                        super::Role::User => "user",
                        super::Role::Assistant => "assistant",
                        super::Role::Tool => "tool",
                    },
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.model,
            "messages": api_messages,
        });

        let resp = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        let choice = &data["choices"][0];
        let content = choice["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let finish_reason = choice["finish_reason"]
            .as_str()
            .unwrap_or("stop")
            .to_string();

        Ok(ProviderResponse {
            content,
            finish_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nous_provider_name() {
        let provider = NousProvider::new("sk-nous-test", "hermes-3-llama-3.1-70b");
        assert_eq!(provider.name(), "nous");
        assert_eq!(provider.model, "hermes-3-llama-3.1-70b");
    }
}
