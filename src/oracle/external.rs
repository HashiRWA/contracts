use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Uint128, QueryRequest, WasmQuery,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] cosmwasm_std::StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Token not found")]
    TokenNotFound {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TokenPrice {
    pub token: Addr,
    pub price: Uint128,
}

const CONFIG_KEY: &str = "config";
const TOKEN_PRICES_KEY: &str = "token_prices";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ExecuteMsg {
    UpdatePrice { token: String, price: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPrice { token: String },
    QueryExternalPrice { oracle_addr: String, denom: String },
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = deps.api.addr_validate(&msg.admin)?;
    let config = Config { admin };
    deps.storage.set(CONFIG_KEY.as_bytes(), &bincode::serialize(&config)?);
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdatePrice { token, price } => try_update_price(deps, info, token, price),
    }
}

fn try_update_price(
    deps: DepsMut,
    info: MessageInfo,
    token: String,
    price: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = bincode::deserialize(&deps.storage.get(CONFIG_KEY.as_bytes()).unwrap().unwrap())?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let token_addr = deps.api.addr_validate(&token)?;
    deps.storage.set(TOKEN_PRICES_KEY.as_bytes(), &bincode::serialize(&TokenPrice { token: token_addr.clone(), price })?);
    Ok(Response::new()
        .add_attribute("method", "update_price")
        .add_attribute("token", token)
        .add_attribute("price", price))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice { token } => query_price(deps, token),
        QueryMsg::QueryExternalPrice { oracle_addr, denom } => query_external_price(deps, oracle_addr, denom),
    }
}

fn query_price(deps: Deps, token: String) -> StdResult<Binary> {
    let token_addr = deps.api.addr_validate(&token)?;
    match deps.storage.get(TOKEN_PRICES_KEY.as_bytes()) {
        Some(data) => {
            let price_info: TokenPrice = bincode::deserialize(&data)?;
            to_binary(&price_info.price)
        }
        None => Err(ContractError::TokenNotFound {})?,
    }
}

fn query_external_price(deps: Deps, oracle_addr: String, denom: String) -> StdResult<Binary> {
    let msg = QueryRequest::Wasm(
        WasmQuery::Smart { 
            contract_addr: oracle_addr, 
            msg: to_binary(&ExternalOracleQueryMsg::Price { symbol: denom })?
        }
    );
    let response: ExternalPriceResponse = deps.querier.query(&msg)?;
    to_binary(&response)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExternalOracleQueryMsg {
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExternalPriceResponse {
    pub price: Uint128,
    pub precision: Uint128,
}
