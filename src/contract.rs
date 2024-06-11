use std::convert::TryInto;
use std::iter::Map;
use crate::external::query_price;
use crate::types::{PoolConfig, UserLendingInfo};

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
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TransactMsg};

use crate::state::{AssetConfig, AssetInfo, GlobalData, UserAssetInfo, ADMIN, ASSETS, ASSET_CONFIG, ASSET_INFO, GLOBAL_DATA, NANOSECONDS_IN_YEAR, POOL_CONFIG, RATE_DENOMINATOR, TOTAL_ASSET_AVAILABLE, USERS_LENDING_INFOS};
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
    msg: TransactMsg,
) -> ContractResult<Response> {
    match msg {
        TransactMsg::Deposit {} => {
            deposit(deps, env, info)
        },
        TransactMsg::Withdraw { denom, amount } => {
            withdraw(deps, env, info, denom, amount)
        },
        TransactMsg::Borrow { denom, amount } => {
            borrow(deps, env, info, denom, amount)
        },
        TransactMsg::Repay {} => {
            repay(deps, env, info)
        },
        
        // TODO: to be executed by admin, to extract all the assets from the pool
        ExecuteMsg::Liquidate {} => {
            // liquidate(deps, env, info)
        },
    }
}



#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    query_handler(deps, env, msg)
}








// TODO: Implement deposit function
// 1) firstly check if the pool has not matured, if the pool is matured revert the transaction
// 2) accept funds from user, and update user's lending_info vec with the new lending_info object (append)
// 3) add asset ammount to the pool's total asset amount


fn deposit(
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



// TODO : verify this with madhav
fn calculate_interest_given_user_deposit_vector(user_lending_infos: Vec<UserLendingInfo>) -> Uint128 {
    // calculate the interest amount for the user given the position array
    // interest rate is interest for a year
    // all the positions are made in different times and have different time periods 
    // a time period is calculated as the difference between the maturity time and the time of deposit
    // calculate the interest for each position and sum all the interests
    let mut interest: Uint128 = Uint128::zero();
    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let maturity_time = pool_config.pool_maturation_date;
    for user_lending_info in user_lending_infos {
        let time_period = maturity_time - user_lending_info.time;
        let interest = user_lending_info.amount * user_lending_info.interest_rate * time_period / NANOSECONDS_IN_YEAR;
        interest += interest;
    }
    interest
}



// TODO: Implement withdraw function
// 0) user can only withdraw the whole amount all at once, after the pool has matured
// 1) firstly check if the pool has matured, if the pool has not matured, revert the transaction (user can only close the lending position after maturity time)
// 2) fetch user's position vector, sum all the amounts in the vector, and check if the amount to be withdrawn should be less than or equal to thetotal amount in the vector
// 3) calculate the interest amount for the user given the position array
// 4) check if the amount that user want's to withdraw + interest amount is less than or equal to the total asset amount in the pool
// 5) if all the conditions are satisfied, then update the user's position vector, and update the total asset amount in the pool
// 6) send the funds to the user

fn withdraw(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
) -> ContractResult<Response> {
    // get pool config
    let pool_config:PoolConfig = POOL_CONFIG.load(deps.storage)?;
    // check if the pool has matured
    if env.block.time < pool_config.pool_maturation_date {
        return Err(ContractError::PoolNotMatured {});
    }
    
    let total_asset_available_in_pool = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
  
    // get the user's lending info
    let users_lending_infos: Map<&Addr, Vec<UserLendingInfo>> = USERS_LENDING_INFOS.load(deps.storage, &info.sender)?;
    // get user lending info from the users lending info map
    let user_lending_infos : Vec<UserLendingInfo> = users_lending_infos.get(&info.sender).unwrap();

    let total_asset_lent = user_lending_infos.iter().map(|user_lending_info| user_lending_info.amount).sum();
    let interest = calculate_interest_given_user_deposit_vector(user_lending_infos);
    let total_asset_to_be_returned = total_asset_lent + interest;

    // check if the total asset to be returned is less that on equal to teh total asset available in the pool
    if total_asset_to_be_returned > total_asset_available_in_pool {
        return Err(ContractError::InsufficientFunds {});
    }

    // delete the user's lending info from the mappig USERS_LENDING_INFOS
    USERS_LENDING_INFOS.remove(deps.storage, &info.sender);

    // update the total asset available in the pool
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available_in_pool - total_asset_to_be_returned);

    // send the funds to the user
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: denom,
            amount: total_asset_to_be_returned,
        }],
    };

    Ok(Response::new().add_message(bank_msg))
}
