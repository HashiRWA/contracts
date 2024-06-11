use cosmwasm_std::{
  entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

mod msg;
mod contract;

#[entry_point]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: Empty)
-> StdResult<Response>
{
  contract::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: msg::QueryMsg)
-> StdResult<Binary>
{
  contract::query(deps, env, msg)
}
