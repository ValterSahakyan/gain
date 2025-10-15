use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Not initialized")]
    NotInitialized,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Sale is paused")]
    SalePaused,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid treasury address")]
    InvalidTreasury,
    #[msg("Invalid mint address")]
    InvalidMint,
    #[msg("Insufficient payment")]
    InsufficientPayment,
    #[msg("Sale has ended")]
    SaleEnded,
    #[msg("Round limit exceeded")]
    RoundLimitExceeded,
    #[msg("Invalid config account")]
    InvalidConfig,
    #[msg("Invalid price account")]
    InvalidPriceAccount,
    #[msg("Price not available")]
    PriceNotAvailable,
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Stale price")]
    StalePrice,
    #[msg("Price too volatile")]
    PriceTooVolatile,
    #[msg("Pyth price account not found")]
    PythAccountNotFound,
    #[msg("Invalid Pyth account")]
    InvalidPythAccount,
    #[msg("Invalid USDC mint address")]
    InvalidUsdcMint,
    #[msg("Invalid EURC mint address")]
    InvalidEurcMint,
}