use anyhow::{Context, Result};

/// Shared HTTP client for Ollama API calls.
pub struct OllamaClient {
    pub endpoint: String,
    pub model: String,
    client: reqwest::Client,
}

/// Parameters for a chat request to Ollama.
pub struct ChatRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub json_format: bool,
}

impl OllamaClient {
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            endpoint,
            model,
            client: reqwest::Client::new(),
        }
    }

    /// Send a chat request to the Ollama API and return the response content.
    pub async fn chat(&self, request: ChatRequest) -> Result<String> {
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": request.system_prompt },
                { "role": "user", "content": request.user_prompt }
            ],
            "stream": false,
            "options": {
                "temperature": request.temperature,
                "top_p": request.top_p
            }
        });

        if request.json_format {
            body["format"] = serde_json::json!("json");
        }

        let response = self.client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await
            .context("Failed to parse Ollama response")?;

        let content = response_json["message"]["content"]
            .as_str()
            .context("No content in Ollama response")?
            .to_string();

        Ok(content)
    }
}
