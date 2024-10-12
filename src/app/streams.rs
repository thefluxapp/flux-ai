use anyhow::Error;
use async_nats::jetstream::consumer::pull::Config;

use super::state::AppState;

mod execution;
mod messaging;
mod service;
pub(crate) mod settings;

pub async fn messaging(state: &AppState) -> Result<(), Error> {
    let AppState { js, settings, .. } = state;

    let consumer = js
        .create_consumer_on_stream(
            Config {
                durable_name: Some(settings.streams.messaging.consumer.clone()),
                filter_subject: settings.streams.messaging.subjects.request.clone(),
                ..Default::default()
            },
            settings.streams.messaging.name.clone(),
        )
        .await?;

    // Консюмер молча падает в момент иниализации , а приложка грузится дальше,
    // нужно найти способ пропагейтить ошибку в main тред чтобы приложка не поднималась
    tokio::spawn(messaging::summarize_stream_consumer(
        state.clone(),
        consumer.clone(),
    ));

    Ok(())
}

pub async fn execution(state: &AppState) -> Result<(), Error> {
    tokio::spawn(execution::summarize_streams_execution(state.clone()));

    Ok(())
}
