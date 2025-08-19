use linera_sdk::{
    ContractRuntime,
    linera_base_types::{AccountOwner, ChainId, Timestamp, Message},
};

pub trait ContractRuntimeContext: BaseRuntimeContext {
    fn authenticated_signer(&self) -> Option<AccountOwner>;
    fn send_message(&self, destionation: ChainId, message: Message);
}

pub struct ContractRuntimeAdapter<T> {
    runtime: ContractRuntime<T>;
}
