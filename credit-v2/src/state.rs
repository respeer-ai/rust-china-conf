use crate::types::AgeAmounts;
use async_graphql::SimpleObject;
use linera_sdk::{
    linera_base_types::{AccountOwner, Amount, ApplicationId},
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

pub mod adapter;
pub mod errors;
pub mod state_impl;
