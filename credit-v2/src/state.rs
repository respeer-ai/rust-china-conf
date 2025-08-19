use std::cmp::Ordering;

use crate::interfaces::state::StateInterface;
use crate::{
    abi::CreditError,
    instantiation_argument::InstantiationArgument,
    types::{AgeAmount, AgeAmounts},
};
use async_graphql::SimpleObject;
use async_trait::async_trait;
use linera_sdk::{
    linera_base_types::{AccountOwner, Amount, ApplicationId, Timestamp},
    views::{linera_views, MapView, RegisterView, RootView, SetView, ViewStorageContext},
};

#[derive(RootView, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct CreditState {
    pub _initial_supply: RegisterView<Amount>,
    pub _balance: RegisterView<Amount>,
    pub amount_alive_ms: RegisterView<u64>,
    pub balances: MapView<AccountOwner, AgeAmounts>,
    pub spendables: MapView<AccountOwner, Amount>,
    pub reward_callers: SetView<ApplicationId>,
    pub transfer_callers: SetView<ApplicationId>,
}

#[async_trait]
impl StateInterface for CreditState {
    type Error = CreditError;

    fn instantiate(&mut self, mut argument: InstantiationArgument) {
        if argument.initial_supply.eq(&Amount::ZERO) {
            argument.initial_supply = Amount::from_tokens(100000000);
        }
        self._initial_supply.set(argument.initial_supply);
        self._balance.set(argument.initial_supply);
        self.amount_alive_ms.set(argument.amount_alive_ms);
    }

    fn instantiation_argument(&self) -> InstantiationArgument {
        InstantiationArgument {
            initial_supply: *self._initial_supply.get(),
            amount_alive_ms: *self.amount_alive_ms.get(),
        }
    }

    fn initial_supply(&self) -> Amount {
        *self._initial_supply.get()
    }

    async fn balance(&self, owner: AccountOwner) -> Amount {
        self.balances.get(&owner).await.unwrap().unwrap().sum()
    }

    async fn reward(
        &mut self,
        owner: AccountOwner,
        amount: Amount,
        now: Timestamp,
    ) -> Result<(), CreditError> {
        match self.spendables.get(&owner).await {
            Ok(Some(spendable)) => {
                self.spendables
                    .insert(&owner, spendable.saturating_add(amount))
                    .unwrap();
            }
            _ => {
                self.spendables.insert(&owner, amount).unwrap();
            }
        }

        match self._balance.get().cmp(&amount) {
            Ordering::Less => {
                log::error!(
                    "Here we should correct: supply balance {} reward amount {}",
                    self._balance.get(),
                    amount
                );
                // return Err(CreditError::InsufficientSupplyBalance)
            }
            _ => {}
        }

        self._balance
            .set(self._balance.get().saturating_sub(amount));

        match self.balances.get(&owner).await {
            Ok(Some(mut amounts)) => {
                amounts.amounts.push(AgeAmount {
                    amount,
                    expired: Timestamp::from(
                        now.micros().saturating_add(*self.amount_alive_ms.get()),
                    ),
                });
                match self.balances.insert(&owner, amounts) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(CreditError::ViewError(err)),
                }
            }
            _ => match self.balances.insert(
                &owner,
                AgeAmounts {
                    amounts: vec![AgeAmount {
                        amount,
                        expired: Timestamp::from(
                            now.micros().saturating_add(*self.amount_alive_ms.get()),
                        ),
                    }],
                },
            ) {
                Ok(_) => Ok(()),
                Err(err) => Err(CreditError::ViewError(err)),
            },
        }
    }

    async fn liquidate(&mut self, now: Timestamp) {
        let owners = self.balances.indices().await.unwrap();
        for owner in owners {
            let mut amounts = match self.balances.get(&owner).await {
                Ok(Some(amounts)) => amounts,
                _ => continue,
            };
            let mut spendable = match self.spendables.get(&owner).await {
                Ok(Some(spendable)) => spendable,
                _ => continue,
            };
            amounts.amounts.retain(|amount| {
                let expired = now.micros() > amount.expired.micros();
                if expired {
                    self._balance
                        .set(self._balance.get().saturating_add(amount.amount));
                    spendable = spendable.saturating_sub(amount.amount);
                }
                !expired
            });
            self.spendables.insert(&owner, spendable).unwrap();
            self.balances.insert(&owner, amounts).unwrap();
        }
    }

    fn set_reward_callers(&mut self, application_ids: Vec<ApplicationId>) {
        application_ids
            .iter()
            .for_each(|application_id| self.reward_callers.insert(application_id).unwrap())
    }

    fn set_transfer_callers(&mut self, application_ids: Vec<ApplicationId>) {
        application_ids
            .iter()
            .for_each(|application_id| self.transfer_callers.insert(application_id).unwrap())
    }

    async fn transfer(
        &mut self,
        from: AccountOwner,
        to: AccountOwner,
        amount: Amount,
        now: Timestamp,
    ) -> Result<(), CreditError> {
        match self.spendables.get(&from).await {
            Ok(Some(spendable)) => match spendable.cmp(&amount) {
                Ordering::Less => Err(CreditError::InsufficientAccountBalance),
                _ => {
                    self.spendables
                        .insert(&from, spendable.saturating_sub(amount))?;
                    let mut amounts = self.balances.get(&from).await.unwrap().unwrap();
                    let mut total: Amount = Amount::ZERO;
                    let mut remain: Option<AgeAmount> = None;
                    amounts.amounts.retain(|_amount| {
                        if total.ge(&amount) {
                            return true;
                        }
                        total = total.saturating_add(_amount.amount);
                        if total.ge(&amount) {
                            match total.try_sub(amount) {
                                Ok(result) => {
                                    remain = Some(AgeAmount {
                                        amount: result,
                                        expired: Timestamp::from(
                                            now.micros()
                                                .saturating_add(*self.amount_alive_ms.get()),
                                        ),
                                    })
                                }
                                _ => {}
                            }
                            return false;
                        }
                        return false;
                    });
                    match remain {
                        Some(result) => amounts.amounts.push(result),
                        _ => {}
                    }
                    self.balances.insert(&from, amounts).unwrap();
                    match self.balances.get(&to).await {
                        Ok(Some(mut amounts)) => {
                            amounts.amounts.push(AgeAmount {
                                amount,
                                expired: Timestamp::from(
                                    now.micros().saturating_add(*self.amount_alive_ms.get()),
                                ),
                            });
                            self.balances.insert(&to, amounts).unwrap();
                        }
                        _ => self
                            .balances
                            .insert(
                                &to,
                                AgeAmounts {
                                    amounts: vec![AgeAmount {
                                        amount,
                                        expired: Timestamp::from(
                                            now.micros()
                                                .saturating_add(*self.amount_alive_ms.get()),
                                        ),
                                    }],
                                },
                            )
                            .unwrap(),
                    }
                    match self.spendables.get(&to).await {
                        Ok(Some(spendable)) => self
                            .spendables
                            .insert(&to, spendable.saturating_add(amount))?,
                        _ => self.spendables.insert(&to, amount)?,
                    }
                    Ok(())
                }
            },
            _ => return Err(CreditError::InsufficientAccountBalance),
        }
    }
}
