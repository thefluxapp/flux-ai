use std::sync::Arc;

use anyhow::Error;
use async_nats::jetstream;

use super::settings::AppSettings;

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub jetstream: Arc<jetstream::Context>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let nats = async_nats::connect(&settings.nats.endpoint).await?;
        let jetstream = Arc::new(jetstream::new(nats));

        Ok(Self {
            settings,
            jetstream,
        })
    }
}
