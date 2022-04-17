use anchor_lang::prelude::*;

#[error_code]
pub enum IdoError {
    #[msg("Invalid round time")]
    RoundTimeInvalid,
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
    #[msg("To register with referer, one's member account must be provided")]
    RefererAccountNotProvided,
    #[msg("Invalid amount of referer accounts supplied")]
    RefererAccountsAmount,
    #[msg("Supplied account is not the PDA of user's referer")]
    RefererPda,
    #[msg("Referer must own the token account to get fees")]
    RefererOwner,
    #[msg("Ido is over")]
    IdoIsOver,
}
