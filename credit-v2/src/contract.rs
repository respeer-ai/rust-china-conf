#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::CreditState;
use credit_v2::{CreditAbi, CreditError, InstantiationArgument, Message, Operation};
use linera_sdk::{
    linera_base_types::{AccountOwner, Amount, ApplicationId, ChainId, WithContractAbi},
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
        self.state.initialize_credit(argument).await;
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

impl CreditContract {
    fn require_message_origin_chain_id(&mut self) -> Result<ChainId, CreditError> {
        match self.runtime.message_origin_chain_id() {
            Some(message_id) => Ok(message_id),
            None => Err(CreditError::InvalidMessageId),
        }
    }

    fn require_authenticated_signer(&mut self) -> Result<AccountOwner, CreditError> {
        match self.runtime.authenticated_signer() {
            Some(owner) => Ok(owner),
            None => Err(CreditError::InvalidSigner),
        }
    }

    fn on_op_liquidate(&mut self) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::Liquidate)
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    fn on_op_set_reward_callers(
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

    fn on_op_set_transfer_callers(
        &mut self,
        application_ids: Vec<ApplicationId>,
    ) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::SetTransferCallers { application_ids })
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    fn on_op_transfer(
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

    fn on_op_transfer_ext(&mut self, to: AccountOwner, amount: Amount) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::TransferExt { to, amount })
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    fn on_op_request_subscribe(&mut self) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::RequestSubscribe)
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    fn on_op_reward(&mut self, owner: AccountOwner, amount: Amount) -> Result<(), CreditError> {
        self.runtime
            .prepare_message(Message::Reward { owner, amount })
            .with_authentication()
            .send_to(self.runtime.application_creator_chain_id());
        Ok(())
    }

    async fn on_msg_instantiation_argument(
        &mut self,
        arg: InstantiationArgument,
    ) -> Result<(), CreditError> {
        self.state.initialize_credit(arg).await;
        Ok(())
    }

    async fn on_msg_liquidate(&mut self) -> Result<(), CreditError> {
        self.state.liquidate(self.runtime.system_time()).await;
        Ok(())
    }

    async fn on_msg_reward(
        &mut self,
        owner: AccountOwner,
        amount: Amount,
    ) -> Result<(), CreditError> {
        self.state
            .reward(owner, amount, self.runtime.system_time())
            .await?;
        Ok(())
    }

    async fn on_msg_set_reward_callers(
        &mut self,
        application_ids: Vec<ApplicationId>,
    ) -> Result<(), CreditError> {
        if self.require_message_origin_chain_id()? != self.runtime.application_creator_chain_id() {
            return Err(CreditError::OperationNotAllowed);
        }
        self.state.set_reward_callers(application_ids.clone()).await;
        Ok(())
    }

    async fn on_msg_set_transfer_callers(
        &mut self,
        application_ids: Vec<ApplicationId>,
    ) -> Result<(), CreditError> {
        if self.require_message_origin_chain_id()? != self.runtime.application_creator_chain_id() {
            return Err(CreditError::OperationNotAllowed);
        }
        self.state
            .set_transfer_callers(application_ids.clone())
            .await;
        Ok(())
    }

    async fn on_msg_transfer(
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

    async fn on_msg_transfer_ext(
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

    async fn on_msg_request_subscribe(&mut self) -> Result<(), CreditError> {
        // The subscribe message must be from another chain
        Ok(())
    }
}
