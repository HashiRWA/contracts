use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Timestamp};
use cw_storage_plus::{Item, Map};


use crate::types::{PoolConfig, CoinConfig};


pub const POOL_CONFIG: Item<PoolConfig> = Item::new("pool_config");


pub const INTEREST_EARNED: Map<&Addr, Uint128> = Map::new("interest_earned");
pub const PRINCIPLE_DEPLOYED: Map<&Addr, (Uint128,Timestamp)> = Map::new("principle_deployed");

pub const INTEREST_TO_REPAY: Map<&Addr, Uint128> = Map::new("interest_to_repay");
pub const COLLATERAL_SUBMITTED: Map<&Addr, (Uint128,Timestamp)> = Map::new("collateral_deployed");
    
pub const PRINCIPLE_TO_REPAY: Map<&Addr, (Uint128,Timestamp)> = Map::new("principle_to_repay");

pub const ASSET_CONFIG: Item<CoinConfig> = Item::new("asset_config");
pub const COLLATERAL_CONFIG: Item<CoinConfig> = Item::new("collateral_config");

pub const TOTAL_ASSET_AVAILABLE: Item<Uint128> = Item::new("total_asset_available");
pub const TOTAL_COLLATERAL_AVAILABLE: Item<Uint128> = Item::new("total_collateral_available");
pub const TOTAL_PROTOCOL_EARNINGS: Item<Uint128> = Item::new("total_protocol_earnings");

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const NANOSECONDS_IN_YEAR: u64 = 365 * 24 * 60 * 60 * 1_000_000_000;
