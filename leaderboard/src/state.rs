use async_graphql::SimpleObject;
use linera_sdk::{
    linera_base_types::{AccountOwner, Amount},
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
}

pub mod adapter;
pub mod errors;
pub mod state_impl;
