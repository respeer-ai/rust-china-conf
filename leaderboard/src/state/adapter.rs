use std::{cell::RefCell, rc::Rc};

use super::errors::StateError;
use super::types::LeaderBoardItemValue;
use crate::{
    instantiation_argument::InstantiationArgument, interfaces::state::StateInterface,
    state::LeaderBoardState,
};
use async_trait::async_trait;
use linera_sdk::linera_base_types::{AccountOwner, Amount, Timestamp};

pub struct StateAdapter {
    state: Rc<RefCell<LeaderBoardState>>,
}

impl StateAdapter {
    pub fn new(state: Rc<RefCell<LeaderBoardState>>) -> Self {
        Self { state }
    }
}

#[async_trait(?Send)]
impl StateInterface for StateAdapter {
    type Error = StateError;
    type ValueType = LeaderBoardItemValue;

    fn instantiate(&mut self, argument: InstantiationArgument) {
        self.state.borrow_mut().instantiate(argument)
    }

    fn instantiation_argument(&self) -> InstantiationArgument {
        self.state.borrow().instantiation_argument()
    }

    fn top_k(&self) -> u8 {
        self.state.borrow().top_k()
    }

    async fn value(&self, owner: AccountOwner) -> LeaderBoardItemValue {
        self.state.borrow().value(owner).await
    }

    fn update_value(
        &mut self,
        owner: AccountOwner,
        value: Amount,
        timestamp: Timestamp,
    ) -> Result<(), StateError> {
        self.state
            .borrow_mut()
            .update_value(owner, value, timestamp)
    }
}
