use async_trait::async_trait;
use super::errors::HandlerError;
use super::types::HandlerOutcome;

#[async_trait(?Send)]
pub trait Handler {
    async fn handle(&mut self) -> Result<HandlerOutcome, HandlerError>;
}

