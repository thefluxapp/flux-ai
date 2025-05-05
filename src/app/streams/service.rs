use async_nats::jetstream::kv::Operation;
use futures_lite::StreamExt;
use tokio::task::JoinSet;

use crate::app::{error::AppError, state::AppState};

pub async fn message(state: AppState, req: message::Request) -> Result<(), AppError> {
    let kv = state.streams.kv.clone();

    let Some(key) = req.message_id else {
        return Ok(());
    };

    let entry = kv.entry(&key).await?;

    let revision = match entry {
        Some(entry) => {
            if entry.operation == Operation::Put {
                let version = i64::from_ne_bytes(
                    entry
                        .value
                        .as_ref()
                        .try_into()
                        .map_err(|_| AppError::Other)?,
                );

                if version >= req.version {
                    return Ok(());
                }
            }

            entry.revision
        }
        None => 0,
    };

    kv.update(&key, req.version.to_ne_bytes().to_vec().into(), revision)
        .await?;

    Ok(())
}

pub mod message {
    pub struct Request {
        pub message_id: Option<String>,
        pub version: i64,
    }
}

pub async fn summarize(state: AppState) -> Result<(), AppError> {
    let AppState { settings, .. } = state.clone();

    let kv = state.streams.kv.clone();
    let keys = kv.keys().await?.take(settings.streams.execution.batch_cnt);
    let mut join_set = JoinSet::new();

    tokio::pin!(keys);

    while let Some(key) = keys.next().await {
        match kv.entry(&key?).await? {
            Some(entry) => {
                if entry.operation == Operation::Put {
                    join_set.spawn(summarize::process(
                        state.clone(),
                        summarize::Request {
                            message_id: entry.key,
                            version: i64::from_ne_bytes(
                                entry
                                    .value
                                    .as_ref()
                                    .try_into()
                                    .map_err(|_| AppError::Other)?,
                            ),
                        },
                    ));
                }
            }
            None => {}
        }
    }

    join_set.join_all().await;

    Ok(())
}

mod summarize {
    use std::collections::HashMap;

    use async_nats::jetstream::kv::Operation;
    use bytes::BytesMut;
    use chrono::{DateTime, Utc};
    use flux_messages_api::{get_message_response, GetMessageRequest};
    use flux_users_api::{get_users_response, GetUsersRequest};
    use prost::Message as _;

    use crate::app::{error::AppError, state::AppState};

    #[derive(Debug)]
    pub struct Request {
        pub message_id: String,
        pub version: i64,
    }

    pub struct Response {
        pub message_id: String,
        pub text: String,
        pub version: i64,
    }

    // TBD: maybe split this code?
    pub async fn process(state: AppState, req: Request) -> Result<(), AppError> {
        let AppState {
            clients,
            settings,
            ollama,
            js,
            ..
        } = state.clone();

        let kv = state.streams.kv.clone();
        let key = &req.message_id.clone();

        let messages: Vec<get_message_response::Message> = clients
            .messages_service_client
            .clone()
            .get_message(GetMessageRequest {
                message_id: Some(req.message_id.clone()),
                cursor_message_id: None,
                limit: Some(settings.streams.limit_cnt),
            })
            .await?
            .into_inner()
            .messages
            .into_iter()
            .map(|message| message)
            .collect();

        let mut user_ids: Vec<String> = messages.iter().map(|m| m.user_id().into()).collect();

        user_ids.dedup();

        let users: HashMap<String, get_users_response::User> = clients
            .users_service_client
            .clone()
            .get_users(GetUsersRequest { user_ids })
            .await?
            .into_inner()
            .users
            .into_iter()
            .map(|user| (user.user_id().into(), user))
            .collect();

        let prompt: Vec<String> = messages
            .iter()
            .filter_map(|message| prompt(message, users.get(message.user_id())).ok())
            .collect();

        let text = ollama.summarize(prompt.join("---\n\n")).await?;

        let mut buf = BytesMut::new();
        Into::<flux_ai_api::Stream>::into(Response {
            text,
            version: req.version,
            message_id: req.message_id,
        })
        .encode(&mut buf)?;

        if let Some(entry) = kv.entry(key).await? {
            if entry.operation == Operation::Put {
                let version = i64::from_ne_bytes(
                    entry
                        .value
                        .as_ref()
                        .try_into()
                        .map_err(|_| AppError::Other)?,
                );

                if version <= req.version {
                    kv.delete_expect_revision(key, Some(entry.revision)).await?;

                    js.publish(settings.streams.messaging.stream.subject, buf.into())
                        .await?;
                }
            }
        };

        Ok(())
    }

    fn prompt(
        message: &get_message_response::Message,
        user: Option<&get_users_response::User>,
    ) -> Result<String, AppError> {
        let user = user.ok_or(AppError::NoEntity)?;
        let timestamp = message
            .created_at
            .map(|ts| -> Result<DateTime<Utc>, AppError> {
                Ok(DateTime::from_timestamp(ts.seconds, 0).ok_or(AppError::Other)?)
            })
            .ok_or(AppError::NoEntity)??;

        // TBD: escape template characters in the text
        let prompt = format!(
            "[timestamp={timestamp}] [id={id}] [first_name={first_name}] [last_name={last_name}]:\n{text}\n",
            timestamp = timestamp,
            id = user.user_id(),
            first_name = user.first_name(),
            last_name = user.last_name(),
            text = message.text(),
        );

        Ok(prompt)
    }

    impl From<Response> for flux_ai_api::Stream {
        fn from(res: Response) -> Self {
            Self {
                message_id: Some(res.message_id),
                text: Some(res.text),
                version: Some(res.version),
            }
        }
    }
}

pub mod summarize_stream {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct SummarizeStreamRequest {
        pub stream_id: String,
        pub version: i64,
        pub messages: Vec<Message>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Message {
        pub message_id: String,
        pub user_id: String,
    }
}
