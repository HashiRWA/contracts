use cosmwasm_std::{StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Std Error")]
    StdErr { kind: String, detail: String },

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("InvalidFunds")]
    InvalidFunds {denom : String},


    #[error("Pool has matured, cannot perform this operation")]
    PoolMatured {},

    #[error("Pool has not matured yet")]
    PoolNotMatured {},

    #[error("Position is undercollateralized")]
    Undercollateralized {},

    #[error("Overflow")]
    Overflow {},

    #[error("Excessive funds provided")]
    ExcessiveFunds {},

    #[error("Pool expired and collateral has been forfeited")]
    CollateralForfeited {},

    #[error("Invalid state detected")]
    InvalidState {},

    #[error("Insufficient over-collateralization factor")]
    InsufficientOCF {},

    #[error("Insufficient collateral provided")]
    InsufficientCollateral {},

    #[error("Insufficient funds available")]
    InsufficientFunds {},

    #[error("Position is not available for this operation")]
    PositionNotAvailable {},

    #[error("Option expired (expired at {expired:?})")]
    OptionExpired { expired: u64 },

    #[error("Option not expired (expires at {expires:?})")]
    OptionNotExpired { expires: u64 },
}

pub type ContractResult<T> = Result<T, ContractError>;
