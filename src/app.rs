use anyhow::Error;
use async_nats::jetstream;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

use settings::AppSettings;
use state::AppState;
use tonic::service::Routes;

mod clients;
mod ollama;
mod settings;
mod state;
mod streams;

pub async fn run() -> Result<(), Error> {
    let settings = AppSettings::new()?;
    let state = AppState::new(settings).await?;

    messaging(&state).await?;
    execution(&state).await?;
    http_and_grpc(&state).await?;

    Ok(())
}

async fn http_and_grpc(state: &AppState) -> Result<(), Error> {
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .unwrap();

    let (_, health_service) = tonic_health::server::health_reporter();

    let router = Router::new()
        // .nest("/api", Router::new().route("/healthz", get(|| async {})))
        .with_state(state.to_owned());

    let routes: Routes = router.to_owned().into();
    let router = routes
        .add_service(reflection_service)
        .add_service(health_service)
        .into_axum_router();

    let listener = tokio::net::TcpListener::bind(&state.settings.http.endpoint).await?;

    axum::serve(listener, router).await?;

    Ok(())
}

async fn messaging(state: &AppState) -> Result<(), Error> {
    streams::messaging(&state).await?;

    Ok(())
}

async fn execution(state: &AppState) -> Result<(), Error> {
    streams::execution(state).await?;

    Ok(())
}

struct AppError(Error);

impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            _ => StatusCode::BAD_REQUEST,
        };

        (status, self.0.to_string()).into_response()
    }
}

pub type AppJS = jetstream::Context;
