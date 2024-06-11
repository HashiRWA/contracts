use cosmwasm_std::{
  Binary, Deps, Env, to_json_binary, Uint128
};

use crate::error::ContractResult;
use crate::msg::QueryMsg;



pub fn query_handler(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
  Ok("ok".as_bytes().into())
}
