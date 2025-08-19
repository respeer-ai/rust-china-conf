use async_graphql::SimpleObject;
use linera_sdk::linera_base_types::{Amount, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, SimpleObject)]
pub struct AgeAmount {
    pub amount: Amount,
    pub expired: Timestamp,
}

#[derive(Debug, Deserialize, Serialize, Clone, SimpleObject)]
pub struct AgeAmounts {
    pub amounts: Vec<AgeAmount>,
}

impl AgeAmounts {
    pub fn sum(&self) -> Amount {
        let mut _sum = Amount::ZERO;
        self.amounts
            .iter()
            .for_each(|a| _sum = _sum.try_add(a.amount).unwrap());
        _sum
    }
}
