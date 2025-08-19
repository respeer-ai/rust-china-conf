use crate::instantiation_argument::InstantiationArgument;
use async_trait::async_trait;
use linera_sdk::linera_base_types::{AccountOwner, Amount, ApplicationId, Timestamp};

#[async_trait]
pub trait StateInterface {
    type Error;

    fn instantiate(&mut self, argument: InstantiationArgument);
    fn instantiation_argument(&self) -> InstantiationArgument;
    fn initial_supply(&self) -> Amount;
    async fn balance(&self, owner: AccountOwner) -> Amount;
    async fn reward(
        &mut self,
        owner: AccountOwner,
        amount: Amount,
        now: Timestamp,
    ) -> Result<(), Self::Error>;
    async fn liquidate(&mut self, now: Timestamp);
    fn set_reward_callers(&mut self, application_ids: Vec<ApplicationId>);
    fn set_transfer_callers(&mut self, application_ids: Vec<ApplicationId>);
    async fn transfer(
        &mut self,
        from: AccountOwner,
        to: AccountOwner,
        amount: Amount,
        now: Timestamp,
    ) -> Result<(), Self::Error>;
}
