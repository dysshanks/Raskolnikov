use super::{Message, Provider, ProviderResponse};
use async_trait::async_trait;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
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
            client: super::http_client(),
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

    async fn chat_stream(
        &self,
        messages: &[Message],
        tx: mpsc::UnboundedSender<String>,
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
            "stream": true,
        });

        let mut resp = self.client.post(&url).json(&body).send().await?;

        let mut full_content = String::new();
        let mut buffer = String::new();

        while let Some(chunk) = resp.chunk().await? {
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].to_string();
                buffer = buffer[newline_pos + 1..].to_string();
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(content) = data["message"]["content"].as_str() {
                        let _ = tx.send(content.to_string());
                        full_content.push_str(content);
                    }
                    if data["done"].as_bool().unwrap_or(false) {
                        let finish_reason =
                            data["done_reason"].as_str().unwrap_or("stop").to_string();
                        return Ok(ProviderResponse {
                            content: full_content,
                            finish_reason,
                        });
                    }
                }
            }
        }

        Ok(ProviderResponse {
            content: full_content,
            finish_reason: "stop".to_string(),
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
