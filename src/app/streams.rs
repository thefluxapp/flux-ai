use super::state::AppState;

mod execution;
mod messaging;
mod service;
pub(crate) mod settings;
pub mod state;

pub fn messaging(state: &AppState) {
    tokio::spawn(messaging::message(state.clone()));
}

pub fn execution(state: &AppState) {
    tokio::spawn(execution::summarize(state.clone()));
}
