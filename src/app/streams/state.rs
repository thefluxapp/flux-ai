use std::sync::Arc;

use async_nats::jetstream::kv::Store;

use crate::app::{error::AppError, AppJS};

use super::settings::StreamsSettings;

#[derive(Clone)]
pub struct StreamsState {
    pub kv: Arc<Store>,
}

impl StreamsState {
    pub async fn new(js: &AppJS, settings: &StreamsSettings) -> Result<Self, AppError> {
        let kv = Arc::new(js.get_key_value(&settings.kv.name).await?);

        Ok(Self { kv })
    }
}
