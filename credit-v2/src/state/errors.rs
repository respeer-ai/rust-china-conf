use thiserror::Error;

/// An error that can occur during the contract execution.
#[derive(Debug, Error)]
pub enum StateError {
    /// Failed to deserialize BCS bytes
    #[error("Failed to deserialize BCS bytes")]
    BcsError(#[from] bcs::Error),

    /// Failed to deserialize JSON string
    #[error("Failed to deserialize JSON string")]
    JsonError(#[from] serde_json::Error),

    #[error("NOT IMPLEMENTED")]
    NotImplemented,

    #[error("Caller not allowed")]
    CallerNotAllowed,

    #[error("Operation not allowed")]
    OperationNotAllowed,

    #[error("Cross-application sessions not supported")]
    SessionsNotSupported,

    #[error("Insufficient account balance")]
    InsufficientAccountBalance,

    #[error("Invalid signer")]
    InvalidSigner,

    #[error("Invalid message id")]
    InvalidMessageId,

    #[error("View error")]
    ViewError(#[from] linera_sdk::views::ViewError),
}
