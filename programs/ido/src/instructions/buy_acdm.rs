use crate::{account::*, error::*, helpers::*, referral::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct BuyAcdm<'info> {
    #[account(seeds = [b"ido"], bump = ido.bump)]
    ido: Account<'info, Ido>,
    #[account(mut, associated_token::authority = ido, associated_token::mint = ido.acdm_mint)]
    ido_acdm: Account<'info, TokenAccount>,
    #[account(mut, associated_token::authority = ido, associated_token::mint = ido.usdc_mint)]
    ido_usdc: Account<'info, TokenAccount>,
    buyer: Signer<'info>,
    #[account(seeds = [b"member", buyer.key().as_ref()], bump = buyer_member.bump)]
    buyer_member: Account<'info, Member>,
    #[account(mut)]
    buyer_acdm: Account<'info, TokenAccount>,
    #[account(mut)]
    buyer_usdc: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}
impl<'info> BuyAcdm<'info> {
    fn transfer_acdm(&self, amount: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[b"ido".as_ref(), &[self.ido.bump]]];
        let cpi_accounts = Transfer {
            from: self.ido_acdm.to_account_info(),
            to: self.buyer_acdm.to_account_info(),
            authority: self.ido.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)
    }
}

pub fn buy_acdm<'info>(
    ctx: Context<'_, '_, '_, 'info, BuyAcdm<'info>>,
    acdm_amount: u64,
) -> Result<()> {
    is_sale_round(&ctx.accounts.ido)?;

    let usdc_amount_to_ido = acdm_amount
        .checked_mul(ctx.accounts.ido.acdm_price)
        .ok_or(IdoError::Overflow)?; // 100%
    let usdc_amount_to_referer = usdc_amount_to_ido / 20; // 5%
    let usdc_amount_to_referer2 = usdc_amount_to_ido
        .checked_mul(3)
        .ok_or(IdoError::Overflow)?
        / 100; // 3%

    send_to_referers_and_ido(
        usdc_amount_to_ido,
        usdc_amount_to_referer,
        usdc_amount_to_referer2,
        &ctx.accounts.buyer_member,
        &ctx.accounts.buyer,
        &ctx.accounts.buyer_usdc,
        &ctx.accounts.ido_usdc,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    ctx.accounts.transfer_acdm(acdm_amount)?;

    emit!(BuyAcdmEvent {
        buyer: ctx.accounts.buyer.key(),
        amount: acdm_amount,
    });

    Ok(())
}

#[event]
struct BuyAcdmEvent {
    buyer: Pubkey,
    amount: u64,
}
