use anyhow::Error;
use async_nats::jetstream::consumer::pull::Config;
use axum::{routing::get, Router};

use super::state::AppState;

mod http;
mod messaging;
mod service;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(http::send_example_message))
}

pub async fn messaging(state: &AppState) -> Result<(), Error> {
    let AppState { jetstream, .. } = state;

    let consumer = jetstream
        .create_consumer_on_stream(
            Config {
                durable_name: Some(state.settings.name.clone()),
                ..Default::default()
            },
            "foo",
        )
        .await?;

    // Консюмер молча падает в момент иниализации , а приложка грузится дальше,
    // нужно найти способ пропагейтить ошибку в main тред чтобы приложка не поднималась
    tokio::spawn(messaging::update_streams_consumer(state.clone(), consumer.clone()));

    Ok(())
}
