use anchor_lang::prelude::*;

#[error_code]
pub enum OracleDemoError {
    #[msg("Price is missing")]
    MissingPrice,
    #[msg("Price is stale")]
    StalePrice,
    #[msg("Price is invalid")]
    InvalidPrice,
    #[msg("Arithmetic overflow while scaling the price")]
    ArithmeticOverflow,
}
