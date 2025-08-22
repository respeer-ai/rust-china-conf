use super::CreditContract;

use credit_v2::{
    abi::{Message, Operation, OperationResponse},
    contract_inner::handlers::HandlerFactory,
    runtime::contract::ContractRuntimeAdapter,
    state::adapter::StateAdapter,
};

impl CreditContract {
    pub async fn on_op(&mut self, op: &Operation) -> OperationResponse {
        let runtime_context = ContractRuntimeAdapter::new(self.runtime.clone());
        let state_adapter = StateAdapter::new(self.state.clone());

        let outcome = match HandlerFactory::new(runtime_context, state_adapter, Some(op), None)
            .unwrap()
            .handle()
            .await
        {
            Ok(outcome) => outcome,
            Err(err) => panic!("Failed OP: {:?}: {err}", op),
        };

        // TODO: if messages are available, send it
        // TODO: if events are available, emit it
        ()
    }

    pub fn on_message(&mut self, msg: &Message) {
        let runtime_context = ContractRuntimeAdapter::new(self.runtime.clone());
        let state_adapter = StateAdapter::new(self.state.clone());

        let outcome = match HandlerFactory::new(runtime_context, state_adapter, None, Some(msg)) {
            Ok(outcome) => outcome,
            Err(err) => panic!("Failed MSG {:?}: {err}", msg),
        };
        // TODO: if messages are available, send it
        // TODO: if events are available, emit it
    }
}
