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
    pub pool_debt_interest_rate: Uint128,
    pub pool_lend_interest_rate: Uint128,
    pub min_overcollateralization_factor: Uint128,
    pub asset_address: Addr,
    pub collateral_address: Addr,
}


pub struct CoinConfig {
    pub address: Addr,
    pub denom: String,
    pub decimals: u16,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]

pub struct UserLendingInfo {
    pub amount: Uint128,
    pub time: Timestamp,
    pub interest_rate: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserBorrowingInfo {
    pub type_: String,
    pub amount: Uint128,
    pub time: Timestamp,
    pub collateral_submitted: Uint128,
    pub interest_rate: Uint128,
}