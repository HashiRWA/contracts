use cosmwasm_std::{StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Bank Contract : Invalid Asset")]
    InvalidAsset {},

    #[error("Bank Contract : Invalid Collateral")]
    InvalidCollateral {},

    #[error("Bank Contract : Std Error")]
    StdErr { kind: String, detail: String },

    #[error("Bank Contract : Unauthorized")]
    Unauthorized {},

    #[error("Bank Contract: Allowance Expired")]
    AllowanceExpired {},

    #[error("Bank Contract : InvalidFunds")]
    InvalidFunds {denom : String},

    #[error ("Bank Contract : Insufficient Allowance")]
    InsufficientAllowance {},

    #[error("Bank Contract : Pool has matured, cannot perform this operation")]
    PoolMatured {},

    #[error("Bank Contract : Pool has not matured yet")]
    PoolNotMatured {},

    #[error("Bank Contract : Position is undercollateralized")]
    Undercollateralized {},

    #[error("Bank Contract : Overflow")]
    Overflow {},

    #[error("Bank Contract : Excessive funds provided")]
    ExcessiveFunds {},

    #[error("Bank Contract : Pool expired and collateral has been forfeited")]
    CollateralForfeited {},

    #[error("Bank Contract : Invalid state detected")]
    InvalidState {},

    #[error("Bank Contract : Insufficient over-collateralization factor")]
    InsufficientOCF {},

    #[error("Bank Contract : Insufficient collateral provided")]
    InsufficientCollateral {},

    #[error("Bank Contract : Insufficient funds available")]
    InsufficientFunds {},

    #[error("Bank Contract : Position is not available for this operation")]
    PositionNotAvailable {},

    #[error("Bank Contract : Option expired (expired at {expired:?})")]
    OptionExpired { expired: u64 },

    #[error("Bank Contract : Option not expired (expires at {expires:?})")]
    OptionNotExpired { expires: u64 },
}

pub type ContractResult<T> = Result<T, ContractError>;
