use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw20::Cw20ReceiveMsg;

use crate::types::PoolConfig;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub config: PoolConfig,
    pub oracle: String,
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TransactMsg {
    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
    Deposit (DepositMsg),
    Withdraw (WithdrawMsg),
    Loan (LoanMsg),
    Repay (RepayMsg),

    AddLiquidity {},
    WithdrawInterest {},
    
    Liquidate {},
}


#[cw_serde]
pub struct DepositMsg {
    pub denom : Addr,
    pub amount: Uint128,
}
#[cw_serde]
pub struct WithdrawMsg {
    pub denom : Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub struct RepayMsg {
    pub asset_denom : Addr,
    pub asset_principle: Uint128,
    pub collateral_denom : Addr,
}

#[cw_serde]
pub struct LoanMsg {
    pub asset_denom : Addr,
    pub asset_amount: Uint128,
    pub collateral_denom : Addr,
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
    PoolConfig {},
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

    GetDepositQuote {
        user: Addr,
        amount: Uint128,
    },

    GetWithdrawablePositions {
        user: Addr,
    },

    GetLoanQuote {
        amount: Uint128,
    },

    GetRepayablePositions {
        user: Addr,
    },

    GetRepayQuote {
        user:Addr
    },

}
