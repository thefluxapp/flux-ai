use async_nats::jetstream::{
    context::KeyValueError,
    kv::{EntryError, UpdateError, WatchError, WatcherErrorKind},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("NO_ENTITY")]
    NoEntity,
    #[error(transparent)]
    Decode(#[from] prost::DecodeError),
    #[error(transparent)]
    Encode(#[from] prost::EncodeError),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    Kv(#[from] KeyValueError),
    #[error(transparent)]
    KvEntry(#[from] EntryError),
    #[error(transparent)]
    KvUpdate(#[from] UpdateError),
    #[error(transparent)]
    KvWatch(#[from] WatchError),
    #[error(transparent)]
    KvWatcher(#[from] async_nats::error::Error<WatcherErrorKind>),
    #[error(transparent)]
    Publish(#[from] async_nats::jetstream::context::PublishError),
    #[error(transparent)]
    Error(#[from] flux_lib::error::Error),
    #[error("OTHER")]
    Other,
}
