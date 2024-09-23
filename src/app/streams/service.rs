use anyhow::Error;
use async_nats::jetstream;
use chrono::{DateTime, Utc};
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use futures_lite::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::app::{ollama::OllamaClient, settings::AppSettings, state::AppState, AppJS};

pub async fn summarize_streams(
    js: &AppJS,
    settings: &AppSettings,
    ollama: &OllamaClient,
) -> Result<(), Error> {
    let kv = js.get_key_value(&settings.streams.kv.name).await?;

    let keys = kv.keys().await?.take(settings.streams.execution.batch_size);
    tokio::pin!(keys);

    let mut join_set = JoinSet::new();

    while let Some(key) = keys.next().await {
        match kv.entry(&key?).await? {
            Some(entry) => {
                let request: SummarizeStreamRequest = serde_json::from_slice(&entry.value)?;

                join_set.spawn(summarize_stream(
                    js.clone(),
                    settings.clone(),
                    ollama.clone(),
                    request,
                    kv.clone(),
                ));
            }
            None => {}
        };
    }

    join_set.join_all().await;

    Ok(())
}

async fn summarize_stream(
    js: AppJS,
    settings: AppSettings,
    ollama: OllamaClient,
    stream: SummarizeStreamRequest,
    kv: jetstream::kv::Store,
) -> Result<(), Error> {
    let event: Event = SummarizeStreamResponse {
        id: stream.id,
        text: ollama.summarize(stream.prompt).await?,
    }
    .try_into()?;

    let key = stream.id.to_string();

    if let Some(entry) = kv.entry(&key).await? {
        let prev: SummarizeStreamRequest = serde_json::from_slice(&entry.value)?;

        if prev.updated_at >= stream.updated_at {
            kv.delete_expect_revision(&key, Some(entry.revision))
                .await?;

            js.publish(
                settings.streams.messaging.subjects.response,
                serde_json::to_vec(&event)?.try_into()?,
            )
            .await?;
        }
    };

    Ok(())
}

pub async fn update_stream(state: &AppState, stream: SummarizeStreamRequest) -> Result<(), Error> {
    let AppState { js, settings, .. } = state;

    let kv = js.get_key_value(&settings.streams.kv.name).await?;
    let key = stream.id.to_string();

    match kv.entry(&key).await? {
        Some(entry) => {
            let prev: SummarizeStreamRequest = serde_json::from_slice(&entry.value)?;

            if prev.updated_at < stream.updated_at {
                while let Err(_) = kv
                    .update(&key, serde_json::to_vec(&stream)?.into(), entry.revision)
                    .await
                {
                    println!("Can't UPDATE new KV");
                }
            }
        }
        None => {
            while let Err(_) = kv
                .update(&key, serde_json::to_vec(&stream)?.into(), 0)
                .await
            {
                println!("Can't create new KV");
            }
        }
    };

    Ok(())
}

#[derive(Deserialize, Serialize)]
pub struct SummarizeStreamRequest {
    pub id: Uuid,
    pub updated_at: DateTime<Utc>,
    pub prompt: Vec<String>,
}

#[derive(Serialize)]
struct SummarizeStreamResponse {
    pub id: Uuid,
    pub text: String,
}

impl TryFrom<SummarizeStreamResponse> for Event {
    type Error = Error;

    fn try_from(response: SummarizeStreamResponse) -> Result<Self, Self::Error> {
        let builder = EventBuilderV10::new()
            .id(Uuid::now_v7().to_string())
            .ty("ty")
            .data("application/json", json!(response))
            .source("source");

        Ok(builder.build()?)
    }
}
