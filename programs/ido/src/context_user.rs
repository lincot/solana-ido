use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct RegisterMember<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [b"member", authority.key().as_ref()],
        bump,
        space = 8 + Member::LEN,
    )]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyAcdm<'info> {
    #[account(seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(mut, seeds = [b"ido_acdm"], bump = ido.bump_acdm)]
    pub ido_acdm: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"ido_usdc"], bump = ido.bump_usdc)]
    pub ido_usdc: Account<'info, TokenAccount>,
    pub buyer: Signer<'info>,
    #[account(seeds = [b"member", buyer.key().as_ref()], bump = buyer_member.bump)]
    pub buyer_member: Account<'info, Member>,
    #[account(mut)]
    pub buyer_acdm: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_usdc: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
impl<'info> BuyAcdm<'info> {
    pub fn transfer_acdm(&self, amount: u64) -> Result<()> {
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

#[derive(Accounts)]
pub struct AddOrder<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(
        init,
        payer = seller,
        seeds = [b"order", ido.orders.to_le_bytes().as_ref()],
        bump,
        space = 8 + Order::LEN,
    )]
    pub order: Account<'info, Order>,
    #[account(mut, address = ido.acdm_mint)]
    pub acdm_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = seller,
        seeds = [b"order_acdm", ido.orders.to_le_bytes().as_ref()],
        bump,
        token::authority = order,
        token::mint = acdm_mint,
    )]
    pub order_acdm: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub seller_acdm: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
impl<'info> AddOrder<'info> {
    pub fn transfer_acdm(&self, amount: u64) -> Result<()> {
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

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct RedeemOrder<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Box<Account<'info, Ido>>,
    #[account(mut, seeds = [b"ido_usdc"], bump = ido.bump_usdc)]
    pub ido_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"order", id.to_le_bytes().as_ref()], bump = order.bump)]
    pub order: Account<'info, Order>,
    #[account(mut, seeds = [b"order_acdm", id.to_le_bytes().as_ref()], bump = order.bump_acdm)]
    pub order_acdm: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_acdm: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_usdc: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(address = order.authority)]
    pub seller: UncheckedAccount<'info>,
    #[account(seeds = [b"member", seller.key().as_ref()], bump = seller_member.bump)]
    pub seller_member: Account<'info, Member>,
    #[account(mut, token::authority = order.authority)]
    pub seller_usdc: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
impl<'info> RedeemOrder<'info> {
    pub fn transfer_usdc_to_seller(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.buyer_usdc.to_account_info(),
            to: self.seller_usdc.to_account_info(),
            authority: self.buyer.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)
    }

    pub fn transfer_acdm_to_buyer(&self, id: u64, amount: u64) -> Result<()> {
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

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct RemoveOrder<'info> {
    #[account(mut, seeds = [b"order", id.to_le_bytes().as_ref()], bump = order.bump, close = seller)]
    pub order: Account<'info, Order>,
    #[account(mut, seeds = [b"order_acdm", id.to_le_bytes().as_ref()], bump = order.bump_acdm)]
    pub order_acdm: Account<'info, TokenAccount>,
    #[account(mut, address = order.authority)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub seller_acdm: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
impl<'info> RemoveOrder<'info> {
    pub fn send_leftover_to_seller(&self, id: u64) -> Result<()> {
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

    pub fn close_order_acdm_account(&self, id: u64) -> Result<()> {
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
