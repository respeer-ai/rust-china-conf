pub mod errors;

use crate::abi::{Operation, Message};
use errors::HandlerError;
use crate::interfaces::{runtime::contract::ContractRuntimeContext, state::StateInterface};
use async_trait::async_trait;

#[derive(Debug, Default)]
pub struct HandlerOutcome {
    pub messages: Vec<Message>,
}

#[async_trait]
pub trait Handler {
    async fn handle(&mut self) -> Result<HandlerOutcome, HandlerError>;
}

pub struct HandlerFactory;

impl HandlerFactory {
    fn new_operation_handler(runtime: impl ContractRuntimeContext, mut state: impl StateInterface, op: &Operation) -> Result<Box<dyn Handler>, HandlerError> {
        unimplemented!()
    }

    fn new_message_handler(runtime: impl ContractRuntimeContext, mut state: impl StateInterface, msg: &Message) -> Result<Box<dyn Handler>, HandlerError> {
        unimplemented!()
    }

    pub fn new(runtime: impl ContractRuntimeContext, mut state: impl StateInterface, op: Option<&Operation>, msg: Option<&Message>) -> Result<Box<dyn Handler>, HandlerError> {
        if let Some(op) = op {
            return HandlerFactory::new_operation_handler(runtime, state, op)
        }
        if let Some(msg) = msg {
            return HandlerFactory::new_message_handler(runtime, state, msg)
        }
        Err(HandlerError::InvalidOperationAndMessage)
    }
}
