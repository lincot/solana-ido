use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use account::*;
use context_admin::*;
use context_user::*;
use error::*;
use event::*;
use referral::*;

pub mod account;
pub mod context_admin;
pub mod context_user;
pub mod error;
pub mod event;
pub mod referral;

declare_id!("Hxcws9iykaMYStaLJhHiz3RtxqrpgfjMxaarRoGVan5q");

const INITIAL_ISSUE: u64 = 10_000;
const INITIAL_PRICE: u64 = 100_000;

const fn sale_price_formula(prev_price: u64) -> u64 {
    prev_price * 103 / 100 + INITIAL_PRICE * 2 / 5
}

#[program]
pub mod ido {
    use super::*;

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

    pub fn register_member(ctx: Context<RegisterMember>, referer: Option<Pubkey>) -> Result<()> {
        ctx.accounts.member.bump = *ctx.bumps.get("member").unwrap();
        ctx.accounts.member.referer = referer;

        if let Some(referer) = referer {
            get_referer_member(ctx.remaining_accounts, referer)?;
        }

        emit!(RegisterMemberEvent {
            authority: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    pub fn start_sale_round(ctx: Context<StartSaleRound>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        can_start_sale_round(&ctx.accounts.ido, ts)?;

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

    pub fn start_trade_round(ctx: Context<StartTradeRound>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        can_start_trade_round(&ctx.accounts.ido, &ctx.accounts.ido_acdm, ts)?;

        ctx.accounts.ido.state = IdoState::TradeRound;
        ctx.accounts.ido.current_state_start_ts = ts;
        ctx.accounts.ido.usdc_traded = 0;

        ctx.accounts.burn_acdm()?;

        emit!(StartTradeRoundEvent {});

        Ok(())
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

    pub fn remove_order(ctx: Context<RemoveOrder>, id: u64) -> Result<()> {
        ctx.accounts.send_leftover_to_seller(id)?;
        ctx.accounts.close_order_acdm_account(id)?;

        emit!(RemoveOrderEvent { id });

        Ok(())
    }

    pub fn withdraw_ido_usdc(ctx: Context<WithdrawIdoUsdc>) -> Result<()> {
        ctx.accounts.transfer()?;

        emit!(WithdrawIdoUsdcEvent {});

        Ok(())
    }

    pub fn end_ido(ctx: Context<EndIdo>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        can_end_ido(&ctx.accounts.ido, ts)?;

        ctx.accounts.ido.state = IdoState::Over;
        ctx.accounts.ido.current_state_start_ts = ts;

        emit!(EndIdoEvent {});

        Ok(())
    }
}

fn round_time_over(ido: &Ido, ts: u32) -> Result<()> {
    if ts - ido.current_state_start_ts < ido.round_time {
        err!(IdoError::CannotEndRound)
    } else {
        Ok(())
    }
}

fn can_start_sale_round(ido: &Ido, ts: u32) -> Result<()> {
    match ido.state {
        IdoState::NotStarted => Ok(()),
        IdoState::SaleRound => err!(IdoError::RoundAlreadyStarted),
        IdoState::TradeRound => round_time_over(ido, ts),
        IdoState::Over => err!(IdoError::IdoIsOver),
    }
}

fn can_start_trade_round(ido: &Ido, ido_acdm: &TokenAccount, ts: u32) -> Result<()> {
    match ido.state {
        IdoState::NotStarted => err!(IdoError::NotSaleRound),
        IdoState::SaleRound => {
            if ido_acdm.amount == 0 {
                return Ok(());
            }

            round_time_over(ido, ts)
        }
        IdoState::TradeRound => err!(IdoError::RoundAlreadyStarted),
        IdoState::Over => err!(IdoError::IdoIsOver),
    }
}

fn can_end_ido(ido: &Ido, ts: u32) -> Result<()> {
    match ido.state {
        IdoState::NotStarted => err!(IdoError::NotTradeRound),
        IdoState::SaleRound => err!(IdoError::NotTradeRound),
        IdoState::TradeRound => round_time_over(ido, ts),
        IdoState::Over => err!(IdoError::IdoIsOver),
    }
}

fn is_sale_round(ido: &Ido) -> Result<()> {
    match ido.state {
        IdoState::NotStarted => err!(IdoError::NotSaleRound),
        IdoState::SaleRound => Ok(()),
        IdoState::TradeRound => err!(IdoError::NotSaleRound),
        IdoState::Over => err!(IdoError::IdoIsOver),
    }
}

fn is_trade_round(ido: &Ido) -> Result<()> {
    match ido.state {
        IdoState::NotStarted => err!(IdoError::NotTradeRound),
        IdoState::SaleRound => err!(IdoError::NotTradeRound),
        IdoState::TradeRound => Ok(()),
        IdoState::Over => err!(IdoError::IdoIsOver),
    }
}
