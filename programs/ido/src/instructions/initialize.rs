use crate::{account::*, config::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = ido_authority, seeds = [b"ido"], bump, space = 8 + Ido::LEN)]
    ido: Account<'info, Ido>,
    #[account(mut)]
    ido_authority: Signer<'info>,
    acdm_mint: Account<'info, Mint>,
    #[account(associated_token::authority = ido, associated_token::mint = acdm_mint)]
    ido_acdm: Account<'info, TokenAccount>,
    usdc_mint: Account<'info, Mint>,
    #[account(associated_token::authority = ido, associated_token::mint = usdc_mint)]
    ido_usdc: Account<'info, TokenAccount>,
    rent: Sysvar<'info, Rent>,
    system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, round_time: u32) -> Result<()> {
    let ts = Clock::get()?.unix_timestamp as u32;

    ctx.accounts.ido.bump = *ctx.bumps.get("ido").unwrap();
    ctx.accounts.ido.authority = ctx.accounts.ido_authority.key();
    ctx.accounts.ido.state = IdoState::NotStarted;
    ctx.accounts.ido.acdm_mint = ctx.accounts.acdm_mint.key();
    ctx.accounts.ido.usdc_mint = ctx.accounts.usdc_mint.key();
    ctx.accounts.ido.usdc_traded = INITIAL_ISSUE * INITIAL_PRICE;
    ctx.accounts.ido.round_time = round_time;
    ctx.accounts.ido.current_state_start_ts = ts;

    emit!(InitializeEvent {});

    Ok(())
}

#[event]
struct InitializeEvent {}
