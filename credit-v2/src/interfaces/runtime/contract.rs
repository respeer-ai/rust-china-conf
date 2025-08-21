use linera_sdk::{
    linera_base_types::{AccountOwner, ChainId},
};
use crate::abi::Message;
use super::base::BaseRuntimeContext;

pub trait ContractRuntimeContext: BaseRuntimeContext {
    type Error;

    fn authenticated_signer(&self) -> Option<AccountOwner>;
    fn require_authenticated_signer(&self) -> Result<AccountOwner, Self::Error>;

    fn send_message(&self, destionation: ChainId, message: Message);

    fn message_origin_chain_id(&self) -> Option<ChainId>;
    fn require_message_origin_chain_id(&self) -> Result<ChainId, Self::Error>;
}

