
use std::convert::TryInto;
use crate::external::query_price;
use crate::types::UserLendingInfo;

// Creating a Lending and Borrowing Protocol,
// where users can deposit assets and borrow other assets.
// User can lend anytime in the pool upto a maturity date.
// User can close their lending position only after maturity date.
// User can borrow¯ anytime in the pool upto a maturity date.
// User can close their borrowing position anytime before the maturity date, (if not closed before maturity -> liquidation of collateral).

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, BankMsg, Coin, Addr,
};
// use oracle::msg::PriceResponse;

use crate::error::{ContractError, ContractResult};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::state::{AssetConfig, AssetInfo, GlobalData, UserAssetInfo, ADMIN, ASSETS, ASSET_CONFIG, ASSET_INFO, GLOBAL_DATA, NANOSECONDS_IN_YEAR, POOL_CONFIG, RATE_DENOMINATOR, TOTAL_ASSET_AVAILABLE, USER_LENDING_INFOS};
use crate::query::query_handler;



// TODO : functions to be made
// quoteDeposit
// deposit

// TODO: Implement deposit function
// 1) firstly check if the pool has not matured, if the pool is matured revert the transaction
// 2) accept funds from user, and update user's lending_info vec with the new lending_info object (append)
// 3) add asset ammount to the pool's total asset amount

pub fn deposit(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {

    // get info of asset -> asset config
    let pool_config = POOL_CONFIG.load(deps.storage)?;
    let asset_config = ASSET_CONFIG.load(deps.storage)?;
    let now = env.block.time;

    // check if the pool has matured
    if now > pool_config.pool_maturation_date {
        return Err(ContractError::PoolMatured {});
    }

    // get the amount of asset to be deposited
    let amount = info.funds.iter().find(|coin| coin.denom == asset_config.denom).unwrap().amount;

    // get the user's lending info
    let user_lending_info = UserLendingInfo {
        amount,
        time: now,
        interest_rate: pool_config.pool_lend_interest_rate,
    };

    // update the user's lending info
    let mut user_lending_infos = USER_LENDING_INFOS.load(deps.storage, &info.sender)?;
    user_lending_infos.push(user_lending_info);
    USER_LENDING_INFOS.save(deps.storage, &info.sender, &user_lending_infos)?;

    // update the pool's total asset amount
    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    total_asset_available += amount;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    Ok(Response::default())
}
