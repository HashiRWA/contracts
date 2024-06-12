use cosmwasm_std::{
    entry_point, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, Timestamp, Uint128,
};
use crate::error::{ContractError, ContractResult};
use crate::msg::{InstantiateMsg, QueryMsg, TransactMsg, ExecuteMsg};
use crate::state::{
    ADMIN, ASSET_CONFIG, COLLATERAL_CONFIG, PRINCIPLE_TO_REPAY, COLLATERAL_SUBMITTED, INTEREST_EARNED, INTEREST_TO_REPAY,
    NANOSECONDS_IN_YEAR, POOL_CONFIG, PRINCIPLE_DEPLOYED, TOTAL_ASSET_AVAILABLE, TOTAL_COLLATERAL_AVAILABLE,
};
use crate::types::{CoinConfig, PoolConfig};
use crate::query::query_handler;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::to_binary;
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

    // Initialize asset and collateral configurations
    let asset_config = CoinConfig {
        denom: msg.pool_config.asset_address.to_string(),
        decimals: 6,  
    };
    ASSET_CONFIG.save(deps.storage, &asset_config)?;

    let collateral_config = CoinConfig {
        denom: msg.pool_config.collateral_address.to_string(),
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
            TransactMsg::AddLiquidity {} => add_liquidity(deps, env, info),
            TransactMsg::Deposit {} => deposit(deps, env, info),
            TransactMsg::WithdrawInterest {} => withdraw_interest(deps, env, info),
            TransactMsg::Withdraw { amount } => withdraw(deps, env, info, amount),
            TransactMsg::Borrow { amount } => borrow(deps, env, info, amount),
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

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
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
            // Handle this query, for example, return the list of assets
            // Implement the logic for handling Assets query
            Ok(to_json_binary(&"Assets query response")?)
        },
        QueryMsg::UserAssetsInfo { user } => {
            // Handle this query, for example, return user's asset info
            // Implement the logic for handling UserAssetsInfo query
            Ok(to_json_binary(&format!("User assets info for {}", user))?)
        },
        QueryMsg::UserAssetInfo { user, denom } => {
            // Handle this query, for example, return user's specific asset info
            // Implement the logic for handling UserAssetInfo query
            Ok(to_json_binary(&format!("User asset info for {} and denom {}", user, denom))?)
        },
        QueryMsg::UserData { user } => {
            // Handle this query, for example, return user's data
            // Implement the logic for handling UserData query
            Ok(to_json_binary(&format!("User data for {}", user))?)
        },
        QueryMsg::AssetInfo { denom } => {
            // Handle this query, for example, return asset info
            // Implement the logic for handling AssetInfo query
            Ok(to_json_binary(&format!("Asset info for denom {}", denom))?)
        },
        QueryMsg::AssetsInfo {} => {
            // Handle this query, for example, return all assets info
            // Implement the logic for handling AssetsInfo query
            Ok(to_json_binary(&"All assets info")?)
        },
        QueryMsg::MaxLiquidationAmount { user } => {
            // Handle this query, for example, return the maximum liquidation amount for the user
            // Implement the logic for handling MaxLiquidationAmount query
            Ok(to_json_binary(&format!("Max liquidation amount for {}", user))?)
        },
    }
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
        None => return Err(ContractError::InvalidFunds { denom: asset_info.denom.clone() }),
    };

    let collateral_amount = match info.funds.iter().find(|coin| coin.denom == collateral_info.denom) {
        Some(coin) => coin.amount,
        None => return Err(ContractError::InvalidFunds { denom: collateral_info.denom.clone() }),
    };

    let mut total_asset_available = TOTAL_ASSET_AVAILABLE.load(deps.storage).unwrap_or(Uint128::zero());
    let mut total_collateral_available = TOTAL_COLLATERAL_AVAILABLE.load(deps.storage).unwrap_or(Uint128::zero());

    // Safely add the amounts to the current totals
    total_asset_available = total_asset_available.checked_add(asset_amount)
        .map_err(|_| ContractError::Overflow {})?;
    total_collateral_available = total_collateral_available.checked_add(collateral_amount)
        .map_err(|_| ContractError::Overflow {})?;

    // Save updated totals
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
            pool_config: PoolConfig {
                pool_name: "Test Pool".to_string(),
                pool_symbol: "TP".to_string(),
                pool_maturation_date: 1950000000,  
                pool_debt_interest_rate: Uint128::new(5),
                pool_strike_price: Uint128::new(2),
                pool_lend_interest_rate: Uint128::new(3),
                min_overcollateralization_factor: Uint128::new(2),
                asset_address: Addr::unchecked("asset_token"),
                collateral_address: Addr::unchecked("collateral_token"),
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
    
        // Instantiate the contract
        let contract_addr = instantiate_contract(&mut app, owner);

        // Mock initial balances for the user
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

        // Define the amounts to add as liquidity
        let asset_amount = Uint128::new(1000);
        let collateral_amount = Uint128::new(2000);

        let msg = ExecuteMsg::Transact(TransactMsg::AddLiquidity {});

        let info = mock_info(user.as_str(), &[
            Coin { denom: "asset_token".to_string(), amount: asset_amount },
            Coin { denom: "collateral_token".to_string(), amount: collateral_amount },
        ]);

        // Execute the add_liquidity message
        let result = app.execute_contract(user.clone(), contract_addr.clone(), &msg, &info.funds);

        // Print the result for debugging
        if result.is_err() {
            println!("Execution failed: {:?}", result.unwrap_err());
            assert!(false, "Execution should succeed");
        } else {
            // Print the response for debugging
            println!("Execution succeeded: {:?}", result.unwrap());
        }

        // Query and verify the updated totals
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
        let mut app = App::default();
        let owner = "owner";
        let user = Addr::unchecked("user");
    
        // Instantiate the contract
        let contract_addr = instantiate_contract(&mut app, owner);

        // Mock initial balances for the user
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

    // #[test]
    // fn test_withdraw() {
    //     let mut app = App::default();
    //     let owner = "owner";

    //     let contract_addr = instantiate_contract(&mut app, owner);

    //     let asset_amount = Uint128::new(1000);
    //     let withdraw_amount = Uint128::new(500);

    //     let msg = ExecuteMsg::Transact(TransactMsg::Deposit {});

    //     let user = "user";
    //     let info = mock_info(user, &[
    //         Coin { denom: "asset_token".to_string(), amount: asset_amount },
    //     ]);

    //     app.execute_contract(Addr::unchecked(user), contract_addr.clone(), &msg, &info.funds).unwrap();

    //     let msg = ExecuteMsg::Transact(TransactMsg::Withdraw { amount: withdraw_amount });

    //     app.execute_contract(Addr::unchecked(user), contract_addr.clone(), &msg, &[]).unwrap();

    //     let total_asset_available: Uint128 = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetTotalAssetAvailable {}).unwrap();
    //     assert_eq!(total_asset_available, asset_amount - withdraw_amount);

    //     let user_principle: (Uint128, Timestamp) = app.wrap().query_wasm_smart(contract_addr, &QueryMsg::GetUserPrinciple { user: user.to_string() }).unwrap();
    //     assert_eq!(user_principle.0, asset_amount - withdraw_amount);
    // }

    // #[test]
    // fn test_borrow() {
    //     let mut app = App::default();
    //     let owner = "owner";

    //     let contract_addr = instantiate_contract(&mut app, owner);

    //     let asset_amount = Uint128::new(1000);
    //     let collateral_amount = Uint128::new(2000);
    //     let borrow_amount = Uint128::new(500);

    //     let msg = ExecuteMsg::Transact(TransactMsg::AddLiquidity {});

    //     let info = mock_info(owner, &[
    //         Coin { denom: "asset_token".to_string(), amount: asset_amount },
    //         Coin { denom: "collateral_token".to_string(), amount: collateral_amount },
    //     ]);

    //     app.execute_contract(Addr::unchecked(owner), contract_addr.clone(), &msg, &info.funds).unwrap();

    //     let msg = ExecuteMsg::Transact(TransactMsg::Borrow { amount: borrow_amount });

    //     let user = "user";
    //     let info = mock_info(user, &[
    //         Coin { denom: "collateral_token".to_string(), amount: collateral_amount },
    //     ]);

    //     app.execute_contract(Addr::unchecked(user), contract_addr.clone(), &msg, &info.funds).unwrap();

    //     let total_asset_available: Uint128 = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetTotalAssetAvailable {}).unwrap();
    //     assert_eq!(total_asset_available, asset_amount - borrow_amount);

    //     let user_principle: (Uint128, Timestamp) = app.wrap().query_wasm_smart(contract_addr, &QueryMsg::GetUserPrincipleToRepay { user: user.to_string() }).unwrap();
    //     assert_eq!(user_principle.0, borrow_amount);
    // }
}
