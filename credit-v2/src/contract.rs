#![cfg_attr(target_arch = "wasm32", no_main)]

use credit_v2::abi::{CreditAbi, Message, Operation};
use credit_v2::instantiation_argument::InstantiationArgument;
use credit_v2::interfaces::state::StateInterface;
use credit_v2::state::CreditState;
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

pub struct CreditContract {
    state: CreditState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(CreditContract);

impl WithContractAbi for CreditContract {
    type Abi = CreditAbi;
}

impl Contract for CreditContract {
    type Message = Message;
    type InstantiationArgument = InstantiationArgument;
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = CreditState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        CreditContract { state, runtime }
    }

    async fn instantiate(&mut self, argument: InstantiationArgument) {
        self.runtime.application_parameters();
        self.state.instantiate(argument);
    }

    async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
        match operation {
            Operation::Liquidate => self.on_op_liquidate().expect("Failed OP: liquidate"),
            Operation::SetRewardCallers { application_ids } => self
                .on_op_set_reward_callers(application_ids)
                .expect("Failed OP: set reward callers"),
            Operation::SetTransferCallers { application_ids } => self
                .on_op_set_transfer_callers(application_ids)
                .expect("Failed OP: set transfer callers"),
            Operation::Transfer { from, to, amount } => self
                .on_op_transfer(from, to, amount)
                .expect("Failed OP: transfer"),
            Operation::TransferExt { to, amount } => self
                .on_op_transfer_ext(to, amount)
                .expect("Failed OP: transfer from application"),
            Operation::RequestSubscribe => self
                .on_op_request_subscribe()
                .expect("Failed OP: subscribe"),
            Operation::Reward { owner, amount } => {
                self.on_op_reward(owner, amount).expect("Failed OP: reward")
            }
        }
    }

    async fn execute_message(&mut self, message: Message) {
        match message {
            Message::InstantiationArgument { argument } => self
                .on_msg_instantiation_argument(argument)
                .await
                .expect("Failed MSG: instantiation argument"),
            Message::Liquidate => self
                .on_msg_liquidate()
                .await
                .expect("Failed MSG: liquidate"),
            Message::Reward { owner, amount } => self
                .on_msg_reward(owner, amount)
                .await
                .expect("Failed MSG: reward"),
            Message::SetRewardCallers { application_ids } => self
                .on_msg_set_reward_callers(application_ids)
                .await
                .expect("Failed MSG: set reward callers"),
            Message::SetTransferCallers { application_ids } => self
                .on_msg_set_transfer_callers(application_ids)
                .await
                .expect("Failed MSG: set transfer callers"),
            Message::Transfer { from, to, amount } => self
                .on_msg_transfer(from, to, amount)
                .await
                .expect("Failed MSG: transfer"),
            Message::TransferExt { to, amount } => self
                .on_msg_transfer_ext(to, amount)
                .await
                .expect("Failed MSG: transfer from application"),
            Message::RequestSubscribe => self
                .on_msg_request_subscribe()
                .await
                .expect("Failed MSG: subscribe"),
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

mod contract_impl;
