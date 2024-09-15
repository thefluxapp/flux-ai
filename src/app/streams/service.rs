use anyhow::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::state::AppState;

pub async fn update_streams(state: &AppState, stream: Stream) -> Result<(), Error> {
    let AppState { jetstream, .. } = state;

    let kv = jetstream.get_key_value("streams-summarize-queue").await?;
    let key = stream.id.to_string();

    match kv.entry(&key).await? {
        Some(entry) => {
            let prev: Stream = serde_json::from_slice(&entry.value)?;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Stream {
    pub id: Uuid,
    pub updated_at: DateTime<Utc>,
}
