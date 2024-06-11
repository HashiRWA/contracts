use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoolConfig {
    pub pool_name: String,
    pub pool_symbol: String,
    pub pool_maturation_date: Timestamp,
    pub pool_debt_interest_rate: Uint128, // param to play for leverage / liquidity
    pub pool_strike_price: Uint128,
    pub pool_lend_interest_rate: Uint128,
    pub min_overcollateralization_factor: Uint128, // param to play for leverage / liquidity
    pub asset_address: Addr,
    pub collateral_address: Addr,
}
pub struct CoinConfig {
    pub address: Addr,
    pub denom: String,
    pub decimals: u16,
}