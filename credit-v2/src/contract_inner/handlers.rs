pub mod errors;
pub mod interfaces;
pub mod operation;
pub mod types;

use crate::abi::{Message, Operation};
use crate::interfaces::{runtime::contract::ContractRuntimeContext, state::StateInterface};
use errors::HandlerError;
use interfaces::Handler;
use operation::transfer::TransferHandler;

pub struct HandlerFactory;

impl HandlerFactory {
    fn new_operation_handler(
        runtime: impl ContractRuntimeContext + 'static,
        state: impl StateInterface + 'static,
        op: &Operation,
    ) -> Box<dyn Handler> {
        match op {
            Operation::Transfer { from, to, amount } => {
                Box::new(TransferHandler::new(runtime, state, from, to, amount))
            }
            _ => unimplemented!(),
        }
    }

    fn new_message_handler(
        runtime: impl ContractRuntimeContext,
        state: impl StateInterface,
        msg: &Message,
    ) -> Box<dyn Handler> {
        unimplemented!()
    }

    pub fn new(
        runtime: impl ContractRuntimeContext + 'static,
        state: impl StateInterface + 'static,
        op: Option<&Operation>,
        msg: Option<&Message>,
    ) -> Result<Box<dyn Handler>, HandlerError> {
        if let Some(op) = op {
            return Ok(HandlerFactory::new_operation_handler(runtime, state, op));
        }
        if let Some(msg) = msg {
            return Ok(HandlerFactory::new_message_handler(runtime, state, msg));
        }
        Err(HandlerError::InvalidOperationAndMessage)
    }
}
