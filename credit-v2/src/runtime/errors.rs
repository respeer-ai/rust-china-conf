use thiserror::Error;
use linera_sdk::linera_base_types::AccountPermissionError;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    RuntimeAccountPermissionError(#[from] AccountPermissionError),
}
