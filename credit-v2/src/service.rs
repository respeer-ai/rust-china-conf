#![cfg_attr(target_arch = "wasm32", no_main)]

use async_graphql::{EmptySubscription, Request, Response, Schema};
use credit_v2::abi::Operation;
use credit_v2::state::CreditState;
use linera_sdk::{
    graphql::GraphQLMutationRoot, linera_base_types::WithServiceAbi, views::View, Service,
    ServiceRuntime,
};
use std::sync::Arc;

pub struct CreditService {
    state: Arc<CreditState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(CreditService);

impl WithServiceAbi for CreditService {
    type Abi = credit_v2::abi::CreditAbi;
}

impl Service for CreditService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = CreditState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        CreditService {
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
