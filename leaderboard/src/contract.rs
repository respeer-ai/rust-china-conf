#![cfg_attr(target_arch = "wasm32", no_main)]

use std::{cell::RefCell, rc::Rc};

use leaderboard::{
    abi::{LeaderBoardAbi, Message, Operation},
    instantiation_argument::InstantiationArgument,
    interfaces::state::StateInterface,
    state::LeaderBoardState,
};
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

pub struct LeaderBoardContract {
    state: Rc<RefCell<LeaderBoardState>>,
    runtime: Rc<RefCell<ContractRuntime<Self>>>,
}

linera_sdk::contract!(LeaderBoardContract);

impl WithContractAbi for LeaderBoardContract {
    type Abi = LeaderBoardAbi;
}

impl Contract for LeaderBoardContract {
    type Message = Message;
    type InstantiationArgument = InstantiationArgument;
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = LeaderBoardState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        LeaderBoardContract {
            state: Rc::new(RefCell::new(state)),
            runtime: Rc::new(RefCell::new(runtime)),
        }
    }

    async fn instantiate(&mut self, argument: InstantiationArgument) {
        self.runtime.borrow_mut().application_parameters();
        self.state.borrow_mut().instantiate(argument);
    }

    async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
        self.on_op(&operation).await
    }

    async fn execute_message(&mut self, message: Message) {
        self.on_message(&message)
    }

    async fn store(self) {
        self.state
            .borrow_mut()
            .save()
            .await
            .expect("Failed to save state");
    }
}

mod contract_impl;
