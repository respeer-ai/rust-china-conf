#![cfg_attr(target_arch = "wasm32", no_main)]

use std::{cell::RefCell, rc::Rc};

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
    state: Rc<RefCell<CreditState>>,
    runtime: Rc<RefCell<ContractRuntime<Self>>>,
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
        CreditContract {
            state: Rc::new(RefCell::new(state)),
            runtime: Rc::new(RefCell::new(runtime)),
        }
    }

    async fn instantiate(&mut self, argument: InstantiationArgument) {
        self.runtime.borrow_mut().application_parameters();
        self.state.borrow_mut().instantiate(argument);
    }

    async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
        self.on_op(&operation)
    }

    async fn execute_message(&mut self, message: Message) {
        self.on_message(&message)
    }

    async fn store(mut self) {
        self.state
            .borrow_mut()
            .save()
            .await
            .expect("Failed to save state");
    }
}

mod contract_impl;
