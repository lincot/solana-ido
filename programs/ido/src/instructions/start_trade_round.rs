use crate::{account::*, error::*, helpers::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct StartTradeRound<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    ido: Account<'info, Ido>,
    #[account(mut, address = ido.authority)]
    ido_authority: Signer<'info>,
    #[account(mut, address = ido.acdm_mint)]
    acdm_mint: Account<'info, Mint>,
    #[account(mut, associated_token::authority = ido, associated_token::mint = acdm_mint)]
    ido_acdm: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}
impl<'info> StartTradeRound<'info> {
    fn burn_acdm(&self) -> Result<()> {
        if self.ido_acdm.amount == 0 {
            return Ok(());
        }

        let signer: &[&[&[u8]]] = &[&[b"ido".as_ref(), &[self.ido.bump]]];
        let cpi_accounts = Burn {
            mint: self.acdm_mint.to_account_info(),
            from: self.ido_acdm.to_account_info(),
            authority: self.ido.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::burn(cpi_ctx, self.ido_acdm.amount)
    }

    fn can_start_trade_round(&self, ts: u32) -> Result<()> {
        match self.ido.state {
            IdoState::NotStarted => err!(IdoError::NotSaleRound),
            IdoState::SaleRound => {
                if self.ido_acdm.amount == 0 {
                    return Ok(());
                }

                round_time_over(&self.ido, ts)
            }
            IdoState::TradeRound => err!(IdoError::RoundAlreadyStarted),
            IdoState::Over => err!(IdoError::IdoIsOver),
        }
    }
}

pub fn start_trade_round(ctx: Context<StartTradeRound>) -> Result<()> {
    let ts = Clock::get()?.unix_timestamp as u32;

    ctx.accounts.can_start_trade_round(ts)?;

    ctx.accounts.ido.state = IdoState::TradeRound;
    ctx.accounts.ido.current_state_start_ts = ts;
    ctx.accounts.ido.usdc_traded = 0;

    ctx.accounts.burn_acdm()?;

    emit!(StartTradeRoundEvent {});

    Ok(())
}

#[event]
struct StartTradeRoundEvent {}
