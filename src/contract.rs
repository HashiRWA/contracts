use std::convert::TryInto;
use std::iter::Map;
use crate::external::query_price;
use crate::types::{CoinConfig, PoolConfig, UserBorrowingInfo, UserLendingInfo};
use std::ops::Sub
// Creating a Lending and Borrowing Protocol,
// where users can deposit assets and borrow other assets.
// User can lend anytime in the pool upto a maturity date.
// User can close their lending position only after maturity date.
// User can borrow anytime in the pool upto a maturity date.
// User can close their borrowing position anytime before the maturity date, (if not closed before maturity -> liquidation of collateral).

use cosmwasm_std::{
    entry_point, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,  Timestamp, Uint128
};
// use oracle::msg::PriceResponse;

use crate::error::{ContractError, ContractResult};
use crate::msg::{ InstantiateMsg, QueryMsg, TransactMsg};

use crate::state::{ADMIN, ASSET_CONFIG, COLLATERAL_CONFIG, INTEREST_EARNED, NANOSECONDS_IN_YEAR, POOL_CONFIG, PRINCIPLE_DEPLOYED, TOTAL_ASSET_AVAILABLE, TOTAL_COLLATERAL_AVAILABLE, USERS_BORROWING_INFOS, USERS_LENDING_INFOS};
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
            withdraw(deps, env, info, denom)
        },
        TransactMsg::Borrow { denom, amount } => {
            borrow(deps, env, info, denom, amount)
        },
        TransactMsg::Repay {} => {
            repay(deps, env, info)
        },
        
        TransactMsg::Liquidate {} => {
            liquidate(deps, env, info)
        },
    }
}



#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    query_handler(deps, env, msg)
}





// can only be called by the admin
// Send all the funds available in the pool to the admin

fn liquidate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }
    let total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    let asset_info: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;
    let collateral_info: CoinConfig = COLLATERAL_CONFIG.load(deps.storage)?;
    let bank_msg = BankMsg::Send {
        to_address: admin.to_string(),
        amount: vec![
            Coin {
                denom: asset_info.denom.to_string(),
                amount: total_asset_available,
            },
            Coin {
                denom: collateral_info.denom.to_string(),
                amount: total_collateral_available,
            },
        ],
    };
    Ok(Response::new().add_message(bank_msg))
}


fn calculate_simple_interest(principle: Uint128, interest_rate: Uint128, time_period: u64) -> Uint128 {
    // formula : P * R(for 1 year) * T (in nanoseconds) / NANOSECONDS_IN_YEAR
    principle * interest_rate * Uint128::from(time_period / NANOSECONDS_IN_YEAR)
}

fn get_time_period(now: Timestamp, time: Timestamp) -> u64 {
    now.seconds() - time.seconds()
}


fn deposit(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {

    let pool_config:PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let asset_config:CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let now = env.block.time;
    
    if now > pool_config.pool_maturation_date {return Err(ContractError::PoolMatured {});}

    let principle_deployed_map : Map<&Addr, (Uint128,Timestamp)> = PRINCIPLE_DEPLOYED.load(deps.storage)?;
    let interest_earned_map : Map<&Addr, Uint128> = INTEREST_EARNED.load(deps.storage)?;
    
    let interest_earned_by_user = interest_earned_map.get(&info.sender).unwrap_or_default();
    let principle_and_timestamp:(Uint128,Timestamp) = principle_deployed_map.get(&info.sender).unwrap_or_default();

    let principle_deployed = principle_and_timestamp.0;
    let last_deposit_time = principle_and_timestamp.1;

    let time_period = get_time_period(now, last_deposit_time);
    let interest_since_last_deposit = calculate_simple_interest(principle_deployed, pool_config.pool_lend_interest_rate, time_period);

    let principle_to_deposit = info.funds.iter().find(|coin| coin.denom == asset_config.denom).unwrap().amount;

    INTEREST_EARNED.save(deps.storage, &info.sender, &interest_earned_by_user + interest_since_last_deposit);
    PRINCIPLE_DEPLOYED.save(deps.storage, &info.sender, (&principle_deployed + principle_to_deposit, now));

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    total_asset_available += principle_to_deposit;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    Ok(Response::default())
}


fn withdraw(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount : Uint128
) -> ContractResult<Response> {
    
    let principle_to_withdraw = amount;
    let pool_config:PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let asset_config:CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let now = env.block.time;

    let principle_deployed_map : Map<&Addr, Uint128> = PRINCIPLE_DEPLOYED.load(deps.storage)?;
    let interest_earned_map : Map<&Addr, Uint128> = INTEREST_EARNED.load(deps.storage)?;
    
    let interest_earned_by_user = interest_earned_map.get(&info.sender).unwrap_or_default();
    let principle_and_timestamp:(Uint128,Timestamp) = principle_deployed_map.get(&info.sender).unwrap_or_default();

    let principle_deployed = principle_and_timestamp.0;
    let last_deposit_time = principle_and_timestamp.1;

    // can't execute withdraw if user doesn't have any position
    if principle_deployed == Uint128::zero() && interest_earned_by_user == Uint128::zero() {
        return Err(ContractError::PositionNotAvailable {});
    } else if principle_deployed < principle_to_withdraw {
        return Err(ContractError::InsufficientFunds {});
    }
 
    let time_period = get_time_period(now, last_deposit_time);
    let interest = calculate_simple_interest(principle_deployed, pool_config.pool_lend_interest_rate, time_period);

    // TODO: can send interest with principle but not sending right now

    INTEREST_EARNED.save(deps.storage, &info.sender, &interest_earned_by_user + interest);
    PRINCIPLE_DEPLOYED.save(deps.storage, &info.sender, (&principle_deployed - principle_to_withdraw, now));

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    total_asset_available -= principle_to_withdraw;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    // send the funds to the user
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: asset_config.denom.to_string(),
            amount: principle_to_withdraw,
        }],
    };

    Ok(Response::default())
}


// TODO: a function that let's user transfer all the interest earned to their account
fn withdraw_interest(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let interest_earned_map : Map<&Addr, Uint128> = INTEREST_EARNED.load(deps.storage)?;
    let interest_earned_by_user = interest_earned_map.get(&info.sender).unwrap_or_default();
    INTEREST_EARNED.save(deps.storage, &info.sender, &Uint128::zero());
    let asset_config:CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: asset_config.denom.to_string(),
            amount: interest_earned_by_user,
        }],
    };
    Ok(Response::new().add_message(bank_msg))
}


// TODO : verify this with madhav
fn calculate_interest_given_user_deposit_vector(deps : DepsMut, user_lending_infos: Vec<UserLendingInfo>) -> Uint128 {
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
        let interest = user_lending_info.amount * user_lending_info.interest_rate * Uint128::from(time_period / NANOSECONDS_IN_YEAR);
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

fn withdraw2(
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
    let interest = calculate_interest_given_user_deposit_vector(deps, user_lending_infos);
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


// TODO: Create Borrow Function
// 1) check if the pool has matured, if the pool has matured, revert the transaction
// 2) user will send the amount of asset that they want to borrow
// 3) user will also be sending the amount they will submit as collateral.
// 4) check if the amount of asset that user wants to borrow is less than or equal to the total asset amount in the pool
// 5) if the amount of asset that user wants to borrow is greater than the total asset amount in the pool, revert the transaction saying "Insufficient Funds"
// 6) else
// 7) check if the collateral provided by the user is over collateralised or not.
// 8) if the collateral provided by the user is not over collateralised, revert the transaction saying "Insufficient Collateral"
// 9) else
// 10) update the user's borrowing info vector with the new borrowing info object
// 11) update the total collateral amount in the pool (will become greater)
// 12) update the total asset amount in the pool (will become lesser)
// 13) send the sender the amount of asset that they wanted to borrow
// 14) return the response
fn borrow(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    amount: Uint128, // amount of asset that user wants to borrow
) -> ContractResult<Response> {
    // get pool config
    let pool_config:PoolConfig = POOL_CONFIG.load(deps.storage)?;
    // check if the pool has matured
    if env.block.time > pool_config.pool_maturation_date {
        return Err(ContractError::PoolMatured {});
    }
    
    let total_asset_available_in_pool = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    let total_collateral_available_in_pool = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;

// 4) check if the amount of asset that user wants to borrow is less than or equal to the total asset amount in the pool

    if amount > total_asset_available_in_pool {
        return Err(ContractError::InsufficientFunds {});
    }

    // 7) check if the collateral provided by the user is over collateralised or not.
    let asset_config = ASSET_CONFIG.load(deps.storage)?;
    let collateral_config = COLLATERAL_CONFIG.load(deps.storage)?;
    let collateral_amount = info.funds.iter().find(|coin| coin.denom == collateral_config.denom).unwrap().amount;

    let collateral_price: Uint128 = queryPrice();
    let asset_price: Uint128 = queryPrice();

    let collateral_value = Uint128::from(collateral_amount) * Uint128::from(collateral_price);
    let asset_value = amount * asset_price;

    let collateral_to_asset_ratio = collateral_value / asset_value;
    
    if collateral_to_asset_ratio < pool_config.min_overcollateralization_factor {
        return Err(ContractError::Undercollateralized {});
    }

    // 10) update the user's borrowing info vector with the new borrowing info object
    let user_borrowing_info = UserBorrowingInfo {
        type_ : String::from("borrow"),
        amount,
        time: env.block.time,
        collateral_submitted: collateral_amount,
        interest_rate: pool_config.pool_debt_interest_rate,
    };

    let mut users_borrowing_infos : Map<&Addr, Vec<UserBorrowingInfo>>  = USERS_BORROWING_INFOS.load(deps.storage, &info.sender)?;
    // get user's borrowing info from the users borrowing info map
    let user_borrowing_infos : Vec<UserBorrowingInfo> = users_borrowing_infos.get(&info.sender).unwrap();
    user_borrowing_infos.push(user_borrowing_info);
    USERS_BORROWING_INFOS.save(deps.storage, &info.sender, &user_borrowing_infos)?;

    // 11) update the total collateral amount in the pool (will become greater)
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available_in_pool + collateral_amount);

    // 12) update the total asset amount in the pool (will become lesser)
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available_in_pool - amount);

    // 13) send the sender the amount of asset that they wanted to borrow
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: denom,
            amount: amount,
        }],
    };

    Ok(Response::new().add_message(bank_msg))
}


// TODO: Write calculate_debt function
// 1) takes in a vector of user borrowing info
// 2) the array consumes of the user's borrowing info objects
// 3) each object has equal possibility of either positive amount (user taking more debt) or negative amount (user repaying some debt)
// 4) while iterating over the array, calculate the interest for each borrowing info object
// 5) interest is calcuated as the amount * interest_rate * time_period / NANOSECONDS_IN_YEAR
// 6) time period is calculated as the differnce between current time and the time of borrowing
// 7) sum all the interests and  sum all the amounts in the borrowing info objects
// 8) return the sum of all the interests and the sum of all the amounts

fn calculate_debt(
    deps: DepsMut,
    env: Env,
    user_borrowing_infos: Vec<UserBorrowingInfo>) -> (Uint128,Uint128) {
    let mut overall_principle: Uint128 = Uint128::zero();
    let mut overall_interest: Uint128 = Uint128::zero();
    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let now : Timestamp = env.block.time;
    
    for user_borrowing_info in user_borrowing_infos {
        let time_period = now.seconds() - user_borrowing_info.time.seconds();
        // TODO: get this formula validated from madhav
        let interest = user_borrowing_info.amount * user_borrowing_info.interest_rate * Uint128::from(time_period / NANOSECONDS_IN_YEAR);
        overall_interest += interest;
        overall_principle += user_borrowing_info.amount;
    }

    (overall_principle, overall_interest)
    
}




// TODO: Write Repay Function
// 1) check if the pool has matured, if the pool has matured, revert the transaction
// 2) user will send any amount less than equal to total debt they have (total debt = sum of all the borrowings + interests)  
// 4) calculate the whole of user's debt using a calcualte_debt function, which will calculate the debt using the user's borrowing info vector
// 5) check if the amount that user wants to repay is less than or equal to the total debt
// 6) if the amount that user wants to repay is greater than the total debt, revert the transaction saying "Excessive Funds"
// 7) else
// 8) update the user's borrowing info vector with the new borrowing info object
// 9) update the total collateral amount in the pool (will become lesser)
// 10) update the total asset amount in the pool (will become greater)
// 11) send the sender the amount of collateral that they will gain back after repaying the debt
// 12) return the response

fn repay(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    // get pool config
    let pool_config:PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let asset_config = ASSET_CONFIG.load(deps.storage)?;
    // check if the pool has matured
    if env.block.time > pool_config.pool_maturation_date {
        return Err(ContractError::PoolMatured {});
    }
    
    let total_asset_available_in_pool = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    let total_collateral_available_in_pool = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;

    // get the user's borrowing info
    let users_borrowing_infos: Map<&Addr, Vec<UserBorrowingInfo>> = USERS_BORROWING_INFOS.load(deps.storage, &info.sender)?;
    // get user borrowing info from the users borrowing info map
    let user_borrowing_infos : Vec<UserBorrowingInfo> = users_borrowing_infos.get(&info.sender).unwrap();

    let (principle, interest) = calculate_debt(
        deps,
        env,
        user_borrowing_infos
    );
    let total_user_debt = principle + interest;
    let amount = info.funds.iter().find(|coin| coin.denom == asset_config.denom).unwrap().amount;

    // check if the amount that user wants to repay is less than or equal to the total debt
    if amount > total_user_debt {
        return Err(ContractError::ExcessiveFunds {});
    }

    
    
    let collateral_value = queryPrice();
    let asset_value = queryPrice();

    // Calculate the user's collateral according to the principle they are paying back
    let collateral_amount = amount * collateral_value / asset_value;

    // update the user's borrowing info vector with the new borrowing info object
    let user_borrowing_info = UserBorrowingInfo {
        type_ : String::from("repay"),
        amount,
        time: env.block.time,
        collateral_submitted: collateral_amount,
        interest_rate: pool_config.pool_debt_interest_rate,
    };

    user_borrowing_infos.push(user_borrowing_info);
    USERS_BORROWING_INFOS.save(deps.storage, &info.sender, &user_borrowing_infos)?;

    let collateral_info:CoinConfig = COLLATERAL_CONFIG.load(deps.storage)?;
    // update the total collateral amount in the pool (will become lesser)
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available_in_pool - collateral_amount);
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available_in_pool + amount);

    // send the funds to the user
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: collateral_info.denom.to_string(),
            amount: collateral_amount,
        }],
    };

    Ok(Response::new().add_message(bank_msg))

}
