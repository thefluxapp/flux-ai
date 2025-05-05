use flux_lib::error::Error;
use futures_lite::StreamExt as _;
use tracing::error;

use crate::app::state::AppState;

pub async fn message(state: AppState) -> Result<(), Error> {
    let AppState { js, settings, .. } = state.clone();

    let consumer = message::consumer(&js, &settings).await?;
    // let consumer = js
    //     .create_consumer_on_stream(
    //         Config {
    //             durable_name: Some(settings.streams.messaging.message.consumer.clone()),
    //             filter_subjects: settings.streams.messaging.message.subjects.clone(),
    //             ..Default::default()
    //         },
    //         settings.nats.stream.clone(),
    //     )
    //     .await?;

    let mut messages = consumer.messages().await?;

    // let msgs = consumer.messages().await?;
    // tokio::pin!(msgs);

    while let Some(message) = messages.next().await {
        if let Err(err) = message::handler(state.clone(), message?).await {
            error!("{}", err);
        }
    }

    // while let Some(msg) = msgs.next().await {
    //     if let Err(err) = async {
    //         let msg = msg.map_err(Error::msg)?;

    //         let flux_messages_api::Message { message, stream } =
    //             flux_messages_api::Message::decode(msg.payload.clone())?;

    //         msg.ack().await.map_err(Error::msg)?;

    //         Ok::<(), Error>(())
    //     }
    //     .await
    //     {
    //         error!("{}", err);
    //     }
    // }

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

// pub async fn summarize_stream_consumer(
//     state: AppState,
//     consumer: PullConsumer,
// ) -> Result<(), Error> {
//     loop {
//         if let Err(e) = summarize_stream(&state, &consumer).await {
//             println!("Error: {}", e.0)
//         }
//     }
// }

// async fn summarize_stream(state: &AppState, consumer: &PullConsumer) -> Result<(), AppError> {
//     let AppState { settings, js, .. } = state;

//     let kv = js.get_key_value(&settings.streams.kv.name).await?;

//     let messages = consumer.messages().await?;
//     tokio::pin!(messages);

//     while let Some(message) = messages.try_next().await? {
//         let request = SummarizeStreamRequest::decode(message.payload.clone())?;

//         match service::summarize_stream(kv.clone(), request.try_into()?).await {
//             Ok(()) => message.ack().await.map_err(Error::msg)?,
//             Err(e) => println!("ErroRR: {}", e),
//         };
//     }

//     Ok(())
// }

// impl TryFrom<SummarizeStreamRequest> for service::summarize_stream::SummarizeStreamRequest {
//     type Error = Error;

//     fn try_from(request: SummarizeStreamRequest) -> Result<Self, Self::Error> {
//         let data = Self {
//             stream_id: request.stream_id().into(),
//             version: request.version(),
//             messages: request
//                 .messages
//                 .iter()
//                 .map(|v| service::summarize_stream::Message {
//                     message_id: v.message_id().into(),
//                     user_id: v.user_id().into(),
//                 })
//                 .collect(),
//         };

//         Ok(data)
//     }
// }
