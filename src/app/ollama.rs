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
        let req = OllamaRequest::new([self.settings.instruction.clone(), prompt].concat());

        dbg!(&req);

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

#[derive(Serialize, Debug)]
struct OllamaRequest {
    pub model: OllamaModel,
    pub stream: bool,
    pub prompt: String,
    pub options: OllamaOptions,
}

impl OllamaRequest {
    pub fn new(prompt: String) -> Self {
        Self {
            model: OllamaModel::GEMMA3_4B,
            prompt,
            stream: false,
            options: OllamaOptions { temperature: 0.3 },
        }
    }
}

#[derive(Serialize, Debug)]
enum OllamaModel {
    // #[serde(rename = "gemma2:2b")]
    // GEMMA2_2B,
    #[serde(rename = "gemma3:4b")]
    GEMMA3_4B,
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
