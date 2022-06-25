use crate::{account::*, error::*, helpers::*, referral::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct RedeemOrder<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    ido: Box<Account<'info, Ido>>,
    #[account(mut, associated_token::authority = ido, associated_token::mint = ido.usdc_mint)]
    ido_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"order", id.to_le_bytes().as_ref()], bump = order.bump)]
    order: Account<'info, Order>,
    #[account(mut, associated_token::authority = order, associated_token::mint = ido.acdm_mint)]
    order_acdm: Account<'info, TokenAccount>,
    #[account(mut)]
    buyer: Signer<'info>,
    #[account(mut)]
    buyer_acdm: Account<'info, TokenAccount>,
    #[account(mut)]
    buyer_usdc: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(address = order.authority)]
    seller: UncheckedAccount<'info>,
    #[account(seeds = [b"member", seller.key().as_ref()], bump = seller_member.bump)]
    seller_member: Account<'info, Member>,
    #[account(mut, token::authority = order.authority)]
    seller_usdc: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}
impl<'info> RedeemOrder<'info> {
    fn transfer_usdc_to_seller(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.buyer_usdc.to_account_info(),
            to: self.seller_usdc.to_account_info(),
            authority: self.buyer.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)
    }

    fn transfer_acdm_to_buyer(&self, id: u64, amount: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[b"order".as_ref(), &id.to_le_bytes(), &[self.order.bump]]];
        let cpi_accounts = Transfer {
            from: self.order_acdm.to_account_info(),
            to: self.buyer_acdm.to_account_info(),
            authority: self.order.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)
    }
}

pub fn redeem_order<'info>(
    ctx: Context<'_, '_, '_, 'info, RedeemOrder<'info>>,
    id: u64,
    acdm_amount: u64,
) -> Result<()> {
    is_trade_round(&ctx.accounts.ido)?;

    let usdc_amount_total = acdm_amount
        .checked_mul(ctx.accounts.order.price)
        .ok_or(IdoError::Overflow)?;
    ctx.accounts.ido.usdc_traded = (ctx.accounts.ido.usdc_traded)
        .checked_add(usdc_amount_total)
        .ok_or(IdoError::Overflow)?;

    let usdc_amount_to_ido = usdc_amount_total / 20; // 5%
    let usdc_amount_to_referer = usdc_amount_to_ido / 2; // 2.5%
    let usdc_amount_to_referer2 = usdc_amount_to_ido - usdc_amount_to_referer; // 2.5%
    let usdc_amount_so_seller = usdc_amount_total - usdc_amount_to_ido; // 95%

    send_to_referers_and_ido(
        usdc_amount_to_ido,
        usdc_amount_to_referer,
        usdc_amount_to_referer2,
        &ctx.accounts.seller_member,
        &ctx.accounts.buyer,
        &ctx.accounts.buyer_usdc,
        &ctx.accounts.ido_usdc,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    ctx.accounts
        .transfer_usdc_to_seller(usdc_amount_so_seller)?;

    ctx.accounts.transfer_acdm_to_buyer(id, acdm_amount)?;

    emit!(RedeemOrderEvent {
        id,
        buyer: ctx.accounts.buyer.key(),
        amount: acdm_amount,
    });

    Ok(())
}

#[event]
struct RedeemOrderEvent {
    id: u64,
    buyer: Pubkey,
    amount: u64,
}
