use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::PoolConfig;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub pool_config: PoolConfig,
    pub oracle: String,
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TransactMsg {
    AddLiquidity {},
    Deposit {},
    WithdrawInterest {},
    Withdraw {
        amount: Uint128,
    },
    Borrow {
        amount: Uint128,
    },
    Repay {},
    Liquidate {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ExecuteMsg {
    Transact(TransactMsg),
    UpdateUserAssetInfo {
        user_addr: String, 
    },
    UpdateAsset {
        denom: String,
        decimals: u16,
        target_utilization_rate_bps: u32,
        min_rate: u32,
        optimal_rate: u32,
        max_rate: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum QueryMsg {
    Assets {},
    UserAssetsInfo {
        user: String,
    },
    UserAssetInfo {
        user: String,
        denom: String,
    },
    UserData {
        user: String,
    },
    AssetInfo {
        denom: String,
    },
    AssetsInfo {},
    MaxLiquidationAmount {
        user: String,
    },
    GetOwner {},
    GetTotalAssetAvailable {},
    GetTotalCollateralAvailable {},
    GetUserPrinciple { user: String },
    GetUserPrincipleToRepay { user: String },
}
