use anyhow::Error;
use chrono::Duration;
use futures_lite::StreamExt as _;
use tokio::time;
use tokio_stream::wrappers::IntervalStream;

use crate::app::{state::AppState, streams::service};

pub async fn summarize_streams_execution(state: AppState) -> Result<(), Error> {
    let AppState {
        settings,
        js,
        clients,
        ollama,
        ..
    } = state;

    let kv = js.get_key_value(&settings.streams.kv.name).await?;

    let interval_stream = IntervalStream::new(time::interval(
        Duration::milliseconds(settings.streams.execution.period_ms).to_std()?,
    ));
    tokio::pin!(interval_stream);

    while let Some(_) = interval_stream.next().await {
        if let Err(e) = service::summarize_streams(
            kv.clone(),
            js.clone(),
            settings.streams.clone(),
            clients.clone(),
            ollama.clone(),
        )
        .await
        {
            println!("Error: {}", e);
        }
    }

    Ok(())
}
