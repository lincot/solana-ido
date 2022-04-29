use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use account::*;
use context_admin::*;
use context_user::*;
use error::*;
use event::*;

pub mod account;
pub mod context_admin;
pub mod context_user;
pub mod error;
pub mod event;

declare_id!("Hxcws9iykaMYStaLJhHiz3RtxqrpgfjMxaarRoGVan5q");

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
        ctx.accounts.ido.bump_acdm = *ctx.bumps.get("ido_acdm").unwrap();
        ctx.accounts.ido.bump_usdc = *ctx.bumps.get("ido_usdc").unwrap();
        ctx.accounts.ido.authority = ctx.accounts.ido_authority.key();
        ctx.accounts.ido.state = IdoState::NotStarted;
        ctx.accounts.ido.acdm_mint = ctx.accounts.acdm_mint.key();
        ctx.accounts.ido.usdc_mint = ctx.accounts.usdc_mint.key();
        ctx.accounts.ido.usdc_traded = 1_000_000_000;
        ctx.accounts.ido.round_time = round_time;
        ctx.accounts.ido.current_state_start_ts = ts;

        emit!(InitializeEvent { ts });

        Ok(())
    }

    pub fn register_member(ctx: Context<RegisterMember>, referer: Option<Pubkey>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        ctx.accounts.member.bump = *ctx.bumps.get("member").unwrap();
        ctx.accounts.member.referer = referer;

        if let Some(referer) = referer {
            get_referer_member(ctx.remaining_accounts, referer)?;
        }

        emit!(RegisterMemberEvent { ts });

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

        emit!(StartSaleRoundEvent { ts });

        Ok(())
    }

    pub fn buy_acdm<'a, 'b, 'info>(
        ctx: Context<'a, 'b, 'b, 'info, BuyAcdm<'info>>,
        acdm_amount: u64,
    ) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        is_sale_round(&ctx.accounts.ido)?;

        let usdc_amount_to_ido = acdm_amount
            .checked_mul(ctx.accounts.ido.acdm_price)
            .ok_or(IdoError::OverflowingArgument)?; // 100%
        let usdc_amount_to_referer = usdc_amount_to_ido / 20; // 5%
        let usdc_amount_to_referer2 = usdc_amount_to_ido
            .checked_mul(3)
            .ok_or(IdoError::OverflowingArgument)?
            / 100; // 3%

        send_to_referers_and_ido(
            usdc_amount_to_ido,
            usdc_amount_to_referer,
            usdc_amount_to_referer2,
            ctx.accounts.buyer_member.referer,
            &ctx.accounts.buyer,
            &ctx.accounts.buyer_usdc,
            &ctx.accounts.ido_usdc,
            &ctx.accounts.token_program,
            ctx.remaining_accounts,
        )?;

        ctx.accounts.transfer_acdm(acdm_amount)?;

        emit!(BuyAcdmEvent { ts });

        Ok(())
    }

    pub fn start_trade_round(ctx: Context<StartTradeRound>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        can_start_trade_round(&ctx.accounts.ido, &ctx.accounts.ido_acdm, ts)?;

        ctx.accounts.ido.state = IdoState::TradeRound;
        ctx.accounts.ido.current_state_start_ts = ts;
        ctx.accounts.ido.usdc_traded = 0;

        ctx.accounts.burn_acdm()?;

        emit!(StartTradeRoundEvent { ts });

        Ok(())
    }

    pub fn add_order(ctx: Context<AddOrder>, acdm_amount: u64, acdm_price: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        is_trade_round(&ctx.accounts.ido)?;

        ctx.accounts.transfer_acdm(acdm_amount)?;

        ctx.accounts.order.bump = *ctx.bumps.get("order").unwrap();
        ctx.accounts.order.bump_acdm = *ctx.bumps.get("order_acdm").unwrap();
        ctx.accounts.order.authority = ctx.accounts.seller.key();
        ctx.accounts.order.price = acdm_price;

        emit!(AddOrderEvent {
            ts,
            id: ctx.accounts.ido.orders,
            amount: acdm_amount,
            price: acdm_price,
            seller: ctx.accounts.seller.key(),
        });

        ctx.accounts.ido.orders += 1;

        Ok(())
    }

    pub fn redeem_order<'a, 'b, 'info>(
        ctx: Context<'a, 'b, 'b, 'info, RedeemOrder<'info>>,
        id: u64,
        acdm_amount: u64,
    ) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        is_trade_round(&ctx.accounts.ido)?;

        let usdc_amount_total = acdm_amount
            .checked_mul(ctx.accounts.order.price)
            .ok_or(IdoError::OverflowingArgument)?;
        ctx.accounts.ido.usdc_traded = ctx
            .accounts
            .ido
            .usdc_traded
            .checked_add(usdc_amount_total)
            .ok_or(IdoError::OverflowingArgument)?;

        let usdc_amount_to_ido = usdc_amount_total / 20; // 5%
        let usdc_amount_to_referer = usdc_amount_to_ido / 2; // 2.5%
        let usdc_amount_to_referer2 = usdc_amount_to_ido - usdc_amount_to_referer; // 2.5%
        let usdc_amount_so_seller = usdc_amount_total - usdc_amount_to_ido; // 95%

        send_to_referers_and_ido(
            usdc_amount_to_ido,
            usdc_amount_to_referer,
            usdc_amount_to_referer2,
            ctx.accounts.seller_member.referer,
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
            ts,
            id,
            buyer: ctx.accounts.buyer.key(),
            amount: acdm_amount
        });

        Ok(())
    }

    pub fn remove_order(ctx: Context<RemoveOrder>, id: u64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        ctx.accounts.send_leftover_to_seller(id)?;
        ctx.accounts.close_order_acdm_account(id)?;

        emit!(RemoveOrderEvent { ts, id });

        Ok(())
    }

    pub fn withdraw_ido_usdc(ctx: Context<WithdrawIdoUsdc>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        ctx.accounts.transfer()?;

        emit!(WithdrawIdoUsdcEvent { ts });

        Ok(())
    }

    pub fn end_ido(ctx: Context<EndIdo>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp as u32;

        can_end_ido(&ctx.accounts.ido, ts)?;

        ctx.accounts.ido.state = IdoState::Over;
        ctx.accounts.ido.current_state_start_ts = ts;

        emit!(EndIdoEvent { ts });

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

fn get_referer_member<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    referer: Pubkey,
) -> Result<Account<'info, Member>> {
    if remaining_accounts.is_empty() {
        return err!(IdoError::RefererMemberAccountNotProvided);
    }

    let referer_member = Account::<Member>::try_from(&remaining_accounts[0])?;

    let pda_key =
        Pubkey::create_program_address(&[b"member", referer.as_ref(), &[referer_member.bump]], &ID)
            .map_err(|_| IdoError::RefererPda)?;
    if referer_member.key() != pda_key {
        return err!(IdoError::RefererPda);
    }

    Ok(referer_member)
}

#[allow(clippy::too_many_arguments)]
fn send_to_referers_and_ido<'info>(
    mut usdc_amount_to_ido: u64,
    usdc_amount_to_referer: u64,
    usdc_amount_to_referer2: u64,
    referer: Option<Pubkey>,
    buyer: &Signer<'info>,
    buyer_usdc: &Account<'info, TokenAccount>,
    ido_usdc: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<()> {
    if let Some(referer) = referer {
        let referer_member = get_referer_member(remaining_accounts, referer)?;

        if remaining_accounts.len() < 2 {
            return err!(IdoError::RefererTokenAccountNotProvided);
        }

        let user2_usdc = Account::<TokenAccount>::try_from(&remaining_accounts[1])?;
        if user2_usdc.owner != referer {
            return err!(IdoError::RefererOwner);
        }

        usdc_amount_to_ido -= usdc_amount_to_referer;

        msg!("sending fee to first referer");

        let cpi_accounts = Transfer {
            from: buyer_usdc.to_account_info(),
            to: remaining_accounts[1].clone(),
            authority: buyer.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, usdc_amount_to_referer)?;

        if let Some(referer2) = referer_member.referer {
            if remaining_accounts.len() < 3 {
                return err!(IdoError::RefererTokenAccountNotProvided);
            }

            let user3_usdc = Account::<TokenAccount>::try_from(&remaining_accounts[2])?;
            if user3_usdc.owner != referer2 {
                return err!(IdoError::RefererOwner);
            }

            usdc_amount_to_ido -= usdc_amount_to_referer2;

            msg!("sending fee to second referer");

            let cpi_accounts = Transfer {
                from: buyer_usdc.to_account_info(),
                to: remaining_accounts[2].clone(),
                authority: buyer.to_account_info(),
            };
            let cpi_program = token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, usdc_amount_to_referer2)?;
        }
    }

    if usdc_amount_to_ido == 0 {
        return Ok(());
    }
    let cpi_accounts = Transfer {
        from: buyer_usdc.to_account_info(),
        to: ido_usdc.to_account_info(),
        authority: buyer.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, usdc_amount_to_ido)
}
