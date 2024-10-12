use std::sync::Arc;

use anyhow::Error;
use async_nats::jetstream;

use super::{clients::AppClients, ollama::OllamaClient, settings::AppSettings, AppJS};

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub js: Arc<AppJS>,
    pub ollama: Arc<OllamaClient>,
    pub clients: Arc<AppClients>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let nats = async_nats::connect(&settings.nats.endpoint).await?;
        let js = Arc::new(jetstream::new(nats));
        let ollama = Arc::new(OllamaClient::new(settings.ollama.clone()));
        let clients = Arc::new(AppClients::new(settings.clients.clone()).await?);

        Ok(Self {
            settings,
            js,
            ollama,
            clients,
        })
    }
}
