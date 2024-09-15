use anyhow::Error;
use async_nats::jetstream::consumer::PullConsumer;
use cloudevents::{Data, Event};
use futures_lite::StreamExt as _;

use crate::app::{
    state::AppState,
    streams::service::{self, Stream},
};

pub async fn update_streams_consumer(state: AppState, consumer: PullConsumer) -> Result<(), Error> {
    loop {
        if let Err(e) = update_streams(&state, &consumer).await {
            println!("Error: {}", e)
        }
    }
}

async fn update_streams(state: &AppState, consumer: &PullConsumer) -> Result<(), Error> {
    println!("SUB");

    let messages = consumer.messages().await?;
    tokio::pin!(messages);

    while let Some(message) = messages.try_next().await? {
        let event: Event = serde_json::from_slice(&message.payload)?;

        if let Some(Data::Json(data)) = event.data() {
            let stream: Stream = serde_json::from_value(data.to_owned())?;

            match service::update_streams(state, stream).await {
                Ok(()) => message.ack().await.map_err(Error::msg)?,
                Err(e) => println!("Error: {}", e),
            };
        }
    }

    Ok(())
}
