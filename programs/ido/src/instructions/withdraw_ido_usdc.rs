use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct WithdrawIdoUsdc<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    ido: Account<'info, Ido>,
    #[account(mut, address = ido.authority)]
    ido_authority: Signer<'info>,
    #[account(mut, associated_token::authority = ido, associated_token::mint = ido.usdc_mint)]
    ido_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    to: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}
impl<'info> WithdrawIdoUsdc<'info> {
    fn transfer(&self) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[b"ido".as_ref(), &[self.ido.bump]]];
        let cpi_accounts = Transfer {
            from: self.ido_usdc.to_account_info(),
            to: self.to.to_account_info(),
            authority: self.ido.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, self.ido_usdc.amount)
    }
}

pub fn withdraw_ido_usdc(ctx: Context<WithdrawIdoUsdc>) -> Result<()> {
    ctx.accounts.transfer()?;

    emit!(WithdrawIdoUsdcEvent {});

    Ok(())
}

#[event]
struct WithdrawIdoUsdcEvent {}
