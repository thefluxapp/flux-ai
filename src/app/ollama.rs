use std::sync::Arc;

use flux_lib::error::Error;
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

    pub async fn summarize(&self, prompt: String) -> Result<String, Error> {
        let endpoint = [self.settings.endpoint.clone(), "/api/generate".into()].concat();
        let req = OllamaRequest::new(
            self.settings.model.clone(),
            [self.settings.instruction.clone(), prompt].concat(),
        );

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
    pub model: OllamaModel,
}

#[derive(Serialize, Debug)]
struct OllamaRequest {
    pub model: OllamaModel,
    pub stream: bool,
    pub prompt: String,
    pub options: OllamaOptions,
}

impl OllamaRequest {
    pub fn new(model: OllamaModel, prompt: String) -> Self {
        Self {
            model,
            prompt,
            stream: false,
            options: OllamaOptions { temperature: 0.3 },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OllamaModel {
    #[serde(rename = "gemma2:2b")]
    Gemma3_2b,
    #[serde(rename = "gemma3:4b")]
    Gemma3_4b,
    #[serde(rename = "gemma3:4b-it-qat")]
    Gemma3_4bQat,
}

#[derive(Serialize, Debug)]
struct OllamaOptions {
    pub temperature: f32,
}

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    pub response: String,
    // pub done: bool,
}
