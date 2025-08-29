use async_graphql::SimpleObject;
use linera_sdk::linera_base_types::{Amount, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize, Debug, SimpleObject)]
pub struct LeaderBoardItemValue {
    pub value: Amount,
    pub timestamp: Timestamp,
}
