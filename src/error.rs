use core::error;

use cosmwasm_std::{Coin, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {

    #[error("pool matured")]
    PoolMatured {},

    #[error("pool not matured")]
    PoolNotMatured {},

    #[error("undercollateralized")]
    Undercollateralized {},

    #[error("excessive funds")]
    ExcessiveFunds {},

    #[error("pool expired and collateral forfeited")]
    CollateralForfeited {},


    #[error("invalid state")]
    InvalidState {},


    #[error("invalid asset")]
    InsufficientOCF {},


    #[error("invalid asset")]
    InsufficientCollateral {},

    #[error("position is not available")]
    PositionNotAvailable {},

    #[error("expired option (expired {expired:?})")]
    OptionExpired { expired: u64 },

    #[error("not expired option (expires {expires:?})")]
    OptionNotExpired { expires: u64 },

    #[error("unauthorized")]
    Unauthorized {},

}

pub type ContractResult<T> = Result<T, ContractError>;
