use super::CreditContract;

use credit_v2::{
    abi::{Message, Operation, OperationResponse},
    contract_inner::handlers::HandlerFactory,
    runtime::contract::ContractRuntimeAdapter,
};

impl CreditContract {
    pub fn on_op(&mut self, op: &Operation) -> OperationResponse {
        let runtime_context = ContractRuntimeAdapter::new(self.runtime.clone());
        let mut state_ref = self.state.borrow_mut();

        let rc = match HandlerFactory::new(runtime_context, &mut *state_ref, Some(op), None) {
            Ok(rc) => rc,
            Err(err) => panic!("Failed OP {:?}: {err}", op),
        };
        // TODO: if messages are available, send it
        // TODO: if events are available, emit it
        ()
    }

    pub fn on_message(&mut self, msg: &Message) {
        let runtime_context = ContractRuntimeAdapter::new(self.runtime.clone());
        let mut state_ref = self.state.borrow_mut();

        let rc = match HandlerFactory::new(runtime_context, &mut *state_ref, None, Some(msg)) {
            Ok(rc) => rc,
            Err(err) => panic!("Failed MSG {:?}: {err}", msg),
        };
        // TODO: if messages are available, send it
        // TODO: if events are available, emit it
    }
}
