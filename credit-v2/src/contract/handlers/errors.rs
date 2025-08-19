use thiserror::Error;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("Only authenticated signer is allowed")]
    UnauthenticatedSigner,

    #[error("Permission denied")]
    PermissionDenied,
}
