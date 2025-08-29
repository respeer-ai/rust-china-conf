#![cfg_attr(target_arch = "wasm32", no_main)]

use async_graphql::{EmptySubscription, Request, Response, Schema};
use leaderboard::abi::Operation;
use leaderboard::state::LeaderBoardState;
use linera_sdk::{
    graphql::GraphQLMutationRoot, linera_base_types::WithServiceAbi, views::View, Service,
    ServiceRuntime,
};
use std::sync::Arc;

pub struct LeaderBoardService {
    state: Arc<LeaderBoardState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(LeaderBoardService);

impl WithServiceAbi for LeaderBoardService {
    type Abi = leaderboard::abi::LeaderBoardAbi;
}

impl Service for LeaderBoardService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = LeaderBoardState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        LeaderBoardService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(
            self.state.clone(),
            Operation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .finish();
        schema.execute(request).await
    }
}
