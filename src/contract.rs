use std::iter::Map;
use crate::types::{CoinConfig, PoolConfig};
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

use crate::state::{ADMIN, ASSET_CONFIG, COLLATERAL_CONFIG, PRINCIPLE_TO_REPAY , COLLATERAL_SUBMITTED, INTEREST_EARNED, INTEREST_TO_REPAY, NANOSECONDS_IN_YEAR, POOL_CONFIG, PRINCIPLE_DEPLOYED, TOTAL_ASSET_AVAILABLE, TOTAL_COLLATERAL_AVAILABLE};
use crate::query::query_handler;


#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    POOL_CONFIG.save(deps.storage, &msg.pool_config)?;
    ADMIN.save(deps.storage, &msg.admin)?;
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
        TransactMsg::Withdraw { amount } => {
            withdraw(deps, env, info, amount)
        },
        TransactMsg::Borrow { amount } => {
            borrow(deps, env, info, amount)
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


fn borrow(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    
    let pool_config:PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let now = env.block.time;
    
    if now > pool_config.pool_maturation_date {return Err(ContractError::PoolMatured {});}
    
    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    let mut total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;
    
    let amount_to_borrow = amount;

    if total_asset_available < amount_to_borrow { return Err(ContractError::InsufficientFunds {}); }
    
    let asset_config:CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let collateral_config:CoinConfig = COLLATERAL_CONFIG.load(deps.storage)?;
    let collateral_submitted_map : Map<&Addr, (Uint128,Timestamp)> = COLLATERAL_SUBMITTED.load(deps.storage)?;
    let principle_to_repay_map :  Map<&Addr, (Uint128,Timestamp)> = PRINCIPLE_TO_REPAY.load(deps.storage)?;
    let interest_to_repay_map : Map<&Addr, Uint128> = INTEREST_TO_REPAY.load(deps.storage)?;  

    let interest_to_repay_by_user = interest_to_repay_map.get(&info.sender).unwrap_or_default();
    let principle_to_repay_by_user_reply : (Uint128, Timestamp) = principle_to_repay_map.get(&info.sender).unwrap_or_default();
    let collateral_submitted_by_user_reply: (Uint128, Timestamp) = collateral_submitted_map.get(&info.sender).unwrap_or_default();


    let principle_to_repay_by_user = principle_to_repay_by_user_reply.0;
    let last_deposit_time = principle_to_repay_by_user_reply.1;

    let collateral_submitted_by_user = collateral_submitted_by_user_reply.0;
    let last_deposit_time_collateral = collateral_submitted_by_user_reply.1;

    if last_deposit_time != last_deposit_time_collateral { return Err(ContractError::InvalidState {}); }
    if principle_to_repay_by_user == Uint128::zero() && interest_to_repay_by_user == Uint128::zero() && collateral_submitted_by_user == Uint128::zero() { return Err(ContractError::PositionNotAvailable {}); }
    
    let collateral_amount_sent = info.funds.iter().find(|coin| coin.denom == collateral_config.denom).unwrap().amount;

    // calculations will be based on the amount to be borrowed.

    // figuring out ocf.
    let strike = pool_config.pool_strike_price; // strike = collateral / asset
    let current_ocf = pool_config.min_overcollateralization_factor;

    if current_ocf < 1 { return Err(ContractError::InsufficientOCF {}); }

    // x gold = amount of usdc * strike * ocf
    let needed_collateral: Uint128 = amount_to_borrow * strike * current_ocf;
    if collateral_amount_sent < needed_collateral { return Err(ContractError::InsufficientCollateral {});}
    
    let time_period = get_time_period(now, last_deposit_time);
    let interest_on_current_principle = calculate_simple_interest(principle_to_repay_by_user, pool_config.pool_debt_interest_rate, time_period);
    
    // update user's interest on current loan
    INTEREST_TO_REPAY.save(deps.storage, &info.sender, &interest_to_repay_by_user + interest_on_current_principle);

    // update user's loan
    PRINCIPLE_TO_REPAY.save(deps.storage, &info.sender, (&principle_to_repay_by_user + amount_to_borrow, now));

    // update user's collateral
    COLLATERAL_SUBMITTED.save(deps.storage, &info.sender, (&collateral_submitted_by_user + needed_collateral, now));

    // update total asset available in the pool
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available - amount_to_borrow);
    // update total collateral available in the pool
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available + needed_collateral);

    // send the funds to the user, 
    // send the amount of asset that they wanted to borrow
    // send the amount of collateral that they will have left after -> collateral_amount_sent - needed_collateral
    // TODO: ensure that this is not sending collateral tokens out of the blue. this is to minimise the dust positions
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![
            Coin {
                denom: asset_config.denom.to_string(),
                amount: amount_to_borrow,
            },
            Coin {
                denom: collateral_config.denom.to_string(),
                amount: collateral_amount_sent - needed_collateral,
            },
        ],
    };

    Ok(Response::new().add_message(bank_msg))
}

fn repay(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    
    let pool_config:PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let now = env.block.time;
    
    if now > pool_config.pool_maturation_date {return Err(ContractError::CollateralForfeited {});}
    
    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    let mut total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;
    
    
    let asset_config:CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let collateral_config:CoinConfig = COLLATERAL_CONFIG.load(deps.storage)?;
   
    let amount_to_repay = info.funds.iter().find(|coin| coin.denom == asset_config.denom).unwrap().amount;
   
    let collateral_submitted_map : Map<&Addr, (Uint128,Timestamp)> = COLLATERAL_SUBMITTED.load(deps.storage)?;
    let principle_to_repay_map :  Map<&Addr, (Uint128,Timestamp)> = PRINCIPLE_TO_REPAY.load(deps.storage)?;
    let interest_to_repay_map : Map<&Addr, Uint128> = INTEREST_TO_REPAY.load(deps.storage)?;  

    let interest_to_repay_by_user = interest_to_repay_map.get(&info.sender).unwrap_or_default();
    let principle_to_repay_by_user_reply : (Uint128, Timestamp) = principle_to_repay_map.get(&info.sender).unwrap_or_default();
    
    let collateral_submitted_by_user_reply: (Uint128, Timestamp) = collateral_submitted_map.get(&info.sender).unwrap_or_default();
    
    let principle_to_repay_by_user = principle_to_repay_by_user_reply.0;
    let last_deposit_time = principle_to_repay_by_user_reply.1;

    if interest_to_repay_by_user == Uint128::zero() && principle_to_repay_by_user == Uint128::zero() { return Err(ContractError::PositionNotAvailable {}); }
    
    let collateral_submitted_by_user = collateral_submitted_by_user_reply.0;
    let last_deposit_time_collateral = collateral_submitted_by_user_reply.1;

    if last_deposit_time != last_deposit_time_collateral { return Err(ContractError::InvalidState {}); }
    if principle_to_repay_by_user == Uint128::zero() && interest_to_repay_by_user == Uint128::zero() && collateral_submitted_by_user == Uint128::zero() { return Err(ContractError::PositionNotAvailable {}); }
    
    let time_period = get_time_period(now, last_deposit_time);
    let interest_on_current_principle = calculate_simple_interest(principle_to_repay_by_user, pool_config.pool_debt_interest_rate, time_period);
    
    // update user's interest on current loan
    INTEREST_TO_REPAY.save(deps.storage, &info.sender, &interest_to_repay_by_user + interest_on_current_principle);

    // calculations will be based on the amount to be repayed.

    // figuring out ocf.
    let strike = pool_config.pool_strike_price; // strike = collateral / asset
    let current_ocf = pool_config.min_overcollateralization_factor;

    if current_ocf < 1 { return Err(ContractError::InsufficientOCF {}); }

    // x gold = amount of usdc * strike * ocf
    // TODO: Validate, asking user each time they withdraw, to give accumulated interest as well.
    let needed_collateral: Uint128 = (amount_to_repay + &interest_to_repay_by_user + interest_on_current_principle ) * strike * current_ocf;
    if collateral_submitted_by_user < needed_collateral { return Err(ContractError::InvalidState {});}

    INTEREST_TO_REPAY.save(deps.storage, &info.sender, &Uint128::zero());

    // TODO: possibility of frontrunning
    // update user's loan
    PRINCIPLE_TO_REPAY.save(deps.storage, &info.sender, (&principle_to_repay_by_user - amount_to_repay, now));

    // update user's collateral
    COLLATERAL_SUBMITTED.save(deps.storage, &info.sender, (&collateral_submitted_by_user - needed_collateral, now));

    // update total asset available in the pool
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available - amount_to_repay);
    // update total collateral available in the pool
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available - needed_collateral);

    // send the funds to the user, 
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![
            Coin {
                denom: collateral_config.denom.to_string(),
                amount: needed_collateral,
            },
        ],
    };
    Ok(Response::new().add_message(bank_msg))
}
