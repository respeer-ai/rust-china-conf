use std::{cell::RefCell, rc::Rc};

use super::errors::StateError;
use crate::{
    abi::Message,
    state::CreditState,
    interfaces::state::StateInterface,
    instantiation_argument::InstantiationArgument,
};
use linera_sdk::{
    linera_base_types::{AccountOwner, ChainId, Timestamp, Amount, ApplicationId},
};
use async_trait::async_trait;

pub struct StateAdapter {
    state: Rc<RefCell<CreditState>>,
}

impl StateAdapter {
    pub fn new(state: Rc<RefCell<CreditState>>) -> Self {
        Self { state }
    }
}

#[async_trait(?Send)]
impl StateInterface for StateAdapter {
    type Error = StateError;

    fn instantiate(&mut self, argument: InstantiationArgument) {
        self.state.borrow_mut().instantiate(argument)
    }

    fn instantiation_argument(&self) -> InstantiationArgument {
        self.state.borrow().instantiation_argument()
    }

    fn initial_supply(&self) -> Amount {
        self.state.borrow().initial_supply()
    }

    async fn balance(&self, owner: AccountOwner) -> Amount {
        self.state.borrow().balance(owner).await
    }

    async fn reward(
        &mut self,
        owner: AccountOwner,
        amount: Amount,
        now: Timestamp,
    ) -> Result<(), Self::Error> {
        self.state.borrow_mut().reward(owner, amount, now).await
    }

    async fn liquidate(&mut self, now: Timestamp) {
        self.state.borrow_mut().liquidate(now).await
    }

    fn set_reward_callers(&mut self, application_ids: Vec<ApplicationId>) {
        self.state.borrow_mut().set_reward_callers(application_ids)
    }

    fn set_transfer_callers(&mut self, application_ids: Vec<ApplicationId>) {
        self.state.borrow_mut().set_transfer_callers(application_ids)
    }

    async fn transfer(
        &mut self,
        from: AccountOwner,
        to: AccountOwner,
        amount: Amount,
        now: Timestamp,
    ) -> Result<(), Self::Error> {
        self.state.borrow_mut().transfer(from, to, amount, now).await
    }
}
