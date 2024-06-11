use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Timestamp};
use cw_storage_plus::{Item, Map};


use crate::types::PoolConfig;

#[cw_serde]
pub struct UserAssetInfo {
    pub collateral_token_address: Addr,
    pub collateral_amount: Uint128,
    pub borrow_token_address: Addr,
    pub borrow_amount: Uint128,
    pub cumulative_interest: Uint128,
}

#[cw_serde]
pub struct PoolInfo {
    pub token1_address: Addr,
    pub token1_amount: Uint128,
    pub token2_address: Addr,
    pub token2_amount: Uint128,
}

pub const USER_ASSET_INFOS: Map<&Addr, UserAssetInfo> = Map::new("user_asset_infos");
pub const POOL_INFO: Item<PoolInfo> = Item::new("pool_info");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub interest_rate: Uint128,
    pub min_liqudation_ratio : Uint128
}


pub const USER_LENDING_INFOS: Map<&Addr, UserLendingInfo> = Map::new("user_lending_infos");
pub const USER_BORROWING_INFOS: Map<&Addr, UserBorrowingInfo> = Map::new("user_borrowing_infos");

pub const POOL_CONFIG: Item<PoolConfig> = Item::new("pool_config");
pub const ADMIN: Item<Addr> = Item::new("admin");
pub const NANOSECONDS_IN_YEAR: u64 = 365 * 24 * 60 * 60 * 1_000_000_000;

pub const CONFIG: Item<Config> = Item::new("config");