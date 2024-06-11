
use std::convert::TryInto;
use crate::external::query_price;

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

use crate::state::{POOL_CONFIG,  ASSETS, ASSET_INFO, ADMIN, UserAssetInfo, AssetConfig, AssetInfo, GLOBAL_DATA, GlobalData, RATE_DENOMINATOR, NANOSECONDS_IN_YEAR};
use crate::query::query_handler;



// TODO : functions to be made
// quoteDeposit
// deposit


pub fn quoteDeposit(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
  ) -> Result<Response, ContractError> {
    update(&mut deps, &env, &info.sender)?;
  
  
    for coin in info.funds.iter() { 
        // Fetch global cumulative interest for this asset
        let mut asset_info = ASSET_INFO.load(deps.storage, &coin.denom)?;
  
        USER_ASSET_INFO.update(
            deps.storage, 
            (&info.sender, &coin.denom),
            |user_asset| -> StdResult<UserAssetInfo> {
                match user_asset {
                    Some(mut user_asset) => {
                        user_asset.collateral += coin.amount;
                        Ok(user_asset)
                    },
                    None => {
                        Ok(UserAssetInfo {
                            collateral: coin.amount,
                            borrow_amount: Uint128::zero(),
                            l_asset_amount: Uint128::zero(),
                            cumulative_interest: asset_info.cumulative_interest,
                        })
                    }
                }
            }
        )?;
  
        asset_info.total_collateral += coin.amount;
  
        ASSET_INFO.save(deps.storage, &coin.denom, &asset_info)?;
    }
  
    Ok(Response::default())
  }

pub fn deposit(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
  ) -> Result<Response, ContractError> {
    update(&mut deps, &env, &info.sender)?;
  
  
    for coin in info.funds.iter() { 
        // Fetch global cumulative interest for this asset
        let mut asset_info = ASSET_INFO.load(deps.storage, &coin.denom)?;
  
        USER_ASSET_INFO.update(
            deps.storage, 
            (&info.sender, &coin.denom),
            |user_asset| -> StdResult<UserAssetInfo> {
                match user_asset {
                    Some(mut user_asset) => {
                        user_asset.collateral += coin.amount;
                        Ok(user_asset)
                    },
                    None => {
                        Ok(UserAssetInfo {
                            collateral: coin.amount,
                            borrow_amount: Uint128::zero(),
                            l_asset_amount: Uint128::zero(),
                            cumulative_interest: asset_info.cumulative_interest,
                        })
                    }
                }
            }
        )?;
  
        asset_info.total_collateral += coin.amount;
  
        ASSET_INFO.save(deps.storage, &coin.denom, &asset_info)?;
    }
  
    Ok(Response::default())
  }
