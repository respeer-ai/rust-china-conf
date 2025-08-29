pub mod errors;
pub mod interfaces;
pub mod operation;
pub mod types;

use crate::abi::{Message, Operation};
use crate::interfaces::{
    access_control::AccessControl, runtime::contract::ContractRuntimeContext, state::StateInterface,
};
use errors::HandlerError;
use interfaces::Handler;
use operation::update_value::UpdateValueHandler;

pub struct HandlerFactory;

impl HandlerFactory {
    fn new_operation_handler(
        runtime: impl ContractRuntimeContext + AccessControl + 'static,
        state: impl StateInterface + 'static,
        op: &Operation,
    ) -> Box<dyn Handler> {
        match op {
            Operation::UpdateValue { owner, value } => {
                Box::new(UpdateValueHandler::new(runtime, state, owner, value))
            }
        }
    }

    fn new_message_handler(
        _runtime: impl ContractRuntimeContext + AccessControl,
        _state: impl StateInterface,
        _msg: &Message,
    ) -> Box<dyn Handler> {
        unimplemented!()
    }

    pub fn new(
        runtime: impl ContractRuntimeContext + AccessControl + 'static,
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
