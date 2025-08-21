use thiserror::Error;
use super::handlers::errors::HandlerError;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error(transparent)]
    HandlerError(#[from] HandlerError),
}
