use crate::instantiation_argument::InstantiationArgument;
use async_graphql::{Request, Response};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{AccountOwner, Amount, ApplicationId, ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

pub struct CreditAbi;

pub type OperationResponse = ();

impl ContractAbi for CreditAbi {
    type Operation = Operation;
    type Response = OperationResponse;
}

impl ServiceAbi for CreditAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    Liquidate,
    Transfer {
        from: AccountOwner,
        to: AccountOwner,
        amount: Amount,
    },
    TransferExt {
        to: AccountOwner,
        amount: Amount,
    },
    SetRewardCallers {
        application_ids: Vec<ApplicationId>,
    },
    SetTransferCallers {
        application_ids: Vec<ApplicationId>,
    },
    RequestSubscribe,
    Reward {
        owner: AccountOwner,
        amount: Amount,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Message {
    InstantiationArgument {
        argument: InstantiationArgument,
    },
    Liquidate,
    Reward {
        owner: AccountOwner,
        amount: Amount,
    },
    Transfer {
        from: AccountOwner,
        to: AccountOwner,
        amount: Amount,
    },
    TransferExt {
        to: AccountOwner,
        amount: Amount,
    },
    SetRewardCallers {
        application_ids: Vec<ApplicationId>,
    },
    SetTransferCallers {
        application_ids: Vec<ApplicationId>,
    },
    RequestSubscribe,
}

