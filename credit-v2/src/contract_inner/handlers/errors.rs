use thiserror::Error;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("Invalid operation and message")]
    InvalidOperationAndMessage,
}
