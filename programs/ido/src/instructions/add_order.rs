use crate::{account::*, helpers::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct AddOrder<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    ido: Account<'info, Ido>,
    #[account(
        init,
        payer = seller,
        seeds = [b"order", ido.orders.to_le_bytes().as_ref()],
        bump,
        space = 8 + Order::LEN,
    )]
    order: Account<'info, Order>,
    #[account(mut, address = ido.acdm_mint)]
    acdm_mint: Account<'info, Mint>,
    #[account(mut, associated_token::authority = order, associated_token::mint = acdm_mint)]
    order_acdm: Account<'info, TokenAccount>,
    #[account(mut)]
    seller: Signer<'info>,
    #[account(mut)]
    seller_acdm: Account<'info, TokenAccount>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}
impl<'info> AddOrder<'info> {
    fn transfer_acdm(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.seller_acdm.to_account_info(),
            to: self.order_acdm.to_account_info(),
            authority: self.seller.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)
    }
}

pub fn add_order(ctx: Context<AddOrder>, acdm_amount: u64, acdm_price: u64) -> Result<()> {
    is_trade_round(&ctx.accounts.ido)?;

    ctx.accounts.transfer_acdm(acdm_amount)?;

    ctx.accounts.order.bump = *ctx.bumps.get("order").unwrap();
    ctx.accounts.order.authority = ctx.accounts.seller.key();
    ctx.accounts.order.price = acdm_price;

    emit!(AddOrderEvent {
        id: ctx.accounts.ido.orders,
        seller: ctx.accounts.seller.key(),
        amount: acdm_amount,
        price: acdm_price,
    });

    ctx.accounts.ido.orders += 1;

    Ok(())
}

#[event]
struct AddOrderEvent {
    id: u64,
    seller: Pubkey,
    amount: u64,
    price: u64,
}
