use flux_lib::error::Error;
use futures_lite::StreamExt as _;
use tracing::error;

use crate::app::state::AppState;

pub async fn message(state: AppState) -> Result<(), Error> {
    let AppState { js, settings, .. } = state.clone();

    let consumer = message::consumer(&js, &settings).await?;
    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        if let Err(err) = message::handler(state.clone(), message?).await {
            error!("{}", err);
        }
    }

    Ok(())
}

mod message {
    use async_nats::jetstream::{
        self,
        consumer::{pull::Config, Consumer},
    };
    use flux_lib::error::Error;
    use prost::Message as _;

    use crate::app::{
        error::AppError,
        settings::AppSettings,
        state::AppState,
        streams::service::{self, message::Request},
        AppJS,
    };

    pub async fn consumer(js: &AppJS, settings: &AppSettings) -> Result<Consumer<Config>, Error> {
        Ok(js
            .create_consumer_on_stream(
                Config {
                    durable_name: Some(settings.streams.messaging.message.consumer.clone()),
                    filter_subjects: settings.streams.messaging.message.subjects.clone(),
                    ..Default::default()
                },
                settings.nats.stream.clone(),
            )
            .await?)
    }

    pub async fn handler(state: AppState, message: jetstream::Message) -> Result<(), Error> {
        service::message(state, message.clone().try_into()?).await?;

        message.ack().await.map_err(Error::msg)?;
        Ok(())
    }

    impl TryFrom<jetstream::Message> for Request {
        type Error = AppError;

        fn try_from(message: jetstream::Message) -> Result<Self, Self::Error> {
            let message = flux_messages_api::Message::decode(message.payload.clone())?;

            // TBD: nanos?
            let version = message.updated_at.ok_or(AppError::NoEntity)?.seconds;

            Ok(Self {
                message_id: message.stream.map(|stream| stream.message_id().into()),
                version,
            })
        }
    }
}
