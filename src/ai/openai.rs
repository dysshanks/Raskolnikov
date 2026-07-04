use super::{Message, Provider, ProviderResponse};
use async_trait::async_trait;

pub struct OpenAiProvider {
    base_url: String,
    api_key: String,
    model: String,
    provider_name: &'static str,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str, name: &'static str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            provider_name: name,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn name(&self) -> &'static str {
        self.provider_name
    }

    async fn chat(
        &self,
        messages: &[Message],
    ) -> Result<ProviderResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/chat/completions", self.base_url);

        let openai_messages: Vec<serde_json::Value> = messages
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
            "messages": openai_messages,
        });

        let resp = self
            .client
            .post(&url)
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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_openai_chat() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Hello from OpenAI!"
                    },
                    "finish_reason": "stop"
                }]
            })))
            .mount(&mock_server)
            .await;

        let provider = OpenAiProvider::new(&mock_server.uri(), "sk-test", "gpt-4o", "openai");
        let messages = vec![Message::user("hello")];
        let response = provider.chat(&messages).await.unwrap();
        assert_eq!(response.content, "Hello from OpenAI!");
    }
}
