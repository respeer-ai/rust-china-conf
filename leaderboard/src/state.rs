use async_graphql::SimpleObject;
use base::types::Candidate;
use linera_sdk::{
    linera_base_types::{AccountOwner, Amount, ChainId},
    views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext},
};

pub mod types;

use types::LeaderBoardItemValue;

#[derive(RootView, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct LeaderBoardState {
    pub _values: MapView<AccountOwner, LeaderBoardItemValue>,
    pub top_owners: MapView<AccountOwner, Amount>,

    pub _top_k: RegisterView<u8>,
    pub operator: RegisterView<Option<Candidate>>,
    pub caller: RegisterView<Option<ChainId>>,
}

pub mod adapter;
pub mod errors;
pub mod state_impl;
