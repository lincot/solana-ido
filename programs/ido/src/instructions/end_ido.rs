use crate::{account::*, error::*, helpers::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct EndIdo<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    ido: Account<'info, Ido>,
    #[account(address = ido.authority)]
    ido_authority: Signer<'info>,
}
impl<'info> EndIdo<'info> {
    fn can_end_ido(&self, ts: u32) -> Result<()> {
        match self.ido.state {
            IdoState::NotStarted => err!(IdoError::NotTradeRound),
            IdoState::SaleRound => err!(IdoError::NotTradeRound),
            IdoState::TradeRound => round_time_over(&self.ido, ts),
            IdoState::Over => err!(IdoError::IdoIsOver),
        }
    }
}

pub fn end_ido(ctx: Context<EndIdo>) -> Result<()> {
    let ts = Clock::get()?.unix_timestamp as u32;

    ctx.accounts.can_end_ido(ts)?;

    ctx.accounts.ido.state = IdoState::Over;
    ctx.accounts.ido.current_state_start_ts = ts;

    emit!(EndIdoEvent {});

    Ok(())
}

#[event]
struct EndIdoEvent {}
