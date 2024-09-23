use std::sync::Arc;

use anyhow::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OllamaClient {
    client: Arc<reqwest::Client>,
    settings: OllamaSettings,
}

impl OllamaClient {
    pub fn new(settings: OllamaSettings) -> Self {
        let client = Arc::new(reqwest::Client::new());

        Self { client, settings }
    }

    pub async fn summarize(&self, prompt: Vec<String>) -> Result<String, Error> {
        let endpoint = [self.settings.endpoint.clone(), "/api/generate".into()].concat();
        let req =
            OllamaRequest::new([self.settings.instruction.clone(), prompt.join("\n")].concat());

        let res = self
            .client
            .post(endpoint)
            .json(&req)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        Ok(res.response)
    }
}

#[derive(Deserialize, Clone)]
pub struct OllamaSettings {
    pub endpoint: String,
    pub instruction: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    pub model: OllamaModel,
    pub stream: bool,
    pub prompt: String,
    pub options: OllamaOptions,
}

impl OllamaRequest {
    pub fn new(prompt: String) -> Self {
        Self {
            model: OllamaModel::GEMMA2_2B,
            prompt,
            stream: false,
            options: OllamaOptions { temperature: 0.4 },
        }
    }
}

#[derive(Serialize)]
enum OllamaModel {
    #[serde(rename = "gemma2:2b")]
    GEMMA2_2B,
}

#[derive(Serialize)]
struct OllamaOptions {
    pub temperature: f32,
}

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    pub response: String,
    // pub done: bool,
}
