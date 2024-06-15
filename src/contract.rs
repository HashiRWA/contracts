use cosmwasm_std::{
    entry_point, from_binary, from_json, Addr, BankMsg, Binary, BlockInfo, Coin, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, QueryRequest, Response, StdResult, SubMsg, Timestamp, Uint128, WasmMsg, WasmQuery
};
use cw20::{AllowanceResponse, Balance, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};
use cw20_base::allowances::{
    execute_transfer_from, query_allowance,
};
use crate::error::{ContractError, ContractResult};
use crate::msg::{DepositMsg, ExecuteMsg, InstantiateMsg, LoanMsg, QueryMsg, RepayMsg, TransactMsg, WithdrawMsg};
use crate::state::{
    ADMIN, ASSET_CONFIG, COLLATERAL_CONFIG, COLLATERAL_SUBMITTED, INTEREST_EARNED, INTEREST_TO_REPAY, NANOSECONDS_IN_YEAR, POOL_CONFIG, PRINCIPLE_DEPLOYED, PRINCIPLE_TO_REPAY, TOTAL_ASSET_AVAILABLE, TOTAL_COLLATERAL_AVAILABLE, TOTAL_PROTOCOL_EARNINGS
};
use crate::amount::{self, Amount};
use crate::types::{CoinConfig, PoolConfig};
use cosmwasm_std::to_json_binary;
use cw_utils::{maybe_addr, nonpayable, one_coin};

use cosmwasm_std::to_binary;
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let admin_addr = deps.api.addr_validate(&msg.admin)?;
    POOL_CONFIG.save(deps.storage, &msg.config)?;
    ADMIN.save(deps.storage, &admin_addr)?;

    // Initialize asset and collateral configurations
    let asset_config = CoinConfig {
        denom: msg.config.asset,
        decimals: 6,  
    };
    ASSET_CONFIG.save(deps.storage, &asset_config)?;

    let collateral_config = CoinConfig {
        denom: msg.config.collateral,
        decimals: 6,  
    };
    COLLATERAL_CONFIG.save(deps.storage, &collateral_config)?;

    TOTAL_ASSET_AVAILABLE.save(deps.storage, &Uint128::zero())?;
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &Uint128::zero())?;

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
        ExecuteMsg::Transact(transact_msg) => match transact_msg {

            // TODO: Receive will impl Cw20ReceiveMsg, just to recieve the msg that the funds have been transferred
            TransactMsg::Receive(msg) => execute_receive(deps, env, info, msg),

            TransactMsg::Deposit(msg) => execute_deposit(deps, env, info, msg),
            TransactMsg::Withdraw (msg) => execute_withdraw(deps, env, info, msg),
            TransactMsg::WithdrawInterest {} => execute_withdraw_interest(deps, env, info),
            TransactMsg::Loan(msg) => execute_loan(deps, env, info, msg),
            TransactMsg::Repay(msg) => execute_repay(deps, env, info, msg),

            TransactMsg::AddLiquidity {} => add_liquidity(deps, env, info),
            TransactMsg::Liquidate {} => liquidate(deps, env, info),
        },
        ExecuteMsg::UpdateUserAssetInfo { user_addr } => update_user_asset_info(deps, user_addr),
        ExecuteMsg::UpdateAsset {
            denom,
            decimals,
            target_utilization_rate_bps,
            min_rate,
            optimal_rate,
            max_rate,
        } => update_asset(
            deps,
            denom,
            decimals,
            target_utilization_rate_bps,
            min_rate,
            optimal_rate,
            max_rate,
        ),
    }
}



// Docs :
// This function is used to receive the mesaage that the funds have been transferred
// This function will not accept any coins, only a Cw20ReceiveMsg
// This function does nothing else
fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    Ok(Response::default())
}




// Docs: 
// this is a helper function that fetches for allowance
// asks for spender, owner, and cw20 address and returns the allowance

pub fn fetch_allowance(
    deps: Deps,
    spender: Addr,
    owner: Addr,
    cw20_addr: Addr,
) -> Result<AllowanceResponse, ContractError> {
    let allowance_query = Cw20QueryMsg::Allowance {
        spender: spender.clone().to_string(),
        owner: owner.clone().to_string(),
    };
    let res: AllowanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: cw20_addr.to_string(),
        msg: to_json_binary(&allowance_query)?,
    }))?;
    Ok(res)
}



#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::PoolConfig {} => {
            let pool_config = POOL_CONFIG.load(deps.storage)?;
            Ok(to_json_binary(&pool_config)?)
        },
        QueryMsg::GetOwner {} => {
            let admin = ADMIN.load(deps.storage)?;
            Ok(to_json_binary(&admin)?)
        },
        QueryMsg::GetTotalAssetAvailable {} => {
            let total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
            Ok(to_json_binary(&total_asset_available)?)
        },
        QueryMsg::GetTotalCollateralAvailable {} => {
            let total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;
            Ok(to_json_binary(&total_collateral_available)?)
        },
        QueryMsg::GetUserPrinciple { user } => {
            let user_addr = deps.api.addr_validate(&user)?;
            let principle = PRINCIPLE_DEPLOYED.may_load(deps.storage, &user_addr)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
            Ok(to_json_binary(&principle)?)
        },
        QueryMsg::GetUserPrincipleToRepay { user } => {
            let user_addr = deps.api.addr_validate(&user)?;
            let principle_to_repay = PRINCIPLE_TO_REPAY.may_load(deps.storage, &user_addr)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
            Ok(to_json_binary(&principle_to_repay)?)
        },
        QueryMsg::Assets {} => {
 
            Ok(to_json_binary(&"Assets query response")?)
        },
        QueryMsg::UserAssetsInfo { user } => {

            Ok(to_json_binary(&format!("User assets info for {}", user))?)
        },
        QueryMsg::UserAssetInfo { user, denom } => {

            Ok(to_json_binary(&format!("User asset info for {} and denom {}", user, denom))?)
        },
        QueryMsg::UserData { user } => {
 
            Ok(to_json_binary(&format!("User data for {}", user))?)
        },
        QueryMsg::AssetInfo { denom } => {
    
            Ok(to_json_binary(&format!("Asset info for denom {}", denom))?)
        },
        QueryMsg::AssetsInfo {} => {

            Ok(to_json_binary(&"All assets info")?)
        },
        QueryMsg::MaxLiquidationAmount { user } => {

            Ok(to_json_binary(&format!("Max liquidation amount for {}", user))?)
        },

        QueryMsg::GetDepositQuote { user, amount } => {
            let quote = quote_deposit(deps, user,  amount)?;
            Ok(to_json_binary(&quote)?)
        },

        QueryMsg::GetLoanQuote { amount} => {
            let quote = quote_loan(deps, _env, amount)?;
            Ok(to_json_binary(&quote)?)
        },
        
        QueryMsg::GetWithdrawablePositions { user,} => {
            let quote = get_withdrawable_positions(deps, user)?;
            Ok(to_json_binary(&quote)?)
        },
        QueryMsg::GetRepayablePositions { user, } => {
            let quote = get_repayable_positions(deps, user)?;
            Ok(to_json_binary(&quote)?)
        },
        QueryMsg::GetRepayQuote{ user} => {
            let quote = quote_repay(deps, _env, user)?;
            Ok(to_json_binary(&quote)?)
        },
    }
}


fn quote_repay(    
    deps: Deps,
    env: Env,
    user: Addr,
) -> ContractResult<(Uint128, Uint128, Uint128)> {
    // This nonpayable function ensures that no coins are sent to the contract

    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    if now > pool_config.maturationdate {
        return Err(ContractError::CollateralForfeited {});
    }

    if pool_config.overcollateralizationfactor < Uint128::new(1) {
        return Err(ContractError::InsufficientOCF {});
    }
 
    let (overall_collateral_submitted_by_user, last_collateral_time) = COLLATERAL_SUBMITTED.may_load(deps.storage, &user)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let (overall_principle_to_repay_by_user, last_principle_time) = PRINCIPLE_TO_REPAY.may_load(deps.storage, &user)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let interest_to_repay_by_user_yet = INTEREST_TO_REPAY.may_load(deps.storage, &user)?.unwrap_or(Uint128::zero());

    // TODO: what if somehow - (define) some collateral is left
    if interest_to_repay_by_user_yet == Uint128::zero() && overall_principle_to_repay_by_user == Uint128::zero() {
        return Err(ContractError::PositionNotAvailable {});
    }

    if last_collateral_time != last_principle_time {
        return Err(ContractError::InvalidState {});
    }

    let current_time_period = get_time_period(Timestamp::from_seconds(now), last_principle_time);
    let interest_on_current_principle = calculate_simple_interest(overall_principle_to_repay_by_user, pool_config.debtinterestrate, current_time_period);

    let total_interest_to_pay = interest_to_repay_by_user_yet + interest_on_current_principle;
    let total_loan_to_repay = overall_principle_to_repay_by_user + total_interest_to_pay;
    let total_collateral_to_unlock = calculate_collateral_amount(total_loan_to_repay, pool_config.strikeprice, pool_config.overcollateralizationfactor);

    
    let ret = (total_loan_to_repay, total_interest_to_pay, total_collateral_to_unlock);

    Ok(ret)

}
  



// fn quoteDeosit() 
// This functions is used to calculate the 
// amount of interest the user will earn till maturity
// 1) given the value they have currently entered
// 2) given the total position that they have in the pool

fn quote_deposit(
    deps: Deps,
    user: Addr,
    amount: Uint128,
  ) -> ContractResult<( (Uint128, Uint128),  (Uint128, Uint128))> {
    let pool_config = POOL_CONFIG.load(deps.storage)?;
    let asset_config = ASSET_CONFIG.load(deps.storage)?;
  
    let interest_earned_by_user = INTEREST_EARNED.may_load(deps.storage, &user)?.unwrap_or(Uint128::zero());
  
    let (principle_already_deployed, last_deposit_time) = PRINCIPLE_DEPLOYED.may_load(deps.storage, &user)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
  
    let time_period = get_time_period(Timestamp::from_seconds(pool_config.maturationdate), last_deposit_time);
  
  
    // without the current position
    // at maturity
  
    let interest = calculate_simple_interest(principle_already_deployed, pool_config.lendinterestrate, time_period);
    let user_position_without_new_amount = (principle_already_deployed, interest_earned_by_user + interest);
  
    // with the current position
    // at maturity
  
    let interest = calculate_simple_interest(principle_already_deployed + amount, pool_config.lendinterestrate, time_period);
    let user_position_with_new_amount = (principle_already_deployed + amount, interest_earned_by_user + interest);
  
    Ok((user_position_without_new_amount, user_position_with_new_amount))
  
  }
  
  
  // fn quoteLoan()
  // This function is used to calculate the amount of 
  // interest the user will have to pay till maturity
  // and the amount of collateral they will have to submit now
  // given the amount of the loan they want to take
  // 1) given the value they have currently entered
  // 2) given the total position that they have in the pool
  
  fn quote_loan(
    deps: Deps,
    env: Env,
    amount: Uint128,
  ) -> ContractResult<(Uint128, Uint128, Uint128)> {
    let pool_config = POOL_CONFIG.load(deps.storage)?;
  
    // Don't quote if the pool has expired
    let now = env.block.time.seconds();
    if now > pool_config.maturationdate {
      return Err(ContractError::PoolMatured {});
    }
    // without the previous position
    // at maturity
    
    let quotation_to_maturity_time_period = get_time_period(Timestamp::from_seconds(pool_config.maturationdate), Timestamp::from_seconds(now));
    let interest = calculate_simple_interest(amount, pool_config.debtinterestrate, quotation_to_maturity_time_period);
    let collateral_for_given_position = calculate_collateral_amount(amount, pool_config.strikeprice, pool_config.overcollateralizationfactor);
    let user_position_for_new_amount = (amount, interest, collateral_for_given_position);
  
    Ok(user_position_for_new_amount)
  }
  
  // fn getWithdrawablePositions()
  // This function is used to calculate the total amount of
  // principal the user can withdraw from the pool
  // interest the user has earned till now
  // clubbed with pool config
  
  fn get_withdrawable_positions(
    deps: Deps,
    user: Addr,
  ) -> ContractResult<(Uint128, Uint128)> {
    let pool_config = POOL_CONFIG.load(deps.storage)?;
  
    let interest_earned_by_user = INTEREST_EARNED.may_load(deps.storage, &user)?.unwrap_or(Uint128::zero());
    let (principle_already_deployed, last_deposit_time) = PRINCIPLE_DEPLOYED.may_load(deps.storage, &user)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
  
    let time_period = get_time_period(Timestamp::from_seconds(pool_config.maturationdate), last_deposit_time);
  
    let interest = calculate_simple_interest(principle_already_deployed, pool_config.lendinterestrate, time_period);
    let user_position = (principle_already_deployed, interest_earned_by_user + interest);
  
    Ok(user_position)
  }
  
  // fn getRepayablePositions()
  // This function is used to calculate the total amount of
  // principal the user has to repay to the pool
  // collateral the user can withdraw from the pool
  // interest the user has to repay to the pool
  // clubbed with pool config
  
  fn get_repayable_positions(
    deps: Deps,
    user: Addr,
  ) -> ContractResult<(Uint128, Uint128, Uint128)> {
    let pool_config = POOL_CONFIG.load(deps.storage)?;
  
    let interest_to_repay_by_user = INTEREST_TO_REPAY.may_load(deps.storage, &user)?.unwrap_or(Uint128::zero());
    let principle_to_repay_by_user = PRINCIPLE_TO_REPAY.may_load(deps.storage, &user)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let collateral_submitted_by_user = COLLATERAL_SUBMITTED.may_load(deps.storage, &user)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
  
    let time_period = get_time_period(Timestamp::from_seconds(pool_config.maturationdate), principle_to_repay_by_user.1);
  
    let interest = calculate_simple_interest(principle_to_repay_by_user.0, pool_config.debtinterestrate, time_period);
    let user_position = (principle_to_repay_by_user.0, interest_to_repay_by_user + interest, collateral_submitted_by_user.0);
  
    Ok(user_position)
}

// Placeholder implementation for ExecuteMsg::UpdateUserAssetInfo
fn update_user_asset_info(deps: DepsMut, user_addr: String) -> ContractResult<Response> {
    // Placeholder logic, replace with actual implementation
    Ok(Response::new().add_attribute("action", "update_user_asset_info").add_attribute("user_addr", user_addr))
}

// Placeholder implementation for ExecuteMsg::UpdateAsset
fn update_asset(
    deps: DepsMut,
    denom: String,
    decimals: u16,
    target_utilization_rate_bps: u32,
    min_rate: u32,
    optimal_rate: u32,
    max_rate: u32,
) -> ContractResult<Response> {
    // Placeholder logic, replace with actual implementation
    Ok(Response::new()
        .add_attribute("action", "update_asset")
        .add_attribute("denom", denom)
        .add_attribute("decimals", decimals.to_string())
        .add_attribute("target_utilization_rate_bps", target_utilization_rate_bps.to_string())
        .add_attribute("min_rate", min_rate.to_string())
        .add_attribute("optimal_rate", optimal_rate.to_string())
        .add_attribute("max_rate", max_rate.to_string()))
}

// Existing functions for add_liquidity, deposit, withdraw_interest, withdraw, borrow, repay, and liquidate
fn add_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let asset_info: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let collateral_info: CoinConfig = COLLATERAL_CONFIG.load(deps.storage)?;

    // Ensure funds include the required assets and collateral
    let asset_amount = match info.funds.iter().find(|coin| coin.denom == asset_info.denom) {
        Some(coin) => coin.amount,
        None => return Err(ContractError::InvalidFunds { denom: asset_info.denom.clone().to_string() }),
    };

    let collateral_amount = match info.funds.iter().find(|coin| coin.denom == collateral_info.denom) {
        Some(coin) => coin.amount,
        None => return Err(ContractError::InvalidFunds { denom: collateral_info.denom.clone().to_string() }),
    };

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage).unwrap_or(Uint128::zero());
    let mut total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage).unwrap_or(Uint128::zero());

    // Safely add the amounts to the current totals
    total_asset_available = total_asset_available.checked_add(asset_amount)
        .map_err(|_| ContractError::Overflow {})?;
    total_collateral_available = total_collateral_available.checked_add(collateral_amount)
        .map_err(|_| ContractError::Overflow {})?;

    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available)?;

    Ok(Response::new()
        .add_attribute("action", "add_liquidity")
        .add_attribute("total_asset_available", total_asset_available.to_string())
        .add_attribute("total_collateral_available", total_collateral_available.to_string()))
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
                denom: asset_info.denom.clone().to_string(),
                amount: total_asset_available,
            },
            Coin {
                denom: collateral_info.denom.clone().to_string(),
                amount: total_collateral_available,
            },
        ],
    };
    Ok(Response::new().add_message(bank_msg))
}

fn calculate_simple_interest(principal: Uint128, interest_rate: Uint128, time_period: u64) -> Uint128 {
    principal * interest_rate * Uint128::from(time_period) / Uint128::from(NANOSECONDS_IN_YEAR)
}

fn calculate_collateral_amount(borowing_amount: Uint128, strike_price: Uint128, overcollateralization_factor: Uint128) -> Uint128 {
    (borowing_amount * overcollateralization_factor) / strike_price
}


fn get_time_period(now: Timestamp, time: Timestamp) -> u64 {
    now.seconds() - time.seconds()
}

// Docs: COMPLETED
// This function is used to make a deposit from the user's account to th
// contract account and after that it implements the logic of the deposit

fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: DepositMsg,
) -> Result<Response, ContractError> {
    // This nonpayable function ensures that no coins are sent to the contract
    nonpayable(&info);

    let deposit_details: DepositMsg = from_json(&to_json_binary(&msg)?)?;

    let asset_config = ASSET_CONFIG.load(deps.storage)?;
    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;

    if asset_config.denom != deposit_details.denom {
        return Err(ContractError::InvalidAsset {});
    }
  
    let asset_amount = deposit_details.amount;
    
    let now = env.block.time.seconds();

    if now > pool_config.maturationdate {
        return Err(ContractError::PoolMatured {});
    }

    // This contract needs to check if the user has given enough allowance to this contract to transfer funds on a  cw20 token detailed in msg
    // asset config is the configuration of the asset that the user wants to deposit

    // check with asset cw20 if user has given allowance to this contract to transfer funds
    let allowance_and_expiry: AllowanceResponse = fetch_allowance(
        deps.as_ref(),
        env.contract.address.clone(),
        info.sender.clone(),
        asset_config.denom.clone(),
    )?;

    // if the allowance is lesser than the msg.amount to transfer, 
    // or the allowance has expired, then return an error
    if allowance_and_expiry.allowance < asset_amount {
        return Err(ContractError::InsufficientAllowance {});
    }
    else if allowance_and_expiry.expires.is_expired(&env.block) {
        return Err(ContractError::AllowanceExpired {});
    }


    // Preparing the msg for transferring the funds here.
    // We need to send this msg to the cw20 contract to transfer funds from the user's account to this contract account
    let transfer_msg = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.clone().to_string(),
        recipient: env.contract.address.clone().to_string(),
        amount: asset_amount,
    };
    
    // from here we have deposit logic

     let principle_deployed = PRINCIPLE_DEPLOYED.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
     let interest_earned_by_user = INTEREST_EARNED.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());
 
     let time_period = get_time_period(Timestamp::from_seconds(now), principle_deployed.1);
     let interest_since_last_deposit = calculate_simple_interest(principle_deployed.0, pool_config.lendinterestrate, time_period);
 
     let principle_to_deposit = asset_amount;
 
     INTEREST_EARNED.save(deps.storage, &info.sender, &(interest_earned_by_user + interest_since_last_deposit))?;
     PRINCIPLE_DEPLOYED.save(deps.storage, &info.sender, &(principle_deployed.0 + principle_to_deposit, Timestamp::from_seconds(now)))?;
 
     let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
     total_asset_available += principle_to_deposit;
     TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;
 

    let msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: asset_config.denom.clone().to_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    });

    Ok(Response::new()
    .add_attribute("action", "deposit")
    .add_submessage(msg))
}



// There's no fund to be added to the contract here.
// Here the contract will send a Transfer call from itself to the token
// to send money to the user!!!
fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: WithdrawMsg,
) -> ContractResult<Response> {
    // This nonpayable function ensures that no coins are sent to the contract
    nonpayable(&info);

    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let asset_config: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    let withdraw_details: WithdrawMsg = from_json(&to_json_binary(&msg)?)?;
    let withdraw_amount = withdraw_details.amount;

    if asset_config.denom != withdraw_details.denom {
        return Err(ContractError::InvalidAsset {});
    }
   
    let (principle_deployed, last_deposit_time) = PRINCIPLE_DEPLOYED.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let interest_earned_by_user = INTEREST_EARNED.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());

    if principle_deployed == Uint128::zero() {
        return Err(ContractError::PositionNotAvailable {});
    } else if principle_deployed < withdraw_amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // compare which is earlier, now or pool config maturity, return which ever is earlier
    // this ensures that user is not earing interest after the pool has matured
    let min_time = std::cmp::min(now, pool_config.maturationdate);

    let time_period = get_time_period(Timestamp::from_seconds(min_time), last_deposit_time);
    let interest = calculate_simple_interest(principle_deployed, pool_config.lendinterestrate, time_period);

    INTEREST_EARNED.save(deps.storage, &info.sender, &(interest_earned_by_user + interest))?;
    PRINCIPLE_DEPLOYED.save(deps.storage, &info.sender, &(principle_deployed - withdraw_amount, Timestamp::from_seconds(now)))?;

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    total_asset_available -= withdraw_amount;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    // Preparing the msg for transferring the funds here.
    // We need to send this msg to the cw20 contract to transfer funds from the  contract's account to user's account
    let transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient : info.sender.clone().to_string(),
        amount: withdraw_amount,
    };

    let msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: asset_config.denom.clone().to_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    });

    Ok(Response::new()
    .add_attribute("action", "withdraw")
    .add_submessage(msg))
}

fn execute_withdraw_interest(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    // This nonpayable function ensures that no coins are sent to the contract
    nonpayable(&info);

    let interest_earned_by_user = INTEREST_EARNED.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());

    // TODO: could have used a 'revert if no interest' here

    INTEREST_EARNED.save(deps.storage, &info.sender, &Uint128::zero())?;

    let asset_config: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    
    // Preparing the msg for transferring the funds here.
    // We need to send this msg to the cw20 contract to transfer funds from the contract's account to user's account
    let transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient : info.sender.clone().to_string(),
        amount: interest_earned_by_user,
    };

    let msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: asset_config.denom.clone().to_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    });

    Ok(Response::new()
    .add_attribute("action", "withdraw_interest")
    .add_submessage(msg))
}

fn execute_loan(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: LoanMsg,
) -> Result<Response, ContractError> {
    // This nonpayable function ensures that no coins are sent to the contract
    nonpayable(&info);

    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    if now > pool_config.maturationdate {
        return Err(ContractError::PoolMatured {});
    }

    if pool_config.overcollateralizationfactor < Uint128::new(1) {
        return Err(ContractError::InsufficientOCF {});
    }

    let tokens_details: LoanMsg = from_json(&to_json_binary(&msg)?)?;
    let asset_config: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let collateral_config: CoinConfig = COLLATERAL_CONFIG.load(deps.storage)?;

    if asset_config.denom != tokens_details.asset_denom {
        return Err(ContractError::InvalidAsset {});
    } 
    if collateral_config.denom != tokens_details.collateral_denom {
        return Err(ContractError::InvalidCollateral {});
    }

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    if total_asset_available < tokens_details.asset_amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // This contract needs to check if the user has given enough allowance to this contract to transfer funds from collateral cw20 token
    // collateral config is the configuration of the collateral that the user wants to stake
    // check with collateral cw20 if user has given allowance to this contract to transfer funds
    let allowance_and_expiry: AllowanceResponse = fetch_allowance(
        deps.as_ref(),
        env.contract.address.clone(),
        info.sender.clone(),
        collateral_config.denom.clone(),
    )?;

    // calculate the needful

    let (overall_collateral_submitted_by_user, last_collateral_time) = COLLATERAL_SUBMITTED.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let (overall_principle_to_repay_by_user, last_principle_time) = PRINCIPLE_TO_REPAY.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let overall_interest_to_repay_by_user = INTEREST_TO_REPAY.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());

    if last_collateral_time != last_principle_time {
        return Err(ContractError::InvalidState {});
    }

    // calculate the collateral needed for current loan

    let new_collateral_needed = calculate_collateral_amount(tokens_details.asset_amount, pool_config.strikeprice, pool_config.overcollateralizationfactor);

    // check if the user has approved collateral_needed amount

    if allowance_and_expiry.allowance < new_collateral_needed {
        return Err(ContractError::InsufficientAllowance {});
    }
    if allowance_and_expiry.expires.is_expired(&env.block) {
        return Err(ContractError::AllowanceExpired {});
    }

    // calculating new position interest

    let old_time_period = get_time_period(Timestamp::from_seconds(now), last_principle_time);
    let interest_on_old_principle = calculate_simple_interest(overall_principle_to_repay_by_user, pool_config.debtinterestrate, old_time_period);
    
    INTEREST_TO_REPAY.save(deps.storage, &info.sender, &(overall_interest_to_repay_by_user + interest_on_old_principle))?;
    PRINCIPLE_TO_REPAY.save(deps.storage, &info.sender, &(overall_principle_to_repay_by_user + tokens_details.asset_amount, Timestamp::from_seconds(now)))?;
    COLLATERAL_SUBMITTED.save(deps.storage, &info.sender, &(overall_collateral_submitted_by_user + new_collateral_needed, Timestamp::from_seconds(now)))?;

    total_asset_available -= tokens_details.asset_amount;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    let mut total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;
    total_collateral_available += new_collateral_needed;
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available)?;

    // Now we need to firstly transfer the collateral from the user's account to the contract account
    // then we need to transfer the asset from the contract account to the user's account

    // Preparing the msg for transferring the funds here.
    // Using Transfer since contract's account is to be used by contract itself
    // We need to send this msg to the cw20 contract to transfer funds from the user's account to this contract account
    let collateral_transfer_request = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.clone().to_string(),
        recipient: env.contract.address.clone().to_string(),
        amount: new_collateral_needed,
    };

    let collateral_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: collateral_config.denom.clone().to_string(),
        msg: to_json_binary(&collateral_transfer_request)?,
        funds: vec![],
    });

    // Preparing the msg for transferring the funds here.
    // We need to send this msg to the cw20 contract to transfer funds from the contract's account to this user's account
    let asset_transfer_request = Cw20ExecuteMsg::Transfer {
        recipient: info.sender.clone().to_string(),
        amount: tokens_details.asset_amount,
    };

    let asset_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: asset_config.denom.clone().to_string(),
        msg: to_json_binary(&asset_transfer_request)?,
        funds: vec![],
    });

    Ok(Response::new()
    .add_attribute("action", "loan")
    .add_submessage(collateral_msg)
    .add_submessage(asset_msg))

}


// Just like in borrow there's a transfer of two tokens,
// here also there's a transfer of two tokens
// wherein, this time it's reversed,
// user sends asset sum of principal and interest to the pool 
// and receives collateral in return
// Let's assume it to be full repay on the UI level
// This function is used to repay the loan taken by the user


fn execute_repay(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg : RepayMsg,
) -> ContractResult<Response> {
    // This nonpayable function ensures that no coins are sent to the contract
    nonpayable(&info);

    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    if now > pool_config.maturationdate {
        return Err(ContractError::CollateralForfeited {});
    }

    if pool_config.overcollateralizationfactor < Uint128::new(1) {
        return Err(ContractError::InsufficientOCF {});
    }

    let tokens_details: RepayMsg = from_json(&to_json_binary(&msg)?)?;
    let asset_config: CoinConfig = ASSET_CONFIG.load(deps.storage)?;
    let collateral_config: CoinConfig = COLLATERAL_CONFIG.load(deps.storage)?;

    if asset_config.denom != tokens_details.asset_denom {
        return Err(ContractError::InvalidAsset {});
    } 
    if collateral_config.denom != tokens_details.collateral_denom {
        return Err(ContractError::InvalidCollateral {});
    }

    let (overall_collateral_submitted_by_user, last_collateral_time) = COLLATERAL_SUBMITTED.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let (overall_principle_to_repay_by_user, last_principle_time) = PRINCIPLE_TO_REPAY.may_load(deps.storage, &info.sender)?.unwrap_or((Uint128::zero(), Timestamp::from_seconds(0)));
    let interest_to_repay_by_user_yet = INTEREST_TO_REPAY.may_load(deps.storage, &info.sender)?.unwrap_or(Uint128::zero());

    // TODO: what if somehow - (define) some collateral is left
    if interest_to_repay_by_user_yet == Uint128::zero() && overall_principle_to_repay_by_user == Uint128::zero() {
        return Err(ContractError::PositionNotAvailable {});
    }

    if last_collateral_time != last_principle_time {
        return Err(ContractError::InvalidState {});
    }

    let current_time_period = get_time_period(Timestamp::from_seconds(now), last_principle_time);
    let interest_on_current_principle = calculate_simple_interest(overall_principle_to_repay_by_user, pool_config.debtinterestrate, current_time_period);

    let total_interest_to_pay = interest_to_repay_by_user_yet + interest_on_current_principle;
    let total_loan_to_repay = overall_principle_to_repay_by_user + total_interest_to_pay;
    let total_collateral_to_unlock = calculate_collateral_amount(total_loan_to_repay, pool_config.strikeprice, pool_config.overcollateralizationfactor);

    // I am fine with user repaying lesser than they have debt for since it's can also be a partial repayment.

    // checking approval for contract to take user's tokens and making transaction message
    let allowance_and_expiry: AllowanceResponse = fetch_allowance(
        deps.as_ref(),
        env.contract.address.clone(),
        info.sender.clone(),
        asset_config.denom.clone(),
    )?;

    if allowance_and_expiry.expires.is_expired(&env.block) {
        return Err(ContractError::AllowanceExpired {});
    }

    let loan_user_is_repaying = tokens_details.asset_principle;
    let appropriate_collateral_to_unlock = (total_collateral_to_unlock * loan_user_is_repaying) / total_loan_to_repay;
    let interest_user_has_to_pay = calculate_simple_interest(loan_user_is_repaying, pool_config.debtinterestrate, current_time_period) + interest_to_repay_by_user_yet;
    
    if allowance_and_expiry.allowance < loan_user_is_repaying + interest_user_has_to_pay {
        return Err(ContractError::InsufficientAllowance {});
    }

    INTEREST_TO_REPAY.save(deps.storage, &info.sender, &(total_interest_to_pay - interest_user_has_to_pay))?;
    PRINCIPLE_TO_REPAY.save(deps.storage, &info.sender, &(overall_principle_to_repay_by_user - loan_user_is_repaying, Timestamp::from_seconds(now)))?;
    COLLATERAL_SUBMITTED.save(deps.storage, &info.sender, &(overall_collateral_submitted_by_user - appropriate_collateral_to_unlock, Timestamp::from_seconds(now)))?;

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage)?;
    total_asset_available -= loan_user_is_repaying;
    TOTAL_ASSET_AVAILABLE.save(deps.storage, &total_asset_available)?;

    let mut total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage)?;
    total_collateral_available -= appropriate_collateral_to_unlock;
    TOTAL_COLLATERAL_AVAILABLE.save(deps.storage, &total_collateral_available)?;

    let mut total_protocol_earning = TOTAL_PROTOCOL_EARNINGS.load(deps.storage)?;
    total_protocol_earning += interest_user_has_to_pay;
    TOTAL_PROTOCOL_EARNINGS.save(deps.storage, &total_protocol_earning)?;

    // Transfer the tokens
    let collateral_transfer_request = Cw20ExecuteMsg::Transfer {
        recipient: info.sender.clone().to_string(),
        amount: appropriate_collateral_to_unlock,
    };

    let collateral_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: collateral_config.denom.clone().to_string(),
        msg: to_json_binary(&collateral_transfer_request)?,
        funds: vec![],
    });

    // Preparing the msg for transferring the funds here.
    // We need to send this msg to the cw20 contract to transfer funds from the contract's account to this user's account
    let asset_transfer_request = Cw20ExecuteMsg::TransferFrom  { 
        owner: info.sender.clone().to_string(),
        recipient: env.contract.address.clone().to_string(),
        amount: loan_user_is_repaying + interest_user_has_to_pay
    };

    let asset_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: asset_config.denom.clone().to_string(),
        msg: to_json_binary(&asset_transfer_request)?,
        funds: vec![],
    });

    Ok(Response::new()
    .add_attribute("action", "repay")
    .add_submessage(asset_msg)
    .add_submessage(collateral_msg))    

}