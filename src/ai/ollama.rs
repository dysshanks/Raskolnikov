use super::{Message, Provider, ProviderResponse};
use async_trait::async_trait;

pub struct OllamaProvider {
    host: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(host: &str, model: &str) -> Self {
        Self {
            host: host.trim_end_matches('/').to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_connection(&self) -> bool {
        let url = format!("{}/api/tags", self.host);
        self.client
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    async fn chat(
        &self,
        messages: &[Message],
    ) -> Result<ProviderResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/api/chat", self.host);

        let ollama_messages: Vec<serde_json::Value> = messages
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
            "messages": ollama_messages,
            "stream": false,
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        let data: serde_json::Value = resp.json().await?;

        let content = data["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let finish_reason = data["done_reason"].as_str().unwrap_or("stop").to_string();

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
    async fn test_ollama_chat() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "qwen3",
                "message": {
                    "role": "assistant",
                    "content": "Hello from Ollama!"
                },
                "done_reason": "stop"
            })))
            .mount(&mock_server)
            .await;

        let provider = OllamaProvider::new(&mock_server.uri(), "qwen3");
        let messages = vec![Message::user("hello")];
        let response = provider.chat(&messages).await.unwrap();
        assert_eq!(response.content, "Hello from Ollama!");
        assert_eq!(response.finish_reason, "stop");
    }
}
