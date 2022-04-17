use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = ido_authority, seeds = [b"ido"], bump, space = 8 + Ido::LEN)]
    pub ido: Account<'info, Ido>,
    #[account(mut)]
    pub ido_authority: Signer<'info>,
    pub acdm_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = ido_authority,
        seeds = [b"ido_acdm"],
        bump,
        token::authority = ido,
        token::mint = acdm_mint,
    )]
    pub ido_acdm: Account<'info, TokenAccount>,
    pub usdc_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = ido_authority,
        seeds = [b"ido_usdc"],
        bump,
        token::authority = ido,
        token::mint = usdc_mint,
    )]
    pub ido_usdc: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

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
pub struct StartSaleRound<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(address = ido.authority)]
    pub ido_authority: Signer<'info>,
    pub acdm_mint_authority: Signer<'info>,
    #[account(mut, address = ido.acdm_mint, mint::authority = acdm_mint_authority)]
    pub acdm_mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"ido_acdm"], bump = ido.bump_acdm, token::authority = ido, token::mint = acdm_mint)]
    pub ido_acdm: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
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

#[derive(Accounts)]
pub struct StartTradeRound<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(mut, address = ido.authority)]
    pub ido_authority: Signer<'info>,
    #[account(mut, address = ido.acdm_mint)]
    pub acdm_mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"ido_acdm"], bump = ido.bump_acdm)]
    pub ido_acdm: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AddOrder<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(
        init,
        payer = user,
        seeds = [b"order", ido.orders.to_le_bytes().as_ref()],
        bump,
        space = 8 + Order::LEN,
    )]
    pub order: Account<'info, Order>,
    #[account(mut, address = ido.acdm_mint)]
    pub acdm_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = user,
        seeds = [b"order_acdm", ido.orders.to_le_bytes().as_ref()],
        bump,
        token::authority = order,
        token::mint = acdm_mint,
    )]
    pub order_acdm: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_acdm: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct RedeemOrder<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Box<Account<'info, Ido>>,
    #[account(address = ido.usdc_mint)]
    pub usdc_mint: Box<Account<'info, Mint>>,
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
    #[account(mut, associated_token::authority = order.authority, associated_token::mint = usdc_mint)]
    pub seller_usdc: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct RemoveOrder<'info> {
    #[account(mut, seeds = [b"order", id.to_le_bytes().as_ref()], bump = order.bump, close = user)]
    pub order: Account<'info, Order>,
    #[account(mut, seeds = [b"order_acdm", id.to_le_bytes().as_ref()], bump = order.bump_acdm)]
    pub order_acdm: Account<'info, TokenAccount>,
    #[account(mut, address = order.authority)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_acdm: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawIdoUsdc<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(mut, address = ido.authority)]
    pub ido_authority: Signer<'info>,
    #[account(mut, seeds = [b"ido_usdc"], bump = ido.bump_usdc)]
    pub ido_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EndIdo<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(address = ido.authority)]
    pub ido_authority: Signer<'info>,
}
