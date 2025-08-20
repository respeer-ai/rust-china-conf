use crate::CreditContract;
use credit_v2::abi::{CreditError, Message};
use credit_v2::instantiation_argument::InstantiationArgument;
use credit_v2::interfaces::state::StateInterface;
use linera_sdk::linera_base_types::{AccountOwner, Amount, ApplicationId, ChainId};

impl CreditContract {
    pub fn require_message_origin_chain_id(&mut self) -> Result<ChainId, CreditError> {
        match self.runtime.message_origin_chain_id() {
            Some(message_id) => Ok(message_id),
            None => Err(CreditError::InvalidMessageId),
        }
    }

    pub fn require_authenticated_signer(&mut self) -> Result<AccountOwner, CreditError> {
        match self.runtime.authenticated_signer() {
            Some(owner) => Ok(owner),
            None => Err(CreditError::InvalidSigner),
        }
    }

    pub fn on_op_liquidate(&mut self) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::Liquidate)
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    pub fn on_op_set_reward_callers(
        &mut self,
        application_ids: Vec<ApplicationId>,
    ) -> Result<(), CreditError> {
        if self.runtime.chain_id() != self.runtime.application_creator_chain_id() {
            return Err(CreditError::OperationNotAllowed);
        }
        self.runtime
            .prepare_message(Message::SetRewardCallers { application_ids })
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    pub fn on_op_set_transfer_callers(
        &mut self,
        application_ids: Vec<ApplicationId>,
    ) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::SetTransferCallers { application_ids })
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    pub fn on_op_transfer(
        &mut self,
        from: AccountOwner,
        to: AccountOwner,
        amount: Amount,
    ) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::Transfer { from, to, amount })
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    pub fn on_op_transfer_ext(
        &mut self,
        to: AccountOwner,
        amount: Amount,
    ) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::TransferExt { to, amount })
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    pub fn on_op_request_subscribe(&mut self) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::RequestSubscribe)
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    pub fn on_op_reward(&mut self, owner: AccountOwner, amount: Amount) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::Reward { owner, amount })
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    pub async fn on_msg_instantiation_argument(
        &mut self,
        arg: InstantiationArgument,
    ) -> Result<(), CreditError> {
        self.state.instantiate(arg);
        Ok(())
    }

    pub async fn on_msg_liquidate(&mut self) -> Result<(), CreditError> {
        self.state.liquidate(self.runtime.system_time()).await;
        Ok(())
    }

    pub async fn on_msg_reward(
        &mut self,
        owner: AccountOwner,
        amount: Amount,
    ) -> Result<(), CreditError> {
        self.state
            .reward(owner, amount, self.runtime.system_time())
            .await?;
        Ok(())
    }

    pub async fn on_msg_set_reward_callers(
        &mut self,
        application_ids: Vec<ApplicationId>,
    ) -> Result<(), CreditError> {
        if self.require_message_origin_chain_id()? != self.runtime.application_creator_chain_id() {
            return Err(CreditError::OperationNotAllowed);
        }
        self.state.set_reward_callers(application_ids.clone());
        Ok(())
    }

    pub async fn on_msg_set_transfer_callers(
        &mut self,
        application_ids: Vec<ApplicationId>,
    ) -> Result<(), CreditError> {
        if self.require_message_origin_chain_id()? != self.runtime.application_creator_chain_id() {
            return Err(CreditError::OperationNotAllowed);
        }
        self.state.set_transfer_callers(application_ids.clone());
        Ok(())
    }

    pub async fn on_msg_transfer(
        &mut self,
        from: AccountOwner,
        to: AccountOwner,
        amount: Amount,
    ) -> Result<(), CreditError> {
        self.state
            .transfer(from, to, amount, self.runtime.system_time())
            .await?;
        Ok(())
    }

    pub async fn on_msg_transfer_ext(
        &mut self,
        to: AccountOwner,
        amount: Amount,
    ) -> Result<(), CreditError> {
        let from = self.require_authenticated_signer()?;
        self.state
            .transfer(from, to, amount, self.runtime.system_time())
            .await?;
        Ok(())
    }

    pub async fn on_msg_request_subscribe(&mut self) -> Result<(), CreditError> {
        // The subscribe message must be from another chain
        Ok(())
    }
}
