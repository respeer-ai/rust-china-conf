use linera_sdk::linera_base_types::Amount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct InstantiationArgument {
    pub initial_supply: Amount,
    pub amount_alive_ms: u64,
}
