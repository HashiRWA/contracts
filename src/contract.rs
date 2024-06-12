use cosmwasm_std::{
    entry_point, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, Timestamp, Uint128, StdError,
};
use crate::error::{ContractError, ContractResult};
use crate::msg::{InstantiateMsg, QueryMsg, TransactMsg};
use crate::state::{
    ADMIN, ASSET_CONFIG, COLLATERAL_CONFIG, PRINCIPLE_TO_REPAY, COLLATERAL_SUBMITTED, INTEREST_EARNED, INTEREST_TO_REPAY,
    NANOSECONDS_IN_YEAR, POOL_CONFIG, PRINCIPLE_DEPLOYED, TOTAL_ASSET_AVAILABLE, TOTAL_COLLATERAL_AVAILABLE,
};
use crate::types::{CoinConfig, PoolConfig};
use crate::query::query_handler;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let admin_addr = deps.api.addr_validate(&msg.admin)?;
    POOL_CONFIG.save(deps.storage, &msg.pool_config)?;
    ADMIN.save(deps.storage, &admin_addr)?;
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
        TransactMsg::AddLiquidity {} => add_liquidity(deps, env, info),

        TransactMsg::Deposit {} => deposit(deps, env, info),

        TransactMsg::WithdrawInterest {} => withdraw_interest(deps, env, info),

        TransactMsg::Withdraw { amount } => withdraw(deps, env, info, amount),
        
        TransactMsg::Borrow { amount } => borrow(deps, env, info, amount),
        
        TransactMsg::Repay {} => repay(deps, env, info),
        
        
        TransactMsg::Liquidate {} => liquidate(deps, env, info),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    query_handler(deps, env, msg)
}


fn add_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {

    let asset_info: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let collateral_info: CoinConfig = COLLATERAL_CONFIG.load(deps.storage)?;

    let asset_amount = info.funds.iter().find(|coin| coin.denom == asset_info.denom).unwrap().amount;
    let collateral_amount = info.funds.iter().find(|coin| coin.denom == collateral_info.denom).unwrap().amount;

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    total_asset_available += asset_amount;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    let mut total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;
    total_collateral_available += collateral_amount;
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available)?;

    Ok(Response::default())
}


fn liquidate(
    deps: DepsMut,
    _env: Env,
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
                denom: asset_info.denom.clone(),
                amount: total_asset_available,
            },
            Coin {
                denom: collateral_info.denom.clone(),
                amount: total_collateral_available,
            },
        ],
    };
    Ok(Response::new().add_message(bank_msg))
}

fn calculate_simple_interest(principal: Uint128, interest_rate: Uint128, time_period: u64) -> Uint128 {
    principal * interest_rate * Uint128::from(time_period) / Uint128::from(NANOSECONDS_IN_YEAR)
}

fn get_time_period(now: Timestamp, time: Timestamp) -> u64 {
    now.seconds() - time.seconds()
}

fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let asset_config: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    if now > pool_config.pool_maturation_date {
        return Err(ContractError::PoolMatured {});
    }

    let principle_deployed = PRINCIPLE_DEPLOYED.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let interest_earned_by_user = INTEREST_EARNED.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());

    let time_period = get_time_period(Timestamp::from_seconds(now), principle_deployed.1);
    let interest_since_last_deposit = calculate_simple_interest(principle_deployed.0, pool_config.pool_lend_interest_rate, time_period);

    let principle_to_deposit = info.funds.iter().find(|coin| coin.denom == asset_config.denom).unwrap().amount;

    INTEREST_EARNED.save(deps.storage, &info.sender, &(interest_earned_by_user + interest_since_last_deposit))?;
    PRINCIPLE_DEPLOYED.save(deps.storage, &info.sender, &(principle_deployed.0 + principle_to_deposit, Timestamp::from_seconds(now)))?;

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    total_asset_available += principle_to_deposit;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    Ok(Response::default())
}

fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let asset_config: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    let (principle_deployed, last_deposit_time) = PRINCIPLE_DEPLOYED.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let interest_earned_by_user = INTEREST_EARNED.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());

    if principle_deployed == Uint128::zero() {
        return Err(ContractError::PositionNotAvailable {});
    } else if principle_deployed < amount {
        return Err(ContractError::InsufficientFunds {});
    }

    let time_period = get_time_period(Timestamp::from_seconds(now), last_deposit_time);
    let interest = calculate_simple_interest(principle_deployed, pool_config.pool_lend_interest_rate, time_period);

    INTEREST_EARNED.save(deps.storage, &info.sender, &(interest_earned_by_user + interest))?;
    PRINCIPLE_DEPLOYED.save(deps.storage, &info.sender, &(principle_deployed - amount, Timestamp::from_seconds(now)))?;

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    total_asset_available -= amount;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: asset_config.denom.clone(),
            amount,
        }],
    };

    Ok(Response::default().add_message(bank_msg))
}

fn withdraw_interest(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let interest_earned_by_user = INTEREST_EARNED.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());
    INTEREST_EARNED.save(deps.storage, &info.sender, &Uint128::zero())?;

    let asset_config: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: asset_config.denom.clone(),
            amount: interest_earned_by_user,
        }],
    };
    Ok(Response::new().add_message(bank_msg))
}

fn borrow(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    if now > pool_config.pool_maturation_date {
        return Err(ContractError::PoolMatured {});
    }

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    if total_asset_available < amount {
        return Err(ContractError::InsufficientFunds {});
    }

    let asset_config: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let collateral_config: CoinConfig = COLLATERAL_CONFIG.load(deps.storage)?;

    let (collateral_submitted_by_user, last_collateral_time) = COLLATERAL_SUBMITTED.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let (principle_to_repay_by_user, last_principle_time) = PRINCIPLE_TO_REPAY.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let interest_to_repay_by_user = INTEREST_TO_REPAY.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());

    if last_collateral_time != last_principle_time {
        return Err(ContractError::InvalidState {});
    }
    if principle_to_repay_by_user == Uint128::zero() && interest_to_repay_by_user == Uint128::zero(){
        return Err(ContractError::PositionNotAvailable {});
    }

    let collateral_amount_sent = info.funds.iter().find(|coin| coin.denom == collateral_config.denom).unwrap().amount;
    let strike = pool_config.pool_strike_price;
    let current_ocf = pool_config.min_overcollateralization_factor;

    if current_ocf < Uint128::new(1) {
        return Err(ContractError::InsufficientOCF {});
    }

    let needed_collateral = amount * strike * current_ocf;
    if collateral_amount_sent < needed_collateral {
        return Err(ContractError::InsufficientCollateral {});
    }

    let time_period = get_time_period(Timestamp::from_seconds(now), last_principle_time);
    let interest_on_current_principle = calculate_simple_interest(principle_to_repay_by_user, pool_config.pool_debt_interest_rate, time_period);

    INTEREST_TO_REPAY.save(deps.storage, &info.sender, &(interest_to_repay_by_user + interest_on_current_principle))?;
    PRINCIPLE_TO_REPAY.save(deps.storage, &info.sender, &(principle_to_repay_by_user + amount, Timestamp::from_seconds(now)))?;
    COLLATERAL_SUBMITTED.save(deps.storage, &info.sender, &(collateral_submitted_by_user + needed_collateral, Timestamp::from_seconds(now)))?;

    total_asset_available -= amount;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    let mut total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;
    total_collateral_available += needed_collateral;
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available)?;

    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![
            Coin {
                denom: asset_config.denom.clone(),
                amount,
            },
            Coin {
                denom: collateral_config.denom.clone(),
                amount: collateral_amount_sent - needed_collateral,
            },
        ],
    };

    Ok(Response::new().add_message(bank_msg))
}

fn repay(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    if now > pool_config.pool_maturation_date {
        return Err(ContractError::CollateralForfeited {});
    }

    let amount_to_repay = info.funds.iter().find(|coin| coin.denom == pool_config.asset_address.to_string()).unwrap().amount;

    let (collateral_submitted_by_user, last_collateral_time) = COLLATERAL_SUBMITTED.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let (principle_to_repay_by_user, last_principle_time) = PRINCIPLE_TO_REPAY.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let interest_to_repay_by_user = INTEREST_TO_REPAY.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());

    if interest_to_repay_by_user == Uint128::zero() && principle_to_repay_by_user == Uint128::zero() {
        return Err(ContractError::PositionNotAvailable {});
    }

    if last_collateral_time != last_principle_time {
        return Err(ContractError::InvalidState {});
    }
    if principle_to_repay_by_user == Uint128::zero() && interest_to_repay_by_user == Uint128::zero() && collateral_submitted_by_user == Uint128::zero() {
        return Err(ContractError::PositionNotAvailable {});
    }

    let time_period = get_time_period(Timestamp::from_seconds(now), last_principle_time);
    let interest_on_current_principle = calculate_simple_interest(principle_to_repay_by_user, pool_config.pool_debt_interest_rate, time_period);

    let strike = pool_config.pool_strike_price;
    let current_ocf = pool_config.min_overcollateralization_factor;

    if current_ocf < Uint128::new(1) {
        return Err(ContractError::InsufficientOCF {});
    }

    let needed_collateral = (amount_to_repay + interest_to_repay_by_user + interest_on_current_principle) * strike * current_ocf;
    if collateral_submitted_by_user < needed_collateral {
        return Err(ContractError::InvalidState {});
    }

    INTEREST_TO_REPAY.save(deps.storage, &info.sender, &Uint128::zero())?;

    PRINCIPLE_TO_REPAY.save(deps.storage, &info.sender, &(principle_to_repay_by_user - amount_to_repay, Timestamp::from_seconds(now)))?;
    COLLATERAL_SUBMITTED.save(deps.storage, &info.sender, &(collateral_submitted_by_user - needed_collateral, Timestamp::from_seconds(now)))?;

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    total_asset_available -= amount_to_repay;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    let mut total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;
    total_collateral_available -= needed_collateral;
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available)?;

    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: pool_config.collateral_address.to_string(),
            amount: needed_collateral,
        }],
    };

    Ok(Response::new().add_message(bank_msg))
}
