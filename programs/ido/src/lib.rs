use crate::instructions::*;
use anchor_lang::prelude::*;

mod account;
mod config;
mod error;
mod helpers;
mod instructions;
mod referral;

declare_id!("AUuf3MCis1CgAsFXHgson2r3g4VjqUdD7r3CUc8mEKj3");

#[program]
pub mod ido {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, round_time: u32) -> Result<()> {
        instructions::initialize(ctx, round_time)
    }

    pub fn register_member(ctx: Context<RegisterMember>, referer: Option<Pubkey>) -> Result<()> {
        instructions::register_member(ctx, referer)
    }

    pub fn start_sale_round(ctx: Context<StartSaleRound>) -> Result<()> {
        instructions::start_sale_round(ctx)
    }

    pub fn buy_acdm<'info>(
        ctx: Context<'_, '_, '_, 'info, BuyAcdm<'info>>,
        acdm_amount: u64,
    ) -> Result<()> {
        instructions::buy_acdm(ctx, acdm_amount)
    }

    pub fn start_trade_round(ctx: Context<StartTradeRound>) -> Result<()> {
        instructions::start_trade_round(ctx)
    }

    pub fn add_order(ctx: Context<AddOrder>, acdm_amount: u64, acdm_price: u64) -> Result<()> {
        instructions::add_order(ctx, acdm_amount, acdm_price)
    }

    pub fn redeem_order<'info>(
        ctx: Context<'_, '_, '_, 'info, RedeemOrder<'info>>,
        id: u64,
        acdm_amount: u64,
    ) -> Result<()> {
        instructions::redeem_order(ctx, id, acdm_amount)
    }

    pub fn remove_order(ctx: Context<RemoveOrder>, id: u64) -> Result<()> {
        instructions::remove_order(ctx, id)
    }

    pub fn withdraw_ido_usdc(ctx: Context<WithdrawIdoUsdc>) -> Result<()> {
        instructions::withdraw_ido_usdc(ctx)
    }

    pub fn end_ido(ctx: Context<EndIdo>) -> Result<()> {
        instructions::end_ido(ctx)
    }
}
