use std::convert::TryInto;
use crate::external::query_price;

// Creating a Lending and Borrowing Protocol,
// where users can deposit assets and borrow other assets.
// User can lend anytime in the pool upto a maturity date.
// User can close their lending position only after maturity date.
// User can borrow anytime in the pool upto a maturity date.
// User can close their borrowing position anytime before the maturity date, (if not closed before maturity -> liquidation of collateral).

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, BankMsg, Coin, Addr,
};
// use oracle::msg::PriceResponse;

use crate::error::{ContractError, ContractResult};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::state::{POOL_CONFIG,  ASSETS, ASSET_INFO, ADMIN, UserAssetInfo, AssetConfig, AssetInfo, GLOBAL_DATA, GlobalData, RATE_DENOMINATOR, NANOSECONDS_IN_YEAR};
use crate::query::query_handler;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    POOL_CONFIG.save(deps.storage, &msg.pool_config)?;
    ADMIN.save(deps.storage, &deps.api.addr_validate(&msg.admin)?)?;
    ASSETS.save(deps.storage, &vec![])?;
    let liquidation_threshold = 70 * RATE_DENOMINATOR / 100; 
    let global_data = GlobalData {
        oracle: msg.oracle,
        liquidation_threshold,
    };
    GLOBAL_DATA.save(deps.storage, &global_data)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Deposit {} => {
            deposit(deps, env, info)
        },
        ExecuteMsg::Withdraw { denom, amount } => {
            withdraw(deps, env, info, denom, amount)
        },
        ExecuteMsg::DepositCollateral {} => {
            deposit_collateral(deps, env, info)
        },
        ExecuteMsg::WithdrawCollateral { denom, amount } => {
            withdraw_collateral(deps, env, info, denom, amount)
        },
        ExecuteMsg::Borrow { denom, amount } => {
            borrow(deps, env, info, denom, amount)
        },
        ExecuteMsg::Repay {} => {
            repay(deps, env, info)
        },
        ExecuteMsg::Liquidate { user_addr, denom } => {
            liquidate(deps, env, info, user_addr, denom)
        },
        ExecuteMsg::UpdateUserAssetInfo { user_addr } => {
            update_user_asset_info(deps, env, user_addr)
        },
        ExecuteMsg::UpdateAsset { denom, decimals, target_utilization_rate_bps, min_rate, optimal_rate, max_rate } => {
            update_asset(deps, env, info, denom, target_utilization_rate_bps, decimals, min_rate, optimal_rate, max_rate)
        }
    }
}

















#[entry_point]
pub fn quote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Deposit {} => {
            deposit(deps, env, info)
        },
        ExecuteMsg::Withdraw { denom, amount } => {
            withdraw(deps, env, info, denom, amount)
        },
        ExecuteMsg::DepositCollateral {} => {
            deposit_collateral(deps, env, info)
        },
        ExecuteMsg::WithdrawCollateral { denom, amount } => {
            withdraw_collateral(deps, env, info, denom, amount)
        },
        ExecuteMsg::Borrow { denom, amount } => {
            borrow(deps, env, info, denom, amount)
        },
        ExecuteMsg::Repay {} => {
            repay(deps, env, info)
        },
        ExecuteMsg::Liquidate { user_addr, denom } => {
            liquidate(deps, env, info, user_addr, denom)
        },
        ExecuteMsg::UpdateUserAssetInfo { user_addr } => {
            update_user_asset_info(deps, env, user_addr)
        },
        ExecuteMsg::UpdateAsset { denom, decimals, target_utilization_rate_bps, min_rate, optimal_rate, max_rate } => {
            update_asset(deps, env, info, denom, target_utilization_rate_bps, decimals, min_rate, optimal_rate, max_rate)
        }
    }
}







#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    query_handler(deps, env, msg)
}

