use anyhow::Error;
use async_nats::jetstream::consumer::PullConsumer;
use flux_core_api::SummarizeStreamRequest;
use futures_lite::StreamExt as _;
use prost::Message;

use crate::app::{state::AppState, streams::service, AppError};

pub async fn summarize_stream_consumer(
    state: AppState,
    consumer: PullConsumer,
) -> Result<(), Error> {
    loop {
        if let Err(e) = summarize_stream(&state, &consumer).await {
            println!("Error: {}", e.0)
        }
    }
}

async fn summarize_stream(state: &AppState, consumer: &PullConsumer) -> Result<(), AppError> {
    let AppState { settings, js, .. } = state;

    let kv = js.get_key_value(&settings.streams.kv.name).await?;

    let messages = consumer.messages().await?;
    tokio::pin!(messages);

    while let Some(message) = messages.try_next().await? {
        let request = SummarizeStreamRequest::decode(message.payload.clone())?;

        match service::summarize_stream(kv.clone(), request.try_into()?).await {
            Ok(()) => message.ack().await.map_err(Error::msg)?,
            Err(e) => println!("ErroRR: {}", e),
        };
    }

    Ok(())
}

impl TryFrom<SummarizeStreamRequest> for service::summarize_stream::SummarizeStreamRequest {
    type Error = Error;

    fn try_from(request: SummarizeStreamRequest) -> Result<Self, Self::Error> {
        let data = Self {
            stream_id: request.stream_id().into(),
            version: request.version(),
            messages: request
                .messages
                .iter()
                .map(|v| service::summarize_stream::Message {
                    message_id: v.message_id().into(),
                    user_id: v.user_id().into(),
                })
                .collect(),
        };

        Ok(data)
    }
}
