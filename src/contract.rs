use cosmwasm_std::{
    entry_point, from_binary, from_json, Addr, BankMsg, Binary, BlockInfo, Coin, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, QueryRequest, Response, StdResult, SubMsg, Timestamp, Uint128, WasmMsg, WasmQuery
};
use cw20::{AllowanceResponse, Balance, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};
use cw20_base::allowances::{
    execute_transfer_from, query_allowance,
};
use crate::error::{ContractError, ContractResult};
use crate::msg::{DepositMsg, ExecuteMsg, InstantiateMsg, LoanMsg, QueryMsg, TransactMsg, WithdrawMsg};
use crate::state::{
    ADMIN, ASSET_CONFIG, COLLATERAL_CONFIG, PRINCIPLE_TO_REPAY, COLLATERAL_SUBMITTED, INTEREST_EARNED, INTEREST_TO_REPAY,
    NANOSECONDS_IN_YEAR, POOL_CONFIG, PRINCIPLE_DEPLOYED, TOTAL_ASSET_AVAILABLE, TOTAL_COLLATERAL_AVAILABLE,
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

            TransactMsg::AddLiquidity {} => add_liquidity(deps, env, info),
            TransactMsg::Repay {} => repay(deps, env, info),
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
    }
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

fn repay(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let pool_config: PoolConfig = POOL_CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    if now > pool_config.maturationdate {
        return Err(ContractError::CollateralForfeited {});
    }

    let amount_to_repay = info.funds.iter().find(|coin| coin.denom == pool_config.asset.to_string()).unwrap().amount;

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
    let interest_on_current_principle = calculate_simple_interest(principle_to_repay_by_user, pool_config.debtinterestrate, time_period);

    let strike = pool_config.strikeprice;
    let current_ocf = pool_config.overcollateralizationfactor;

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
            denom: pool_config.collateral.to_string(),
            amount: needed_collateral,
        }],
    };

    Ok(Response::new().add_message(bank_msg))
}
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coins, from_binary, Addr, BankMsg, Coin, Empty, Env, MessageInfo, Response, Storage, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

    // Create a mock contract for testing purposes
    fn mock_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    fn instantiate_contract(app: &mut App, owner: &str) -> Addr {
        let contract_id = app.store_code(mock_contract());
        let msg = InstantiateMsg {
            admin: owner.to_string(),
            config: PoolConfig {
                name: "Test Pool".to_string(),
                symbol: "TP".to_string(),
                maturationdate: 1950000000,  
                debtinterestrate: Uint128::new(5),
                strikeprice: Uint128::new(2),
                lendinterestrate: Uint128::new(3),
                overcollateralizationfactor: Uint128::new(2),
                asset: Addr::unchecked("asset_token"),
                collateral: Addr::unchecked("collateral_token"),
            },
            oracle: "oracle_address".to_string(),  
        };
        let owner_info = mock_info(owner, &[]);
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(owner), &msg, &[], "Test Contract", None)
            .unwrap();
        contract_addr
    }

    #[test]
    fn test_instantiate() {
        let mut app = App::default();
        let owner = "owner";

        let contract_addr = instantiate_contract(&mut app, owner);
        let stored_owner: Addr = app.wrap().query_wasm_smart(contract_addr, &QueryMsg::GetOwner {}).unwrap();
        
        assert_eq!(stored_owner, Addr::unchecked(owner));
    }

    #[test]
    fn test_add_liquidity() {
        let mut app = App::default();
        let owner = "owner";
        let user = Addr::unchecked("user");
    
        let contract_addr = instantiate_contract(&mut app, owner);

        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: user.to_string(),
            amount: vec![
                Coin {
                    denom: "asset_token".to_string(),
                    amount: Uint128::new(10000),
                },
                Coin {
                    denom: "collateral_token".to_string(),
                    amount: Uint128::new(20000),
                },
            ],
        })).unwrap();

        let initial_asset_balance = app.wrap().query_balance(&user, "asset_token").unwrap();
        let initial_collateral_balance = app.wrap().query_balance(&user, "collateral_token").unwrap();
        println!("Initial asset balance: {}", initial_asset_balance.amount);
        println!("Initial collateral balance: {}", initial_collateral_balance.amount);

        let asset_amount = Uint128::new(1000);
        let collateral_amount = Uint128::new(2000);

        let msg = ExecuteMsg::Transact(TransactMsg::AddLiquidity {});

        let info = mock_info(user.as_str(), &[
            Coin { denom: "asset_token".to_string(), amount: asset_amount },
            Coin { denom: "collateral_token".to_string(), amount: collateral_amount },
        ]);

        let result = app.execute_contract(user.clone(), contract_addr.clone(), &msg, &info.funds);

        if result.is_err() {
            println!("Execution failed: {:?}", result.unwrap_err());
            assert!(false, "Execution should succeed");
        } else {
            println!("Execution succeeded: {:?}", result.unwrap());
        }

        let total_asset_available: Uint128 = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetTotalAssetAvailable {}).unwrap();
        let total_collateral_available: Uint128 = app.wrap().query_wasm_smart(contract_addr, &QueryMsg::GetTotalCollateralAvailable {}).unwrap();

        assert_eq!(total_asset_available, asset_amount);
        assert_eq!(total_collateral_available, collateral_amount);

        let final_asset_balance = app.wrap().query_balance(&user, "asset_token").unwrap();
        let final_collateral_balance = app.wrap().query_balance(&user, "collateral_token").unwrap();
        println!("Final asset balance: {}", final_asset_balance.amount);
        println!("Final collateral balance: {}", final_collateral_balance.amount);

        assert_eq!(final_asset_balance.amount, Uint128::new(9000));
        assert_eq!(final_collateral_balance.amount, Uint128::new(18000));
    }


    #[test]
    fn test_deposit() {

        let mut app = App::default();
        let owner = "owner";
        let user = Addr::unchecked("user");
    
  
        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: user.to_string(),
            amount: vec![
                Coin {
                    denom: "asset_token".to_string(),
                    amount: Uint128::new(10000),
                },
                Coin {
                    denom: "collateral_token".to_string(),
                    amount: Uint128::new(20000),
                },
            ],
        })).unwrap();
        let contract_addr = instantiate_contract(&mut app, owner);

        let asset_amount = Uint128::new(1000);

        let msg = ExecuteMsg::Transact(TransactMsg::Deposit {});

        let user = "user";
        let info = mock_info(user, &[
            Coin { denom: "asset_token".to_string(), amount: asset_amount },
        ]);

        app.execute_contract(Addr::unchecked(user), contract_addr.clone(), &msg, &info.funds).unwrap();

        let total_asset_available: Uint128 = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetTotalAssetAvailable {}).unwrap();
        assert_eq!(total_asset_available, asset_amount);

        let user_principle: (Uint128, Timestamp) = app.wrap().query_wasm_smart(contract_addr, &QueryMsg::GetUserPrinciple { user: user.to_string() }).unwrap();
        assert_eq!(user_principle.0, asset_amount);
    }

    #[test]
    fn test_withdraw() {

        let mut app = App::default();
        let owner = "owner";
        let user = Addr::unchecked("user");
    
        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: user.to_string(),
            amount: vec![
                Coin {
                    denom: "asset_token".to_string(),
                    amount: Uint128::new(10000),
                },
                Coin {
                    denom: "collateral_token".to_string(),
                    amount: Uint128::new(20000),
                },
            ],
        })).unwrap();
        let contract_addr = instantiate_contract(&mut app, owner);

        let asset_amount = Uint128::new(1000);
        let withdraw_amount = Uint128::new(500);

        let msg = ExecuteMsg::Transact(TransactMsg::Deposit {});

        let user = "user";
        let info = mock_info(user, &[
            Coin { denom: "asset_token".to_string(), amount: asset_amount },
        ]);

        app.execute_contract(Addr::unchecked(user), contract_addr.clone(), &msg, &info.funds).unwrap();

        let msg = ExecuteMsg::Transact(TransactMsg::Withdraw { amount: withdraw_amount });

        app.execute_contract(Addr::unchecked(user), contract_addr.clone(), &msg, &[]).unwrap();

        let total_asset_available: Uint128 = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetTotalAssetAvailable {}).unwrap();
        assert_eq!(total_asset_available, asset_amount - withdraw_amount);

        let user_principle: (Uint128, Timestamp) = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetUserPrinciple { user: user.to_string() }).unwrap();
        assert_eq!(user_principle.0, asset_amount - withdraw_amount);
        let final_asset_balance = app.wrap().query_balance(contract_addr.clone(), "asset_token").unwrap();
        let final_collateral_balance = app.wrap().query_balance(contract_addr.clone(), "collateral_token").unwrap();
        assert_eq!(final_asset_balance.amount , withdraw_amount);    
    }

    #[test]
    fn test_borrow() {
        let mut app = App::default();
        let owner = "owner";
        let mut app = App::default();
        let owner = "owner";
        let mut app = App::default();
        let owner = "owner";
        let user = Addr::unchecked("user");
    
        let contract_addr = instantiate_contract(&mut app, owner);

        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: user.to_string(),
            amount: vec![
                Coin {
                    denom: "asset_token".to_string(),
                    amount: Uint128::new(10000),
                },
                Coin {
                    denom: "collateral_token".to_string(),
                    amount: Uint128::new(20000),
                },
            ],
        })).unwrap();

        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: owner.to_string(),
            amount: vec![
                Coin {
                    denom: "asset_token".to_string(),
                    amount: Uint128::new(10000),
                },
                Coin {
                    denom: "collateral_token".to_string(),
                    amount: Uint128::new(20000),
                },
            ],
        })).unwrap();
        let contract_addr = instantiate_contract(&mut app, owner);

        let asset_amount = Uint128::new(1000);
        let collateral_amount = Uint128::new(2000);
        let borrow_amount = Uint128::new(500);

        let msg = ExecuteMsg::Transact(TransactMsg::AddLiquidity {});
        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: contract_addr.to_string(),
            amount: vec![
                Coin {
                    denom: "asset_token".to_string(),
                    amount: Uint128::new(10000),
                },
                Coin {
                    denom: "collateral_token".to_string(),
                    amount: Uint128::new(20000),
                },
            ],
        })).unwrap();
        let info = mock_info(owner, &[
            Coin { denom: "asset_token".to_string(), amount: asset_amount },
            Coin { denom: "collateral_token".to_string(), amount: collateral_amount },
        ]);

        app.execute_contract(Addr::unchecked(owner), contract_addr.clone(), &msg, &info.funds).unwrap();

        let msg = ExecuteMsg::Transact(TransactMsg::Borrow { amount: borrow_amount });

        let user = "user";
        let info = mock_info(user, &[
            Coin { denom: "collateral_token".to_string(), amount: collateral_amount },
        ]);

        app.execute_contract(Addr::unchecked(user), contract_addr.clone(), &msg, &info.funds).unwrap();

        let total_asset_available: Uint128 = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetTotalAssetAvailable {}).unwrap();
        assert_eq!(total_asset_available, asset_amount - borrow_amount);

        let user_principle: (Uint128, Timestamp) = app.wrap().query_wasm_smart(contract_addr, &QueryMsg::GetUserPrincipleToRepay { user: user.to_string() }).unwrap();
        assert_eq!(user_principle.0, borrow_amount);
    }

    
    #[test]
    fn test_repay() {
        let mut app = App::default();
        let owner = "owner";
        let user = Addr::unchecked("user");
    
        let contract_addr = instantiate_contract(&mut app, owner);
    
        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: user.to_string(),
            amount: vec![
                Coin {
                    denom: "asset_token".to_string(),
                    amount: Uint128::new(10000),
                },
                Coin {
                    denom: "collateral_token".to_string(),
                    amount: Uint128::new(20000),
                },
            ],
        })).unwrap();
    
        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: contract_addr.to_string(),
            amount: vec![
                Coin {
                    denom: "asset_token".to_string(),
                    amount: Uint128::new(10000),
                },
            ],
        })).unwrap();
    
        let add_liquidity_msg = ExecuteMsg::Transact(TransactMsg::AddLiquidity {});
        let add_liquidity_info = mock_info(user.as_str(), &[
            Coin { denom: "asset_token".to_string(), amount: Uint128::new(1000) },
            Coin { denom: "collateral_token".to_string(), amount: Uint128::new(2000) },
        ]);
    
        app.execute_contract(user.clone(), contract_addr.clone(), &add_liquidity_msg, &add_liquidity_info.funds).unwrap();
    
        let borrow_amount = Uint128::new(500);
        let borrow_msg = ExecuteMsg::Transact(TransactMsg::Borrow { amount: borrow_amount });
        let borrow_info = mock_info(user.as_str(), &[
            Coin { denom: "collateral_token".to_string(), amount: Uint128::new(2000) },
        ]);
    
        app.execute_contract(user.clone(), contract_addr.clone(), &borrow_msg, &borrow_info.funds).unwrap();
    
        let repay_msg = ExecuteMsg::Transact(TransactMsg::Repay {});
        let repay_info = mock_info(user.as_str(), &[
            Coin { denom: "asset_token".to_string(), amount: borrow_amount },
        ]);
        let contract_asset_balance = app.wrap().query_balance(&contract_addr, "asset_token").unwrap();
        assert_eq!(contract_asset_balance.amount, Uint128::new(10000 + 1000 -500));
        let result = app.execute_contract(user.clone(), contract_addr.clone(), &repay_msg, &repay_info.funds);
    
        if result.is_err() {
            println!("Execution failed: {:?}", result.unwrap_err());
            assert!(false, "Execution should succeed");
        } else {
            println!("Execution succeeded: {:?}", result.unwrap());
        }
    
        let user_principle_to_repay: (Uint128, Timestamp) = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetUserPrincipleToRepay { user: user.to_string() }).unwrap();
        assert_eq!(user_principle_to_repay.0, Uint128::zero());
    
        let contract_asset_balance = app.wrap().query_balance(&contract_addr, "asset_token").unwrap();
        assert_eq!(contract_asset_balance.amount, Uint128::new(10000 + 1000 ));
    
        let contract_collateral_balance = app.wrap().query_balance(&contract_addr, "collateral_token").unwrap();
        assert_eq!(contract_collateral_balance.amount, Uint128::new(2000)); 
    
        println!("Contract asset balance: {}", contract_asset_balance.amount);
        println!("Contract collateral balance: {}", contract_collateral_balance.amount);
    }
    

}
