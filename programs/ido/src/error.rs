use anchor_lang::prelude::*;

#[error_code]
pub enum IdoError {
    #[msg("Invalid round time")]
    InvalidRoundTime,
    #[msg("Sale round cannot be ended yet")]
    CannotEndSaleRound,
    #[msg("Trade round cannot be ended yet")]
    CannotEndTradeRound,
    #[msg("Round already started")]
    RoundAlreadyStarted,
    #[msg("This operation can only be invoked during sale round")]
    NotSaleRound,
    #[msg("This operation can only be invoked during trade round")]
    NotTradeRound,
    #[msg("Ido is over")]
    IdoIsOver,
}
