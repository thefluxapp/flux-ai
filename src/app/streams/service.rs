use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Error};
use async_nats::jetstream::kv::{Operation, Store};
use bytes::BytesMut;
use flux_auth_api::{users_service_client::UsersServiceClient, GetUsersRequest};
use flux_core_api::{
    messages_service_client::MessagesServiceClient, GetMessagesRequest, SummarizeStreamResponse,
};
use futures_lite::StreamExt;
use prost::Message;
use summarize_stream::SummarizeStreamRequest;
use tokio::task::JoinSet;
use tonic::transport::Channel;

use crate::app::{clients::AppClients, ollama::OllamaClient, AppJS};

use super::settings::StreamsSettings;

pub async fn summarize_streams(
    kv: Store,
    js: Arc<AppJS>,
    settings: StreamsSettings,
    clients: Arc<AppClients>,
    ollama: Arc<OllamaClient>,
) -> Result<(), Error> {
    let keys = kv.keys().await?.take(settings.execution.batch_size);
    tokio::pin!(keys);

    let mut join_set = JoinSet::new();

    while let Some(key) = keys.next().await {
        match kv.entry(&key?).await? {
            Some(entry) => {
                if entry.operation == Operation::Put {
                    let request: SummarizeStreamRequest = serde_json::from_slice(&entry.value)?;

                    join_set.spawn(summarize_streams_process(
                        kv.clone(),
                        js.clone(),
                        settings.clone(),
                        clients.clone(),
                        ollama.clone(),
                        request,
                    ));
                }
            }
            None => {}
        }
    }

    join_set.join_all().await;

    Ok(())
}

async fn summarize_streams_process(
    kv: Store,
    js: Arc<AppJS>,
    settings: StreamsSettings,
    clients: Arc<AppClients>,
    ollama: Arc<OllamaClient>,
    request: SummarizeStreamRequest,
) -> Result<(), Error> {
    let key = request.stream_id.clone();

    let AppClients {
        users_service_client,
        messages_service_client,
        ..
    } = clients.as_ref();

    let prompt =
        summarize_streams_collect(&request, users_service_client, messages_service_client).await?;

    let response = SummarizeStreamResponse {
        stream_id: Some(request.stream_id),
        text: Some(ollama.summarize(prompt).await?),
        version: Some(request.version),
    };

    let mut buf = BytesMut::new();
    response.encode(&mut buf)?;

    if let Some(entry) = kv.entry(&key).await? {
        if entry.operation == Operation::Put {
            let prev: SummarizeStreamRequest = serde_json::from_slice(&entry.value)?;

            if prev.version <= request.version {
                kv.delete_expect_revision(&key, Some(entry.revision))
                    .await?;

                js.publish(settings.messaging.subjects.response, buf.into())
                    .await?;
            }
        }
    };

    Ok(())
}

async fn summarize_streams_collect(
    request: &SummarizeStreamRequest,
    users_service_client: &UsersServiceClient<Channel>,
    messages_service_client: &MessagesServiceClient<Channel>,
) -> Result<Vec<String>, Error> {
    let messages: HashMap<String, String> = messages_service_client
        .to_owned()
        .get_messages(GetMessagesRequest {
            message_ids: request
                .messages
                .iter()
                .map(|m| m.message_id.clone())
                .collect(),
        })
        .await?
        .into_inner()
        .messages
        .iter()
        .map(|v| (v.message_id().into(), v.text().into()))
        .collect();

    let users: HashMap<String, String> = users_service_client
        .to_owned()
        .get_users(GetUsersRequest {
            user_ids: request.messages.iter().map(|m| m.user_id.clone()).collect(),
        })
        .await?
        .into_inner()
        .users
        .iter()
        .map(|v| (v.user_id().into(), v.first_name().into()))
        .collect();

    let prompt = request
        .messages
        .iter()
        .map(|m| -> Result<String, Error> {
            let user = users
                .get(m.user_id.as_str())
                .ok_or(anyhow!("message not found"))?;
            let text = messages
                .get(m.message_id.as_str())
                .ok_or(anyhow!("user nort found"))?;

            Ok(user.to_owned() + ": " + text)
        })
        .collect::<Result<Vec<String>, Error>>()?;

    Ok(prompt)
}

pub async fn summarize_stream(
    kv: Store,
    request: summarize_stream::SummarizeStreamRequest,
) -> Result<(), Error> {
    let key = request.stream_id.clone();

    while let Err(_) = summarize_stream_process(&kv, key.clone(), &request).await {
        println!("QQQ");
    }

    Ok(())
}

async fn summarize_stream_process(
    kv: &Store,
    key: String,
    request: &SummarizeStreamRequest,
) -> Result<(), Error> {
    let entry = kv.entry(&key).await?;

    let rev = match entry {
        Some(ref e) => {
            if e.operation == Operation::Put {
                let prev: SummarizeStreamRequest = serde_json::from_slice(&e.value)?;

                if prev.version > request.version {
                    return Ok(());
                }
            }

            e.revision
        }
        None => 0,
    };

    kv.update(&key, serde_json::to_vec(&request)?.into(), rev)
        .await?;

    Ok(())
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
