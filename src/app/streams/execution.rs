use chrono::Duration;
use flux_lib::error::Error;
use futures_lite::StreamExt as _;
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tracing::error;

use crate::app::{state::AppState, streams::service};

// pub async fn summarize_streams_execution(state: AppState) -> Result<(), Error> {
//     let AppState {
//         settings,
//         js,
//         clients,
//         ollama,
//         ..
//     } = state;

//     let kv = js.get_key_value(&settings.streams.kv.name).await?;

//     let interval_stream = IntervalStream::new(time::interval(
//         Duration::milliseconds(settings.streams.execution.interval_ms).to_std()?,
//     ));
//     tokio::pin!(interval_stream);

//     while let Some(_) = interval_stream.next().await {
//         if let Err(e) = service::summarize_streams(
//             kv.clone(),
//             js.clone(),
//             settings.streams.clone(),
//             clients.clone(),
//             ollama.clone(),
//         )
//         .await
//         {
//             println!("Error: {}", e);
//         }
//     }

//     Ok(())
// }

pub async fn summarize(state: AppState) -> Result<(), Error> {
    let AppState { settings, .. } = state.clone();

    let mut interval = IntervalStream::new(time::interval(
        Duration::milliseconds(settings.streams.execution.interval_ms).to_std()?,
    ));

    while let Some(_) = interval.next().await {
        if let Err(err) = service::summarize(state.clone()).await {
            error!("{}", err);
        }
    }

    Ok(())
}
