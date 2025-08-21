use linera_sdk::{
    ContractRuntime,
    linera_base_types::{AccountOwner, ChainId, Timestamp, Message},
};
use super::errors::RuntimeError;

pub struct ContractRuntimeAdapter<T> {
    runtime: Box<dyn ContractRuntime<T>>,
}

impl<T> ContractRuntimeContext for ContractRuntimeAdapter<T> {
    type Error = RuntimeError;

    fn authenticated_signer(&self) -> Option<AccountOwner> {
        self.runtime.authenticated_signer()
    }

    fn require_authenticated_signer(&self) -> Result<AccountOwner, RuntimeError> {
        self.runtime.authenticated_signer().ok_or(RuntimeError::InvalidAuthenticatedSigner)
    }

    fn send_message(&self, destionation: ChainId, message: Message) {
        self.runtime.send_message(destination, message)
    }

    fn message_origin_chain_id(&self) -> Option<ChainId> {
        self.runtime.message_oritin_chain_id()
    }

    fn require_message_origin_chain_id(&self) -> Result<ChainId, RuntimeError> {
        self.runtime.message_oritin_chain_id().ok_or(RuntimeError::InvalidMessageOriginChainId)
    }
}
