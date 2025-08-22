use async_trait::async_trait;
use crate::{
    interfaces::{
        runtime::contract::ContractRuntimeContext,
        state::StateInterface,
    },
    contract_inner::handlers::{
        types::HandlerOutcome,
        errors::HandlerError,
        interfaces::Handler,
    },
};

pub struct TransferHandler<R: ContractRuntimeContext, S: StateInterface> {
    runtime: R,
    state: S,
}

impl<R: ContractRuntimeContext, S: StateInterface> TransferHandler<R, S> {
    pub fn new(runtime: R, state: S) -> Self {
        Self { state, runtime }
    }
}

#[async_trait(?Send)]
impl<R: ContractRuntimeContext, S: StateInterface> Handler for TransferHandler<R, S> {
    async fn handle(&mut self) -> Result<HandlerOutcome, HandlerError> {
        unimplemented!()
    }
}
