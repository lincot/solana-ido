use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Token, TokenAccount, Transfer};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct RemoveOrder<'info> {
    #[account(mut, seeds = [b"order", id.to_le_bytes().as_ref()], bump = order.bump, close = seller)]
    order: Account<'info, Order>,
    #[account(mut, associated_token::authority = order, associated_token::mint = seller_acdm.mint)]
    order_acdm: Account<'info, TokenAccount>,
    #[account(mut, address = order.authority)]
    seller: Signer<'info>,
    #[account(mut)]
    seller_acdm: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}
impl<'info> RemoveOrder<'info> {
    fn send_leftover_to_seller(&self, id: u64) -> Result<()> {
        let amount = self.order_acdm.amount;

        if amount == 0 {
            return Ok(());
        }

        let signer: &[&[&[u8]]] = &[&[b"order".as_ref(), &id.to_le_bytes(), &[self.order.bump]]];
        let cpi_accounts = Transfer {
            from: self.order_acdm.to_account_info(),
            to: self.seller_acdm.to_account_info(),
            authority: self.order.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)
    }

    fn close_order_acdm_account(&self, id: u64) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[b"order".as_ref(), &id.to_le_bytes(), &[self.order.bump]]];
        let cpi_accounts = CloseAccount {
            account: self.order_acdm.to_account_info(),
            destination: self.seller.to_account_info(),
            authority: self.order.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::close_account(cpi_ctx)
    }
}

pub fn remove_order(ctx: Context<RemoveOrder>, id: u64) -> Result<()> {
    ctx.accounts.send_leftover_to_seller(id)?;
    ctx.accounts.close_order_acdm_account(id)?;

    emit!(RemoveOrderEvent { id });

    Ok(())
}

#[event]
struct RemoveOrderEvent {
    id: u64,
}
