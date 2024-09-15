use anyhow::Error;
use axum::extract::State;
use chrono::Utc;
use cloudevents::{binding::nats::NatsCloudEvent, Event, EventBuilder, EventBuilderV10};
use serde_json::json;
use uuid::Uuid;

use crate::app::{state::AppState, AppError};

use super::service::Stream;

pub async fn send_example_message(
    State(AppState { jetstream, .. }): State<AppState>,
) -> Result<String, AppError> {
    let event = build_event().await?;

    let payload = NatsCloudEvent::from_event(event)?;

    jetstream
        .publish("foo.qwe", payload.payload.try_into()?)
        .await?;

    Ok("OK".into())
}

async fn build_event() -> Result<Event, Error> {
    let event = EventBuilderV10::new()
        .id(Uuid::now_v7().to_string())
        .ty("ty")
        .source("source")
        // .data("application/json", json!(Stream { id: Uuid::now_v7(), updated_at: Utc::now() }))
        .data(
            "application/json",
            json!(Stream {
                id: Uuid::parse_str("0191f010-174a-78e1-a312-ec822350b564")?,
                updated_at: Utc::now()
            }),
        )
        .build()?;

    Ok(event)
}
