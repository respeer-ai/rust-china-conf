use linera_sdk::{
    ContractRuntime,
    linera_base_types::{AccountOwner, ChainId, Timestamp, Message},
};

pub trait BaseRuntimeContext {
    fn chain_id(&self) -> ChainId;
    fn system_time(&self) -> Timestamp;
}
