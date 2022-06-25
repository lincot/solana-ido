use crate::{account::*, config::*, error::*, helpers::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

#[derive(Accounts)]
pub struct StartSaleRound<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    ido: Account<'info, Ido>,
    #[account(address = ido.authority)]
    ido_authority: Signer<'info>,
    acdm_mint_authority: Signer<'info>,
    #[account(mut, address = ido.acdm_mint, mint::authority = acdm_mint_authority)]
    acdm_mint: Account<'info, Mint>,
    #[account(mut, associated_token::authority = ido, associated_token::mint = acdm_mint)]
    ido_acdm: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}
impl<'info> StartSaleRound<'info> {
    fn mint_acdm(&self, amount: u64) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: self.acdm_mint.to_account_info(),
            to: self.ido_acdm.to_account_info(),
            authority: self.acdm_mint_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount)
    }

    fn can_start_sale_round(&self, ts: u32) -> Result<()> {
        match self.ido.state {
            IdoState::NotStarted => Ok(()),
            IdoState::SaleRound => err!(IdoError::RoundAlreadyStarted),
            IdoState::TradeRound => round_time_over(&self.ido, ts),
            IdoState::Over => err!(IdoError::IdoIsOver),
        }
    }
}

pub fn start_sale_round(ctx: Context<StartSaleRound>) -> Result<()> {
    let ts = Clock::get()?.unix_timestamp as u32;

    ctx.accounts.can_start_sale_round(ts)?;

    ctx.accounts.ido.state = IdoState::SaleRound;
    ctx.accounts.ido.current_state_start_ts = ts;
    ctx.accounts.ido.acdm_price = if ctx.accounts.ido.sale_rounds_started == 0 {
        INITIAL_PRICE
    } else {
        sale_price_formula(ctx.accounts.ido.acdm_price)
    };
    ctx.accounts.ido.sale_rounds_started += 1;

    let amount_to_mint = ctx.accounts.ido.usdc_traded / ctx.accounts.ido.acdm_price;
    ctx.accounts.mint_acdm(amount_to_mint)?;

    emit!(StartSaleRoundEvent {
        acdm_price: ctx.accounts.ido.acdm_price,
        minted_amount: amount_to_mint,
    });

    Ok(())
}

#[event]
struct StartSaleRoundEvent {
    acdm_price: u64,
    minted_amount: u64,
}
