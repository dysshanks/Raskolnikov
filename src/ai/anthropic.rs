use super::{Message, Provider, ProviderResponse};
use async_trait::async_trait;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    async fn chat(
        &self,
        messages: &[Message],
    ) -> Result<ProviderResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/v1/messages", self.base_url);

        let mut system_content = String::new();
        let mut api_messages = Vec::new();

        for msg in messages {
            match msg.role {
                super::Role::System => {
                    system_content = msg.content.clone();
                }
                _ => {
                    api_messages.push(serde_json::json!({
                        "role": match msg.role {
                            super::Role::User => "user",
                            super::Role::Assistant => "assistant",
                            _ => "user",
                        },
                        "content": msg.content,
                    }));
                }
            }
        }

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": api_messages,
            "max_tokens": 4096,
        });

        if !system_content.is_empty() {
            body["system"] = serde_json::json!(system_content);
        }

        let resp = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        let content = data["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .unwrap_or("")
            .to_string();

        let finish_reason = data["stop_reason"].as_str().unwrap_or("stop").to_string();

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
    async fn test_anthropic_chat() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "content": [{"type": "text", "text": "Hello from Claude!"}],
                "stop_reason": "end_turn"
            })))
            .mount(&mock_server)
            .await;

        let provider =
            AnthropicProvider::new(&mock_server.uri(), "sk-ant-test", "claude-sonnet-4-6");

        let messages = vec![
            Message::system("You are a helpful assistant"),
            Message::user("hello"),
        ];
        let response = provider.chat(&messages).await.unwrap();
        assert_eq!(response.content, "Hello from Claude!");
        assert_eq!(response.finish_reason, "end_turn");
    }
}
