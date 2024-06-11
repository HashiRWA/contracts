use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Uint128, QueryRequest, WasmQuery, Storage,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use cosmwasm_std::SystemError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] cosmwasm_std::StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Token not found")]
    TokenNotFound {},

    #[error("Serialization error")]
    SerializationError,

    #[error("Invalid input")]
    InvalidInput {},
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
    deps.storage.set(CONFIG_KEY.as_bytes(), &bincode::serialize(&config).map_err(|_| ContractError::SerializationError)?);
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
    let config: Config = match deps.storage.get(CONFIG_KEY.as_bytes()) {
        Some(data) => bincode::deserialize(&data).map_err(|_| ContractError::SerializationError)?,
        None => return Err(ContractError::Unauthorized {}),
    };

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let token_addr = deps.api.addr_validate(&token)?;
    let token_price = TokenPrice { token: token_addr.clone(), price };

    deps.storage.set(TOKEN_PRICES_KEY.as_bytes(), &bincode::serialize(&token_price).map_err(|_| ContractError::SerializationError)?);
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
            let price_info: TokenPrice = bincode::deserialize(&data).map_err(|_| cosmwasm_std::StdError::generic_err("Serialization error"))?;
            to_binary(&price_info.price)
        },
        None => Err(cosmwasm_std::StdError::not_found("Token price not found")),
    }
}

fn query_external_price(deps: Deps, oracle_addr: String, denom: String) -> StdResult<Binary> {
    let msg = QueryRequest::Wasm(
        WasmQuery::Smart {
            contract_addr: oracle_addr,
            msg: to_binary(&ExternalOracleQueryMsg { symbol: denom })?
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
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, from_binary, Uint128, QuerierResult, SystemResult, QueryRequest, WasmQuery, ContractResult};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { admin: String::from("admin_address") };
        let info = mock_info("creator", &[]);

        // Instantiate the contract
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes, vec![attr("method", "instantiate"), attr("admin", "creator")]);

        // Check the admin address is set correctly
        let config: Config = bincode::deserialize(&deps.storage.get(CONFIG_KEY.as_bytes()).unwrap()).unwrap();
        assert_eq!(config.admin, Addr::unchecked("admin_address"));
    }

    #[test]
    fn update_price() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { admin: String::from("admin_address") };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Update the price as admin
        let update_info = mock_info("admin_address", &[]);
        let update_msg = ExecuteMsg::UpdatePrice {
            token: String::from("token_address"),
            price: Uint128::new(100),
        };
        let res = execute(deps.as_mut(), mock_env(), update_info, update_msg).unwrap();
        assert_eq!(res.attributes, vec![
            attr("method", "update_price"),
            attr("token", "token_address"),
            attr("price", "100"),
        ]);

        // Check the price is stored correctly
        let price_info: TokenPrice = bincode::deserialize(&deps.storage.get(TOKEN_PRICES_KEY.as_bytes()).unwrap()).unwrap();
        assert_eq!(price_info.token, Addr::unchecked("token_address"));
        assert_eq!(price_info.price, Uint128::new(100));
    }

    #[test]
    fn unauthorized_update() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { admin: String::from("admin_address") };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Try to update the price as a non-admin
        let update_info = mock_info("not_admin", &[]);
        let update_msg = ExecuteMsg::UpdatePrice {
            token: String::from("token_address"),
            price: Uint128::new(100),
        };
        let res = execute(deps.as_mut(), mock_env(), update_info, update_msg);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});
    }

    #[test]
    fn query_price() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { admin: String::from("admin_address") };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Update the price as admin
        let update_info = mock_info("admin_address", &[]);
        let update_msg = ExecuteMsg::UpdatePrice {
            token: String::from("token_address"),
            price: Uint128::new(100),
        };
        execute(deps.as_mut(), mock_env(), update_info, update_msg).unwrap();

        // Query the price
        let query_msg = QueryMsg::GetPrice { token: String::from("token_address") };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let price: Uint128 = from_binary(&res).unwrap();
        assert_eq!(price, Uint128::new(100));
    }

    #[test]
    fn query_nonexistent_price() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { admin: String::from("admin_address") };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Query a non-existent price
        let query_msg = QueryMsg::GetPrice { token: String::from("nonexistent_token") };
        let res = query(deps.as_ref(), mock_env(), query_msg);
        assert!(res.is_err());
    }

    // #[test]
    // fn query_external_price() {
    //     let mut deps = mock_dependencies();

    //     let oracle_addr = String::from("oracle_address");
    //     let denom = String::from("usd");

    //     // Mock the response from the external oracle
    //     deps.querier.with_wasm_query(|request: &QueryRequest<ExternalOracleQueryMsg>| -> QuerierResult {
    //         match request {
    //             QueryRequest::Wasm(WasmQuery::Smart { .. }) => {
    //                 SystemResult::Ok(ContractResult::Ok(to_binary(&ExternalPriceResponse {
    //                     price: Uint128::new(123456),
    //                     precision: Uint128::new(1),
    //                 }).unwrap()))
    //             }
    //             _ => SystemResult::Err(SystemError::UnsupportedRequest {
    //                 kind: format!("{:?}", request),
    //             }),
    //         }
    //     });

    //     let query_msg = QueryMsg::QueryExternalPrice { oracle_addr: oracle_addr.clone(), denom: denom.clone() };
    //     let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    //     let price_response: ExternalPriceResponse = from_binary(&res).unwrap();
    //     assert_eq!(price_response.price, Uint128::new(123456));
    //     assert_eq!(price_response.precision, Uint128::new(1));
    // }
}
