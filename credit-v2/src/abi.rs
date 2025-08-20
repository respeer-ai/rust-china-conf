use crate::instantiation_argument::InstantiationArgument;
use async_graphql::{Request, Response};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{AccountOwner, Amount, ApplicationId, ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct CreditAbi;

impl ContractAbi for CreditAbi {
    type Operation = Operation;
    type Response = ();
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

/// An error that can occur during the contract execution.
#[derive(Debug, Error)]
pub enum CreditError {
    /// Failed to deserialize BCS bytes
    #[error("Failed to deserialize BCS bytes")]
    BcsError(#[from] bcs::Error),

    /// Failed to deserialize JSON string
    #[error("Failed to deserialize JSON string")]
    JsonError(#[from] serde_json::Error),

    #[error("NOT IMPLEMENTED")]
    NotImplemented,

    #[error("Caller not allowed")]
    CallerNotAllowed,

    #[error("Operation not allowed")]
    OperationNotAllowed,

    #[error("Cross-application sessions not supported")]
    SessionsNotSupported,

    #[error("Insufficient account balance")]
    InsufficientAccountBalance,

    #[error("Invalid signer")]
    InvalidSigner,

    #[error("Invalid message id")]
    InvalidMessageId,

    #[error("View error")]
    ViewError(#[from] linera_sdk::views::ViewError),
}
