use async_graphql::{Request, Response};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{AccountOwner, Amount, ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

pub struct LeaderBoardAbi;

pub type OperationResponse = ();

impl ContractAbi for LeaderBoardAbi {
    type Operation = Operation;
    type Response = OperationResponse;
}

impl ServiceAbi for LeaderBoardAbi {
    type Query = Request;
    type QueryResponse = Response;
}

/// Can only be called from creation chain
#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    UpdateValue {
        /// It's called from another chain so we should record the caller
        owner: AccountOwner,
        value: Amount,
    },
}

/// No message needed
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Message {
    Unused,
}
