use linera_sdk::{
    linera_base_types::{ChainId, Timestamp},
};

pub trait BaseRuntimeContext {
    fn chain_id(&self) -> ChainId;
    fn system_time(&self) -> Timestamp;
}
