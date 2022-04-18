use anchor_lang::prelude::*;

#[error_code]
pub enum IdoError {
    #[msg("Argument too big or too small")]
    OverflowingArgument,
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
    #[msg("Referer's member account is not provided")]
    RefererMemberAccountNotProvided,
    #[msg("Referer's token account is not provided")]
    RefererTokenAccountNotProvided,
    #[msg("Supplied account is not the PDA of user's referer")]
    RefererPda,
    #[msg("Referer must own the token account to get fees")]
    RefererOwner,
    #[msg("Ido is over")]
    IdoIsOver,
}
