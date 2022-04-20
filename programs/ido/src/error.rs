use anchor_lang::prelude::*;

#[error_code]
pub enum IdoError {
    /// 6000 0x1770
    #[msg("Argument too big or too small")]
    OverflowingArgument,
    /// 6001 0x1771
    #[msg("Round cannot be ended yet")]
    CannotEndRound,
    /// 6002 0x1772
    #[msg("Round already started")]
    RoundAlreadyStarted,
    /// 6003 0x1773
    #[msg("This operation can only be invoked during sale round")]
    NotSaleRound,
    /// 6004 0x1774
    #[msg("This operation can only be invoked during trade round")]
    NotTradeRound,
    /// 6005 0x1775
    #[msg("Referer's member account is not provided")]
    RefererMemberAccountNotProvided,
    /// 6006 0x1776
    #[msg("Referer's token account is not provided")]
    RefererTokenAccountNotProvided,
    /// 6007 0x1777
    #[msg("Supplied account is not the PDA of user's referer")]
    RefererPda,
    /// 6008 0x1778
    #[msg("Referer must own the token account to get fees")]
    RefererOwner,
    /// 6009 0x1779
    #[msg("Ido is over")]
    IdoIsOver,
}
