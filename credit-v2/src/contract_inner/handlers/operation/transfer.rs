use crate::{
    contract_inner::handlers::{errors::HandlerError, interfaces::Handler, types::HandlerOutcome},
    interfaces::{runtime::contract::ContractRuntimeContext, state::StateInterface},
};
use async_trait::async_trait;

use linera_sdk::linera_base_types::{AccountOwner, Amount};

pub struct TransferHandler<R: ContractRuntimeContext, S: StateInterface> {
    runtime: R,
    state: S,

    from: AccountOwner,
    to: AccountOwner,
    amount: Amount,
}

impl<R: ContractRuntimeContext, S: StateInterface> TransferHandler<R, S> {
    pub fn new(
        runtime: R,
        state: S,
        from: &AccountOwner,
        to: &AccountOwner,
        amount: &Amount,
    ) -> Self {
        Self {
            state,
            runtime,
            from: *from,
            to: *to,
            amount: *amount,
        }
    }
}

#[async_trait(?Send)]
impl<R: ContractRuntimeContext, S: StateInterface> Handler for TransferHandler<R, S> {
    async fn handle(&mut self) -> Result<HandlerOutcome, HandlerError> {
        unimplemented!()
    }
}
