use std::sync::Arc;

use anyhow::Error;
use async_nats::jetstream;

use super::{ollama::OllamaClient, settings::AppSettings, AppJS};

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub js: Arc<AppJS>,
    pub ollama: Arc<OllamaClient>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let nats = async_nats::connect(&settings.nats.endpoint).await?;
        let js = Arc::new(jetstream::new(nats));
        let ollama = Arc::new(OllamaClient::new(settings.ollama.clone()));

        Ok(Self {
            settings,
            js,
            ollama,
        })
    }
}
