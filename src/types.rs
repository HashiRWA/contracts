use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Timestamp};
use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoolConfig {
    pub pool_name: String,
    pub pool_symbol: String,
    pub pool_maturation_date: u64,
    pub pool_debt_interest_rate: Uint128,
    pub pool_lend_interest_rate: Uint128,
    pub token0_address: Addr,
    pub token1_address: Addr,
}