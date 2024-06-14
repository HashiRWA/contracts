use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoolConfig {
    pub name: String,
    pub symbol: String,
    pub maturationdate: u64, 
    pub debtinterestrate: Uint128, 
    pub strikeprice: Uint128,
    pub lendinterestrate: Uint128,
    pub overcollateralizationfactor: Uint128, 
    pub asset: Addr,
    pub collateral: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoinConfig {
    pub denom: Addr,
    pub decimals: u16,
}

