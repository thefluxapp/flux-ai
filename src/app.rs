use anyhow::Error;
use axum::{http::StatusCode, response::{IntoResponse, Response}, routing::get, Router};
use settings::AppSettings;
use state::AppState;

mod settings;
mod state;
mod streams;

pub async fn run() -> Result<(), Error> {
    let settings = AppSettings::new()?;
    let state = AppState::new(settings).await?;

    messaging(&state).await?;
    http(&state).await?;

    Ok(())
}

async fn http(state: &AppState) -> Result<(), Error> {
    let router = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/api/healthz", get(|| async {}))
                .nest("/streams", streams::router()),
        )
        .with_state(state.to_owned());

    let listener = tokio::net::TcpListener::bind(&state.settings.http.endpoint).await?;

    println!("START HTTP");

    axum::serve(listener, router).await?;

    Ok(())
}

async fn messaging(state: &AppState) -> Result<(), Error> {
    streams::messaging(&state).await?;

    Ok(())
}

// pub enum AppError {
//     BadRequest,
// }

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

        (status).into_response()
    }
}
