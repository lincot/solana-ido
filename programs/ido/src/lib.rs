use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, CloseAccount, MintTo, Token, TokenAccount, Transfer};

use account::*;
use context::*;
use error::*;
use event::*;

pub mod account;
pub mod context;
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

    pub fn initialize(ctx: Context<Initialize>, round_time: i64) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let ido = &mut ctx.accounts.ido;

        ido.bump = *ctx.bumps.get("ido").unwrap();
        ido.bump_acdm = *ctx.bumps.get("ido_acdm").unwrap();
        ido.bump_usdc = *ctx.bumps.get("ido_usdc").unwrap();
        ido.authority = ctx.accounts.ido_authority.key();
        ido.state = IdoState::NotStarted;
        ido.acdm_mint = ctx.accounts.acdm_mint.key();
        ido.usdc_mint = ctx.accounts.usdc_mint.key();
        ido.usdc_traded = 1_000_000_000;
        if round_time < 0 {
            return err!(IdoError::RoundTimeInvalid);
        }
        ido.round_time = round_time;
        ido.current_state_start_ts = ts;

        Ok(())
    }

    pub fn register_member(ctx: Context<RegisterMember>, referer: Option<Pubkey>) -> Result<()> {
        ctx.accounts.member.bump = *ctx.bumps.get("member").unwrap();
        ctx.accounts.member.referer = referer;

        if let Some(referer) = referer {
            get_referer_member(ctx.remaining_accounts, referer)?;
        }

        Ok(())
    }

    pub fn start_sale_round(ctx: Context<StartSaleRound>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let ido = &mut ctx.accounts.ido;

        match ido.state {
            IdoState::NotStarted => {}
            IdoState::SaleRound => return err!(IdoError::RoundAlreadyStarted),
            IdoState::TradeRound => {
                if ts - ido.current_state_start_ts < ido.round_time {
                    return err!(IdoError::CannotEndTradeRound);
                }
            }
            IdoState::Over => return err!(IdoError::IdoIsOver),
        }

        ido.state = IdoState::SaleRound;
        ido.current_state_start_ts = ts;
        ido.acdm_price = if ido.sale_rounds_started == 0 {
            INITIAL_PRICE
        } else {
            sale_price_formula(ido.acdm_price)
        };
        ido.sale_rounds_started += 1;

        let amount_to_mint = ido.usdc_traded / ido.acdm_price;
        let cpi_accounts = MintTo {
            mint: ctx.accounts.acdm_mint.to_account_info(),
            to: ctx.accounts.ido_acdm.to_account_info(),
            authority: ctx.accounts.acdm_mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount_to_mint)
    }

    pub fn buy_acdm<'a, 'b, 'info>(
        ctx: Context<'a, 'b, 'b, 'info, BuyAcdm<'info>>,
        acdm_amount: u64,
    ) -> Result<()> {
        let ido = &mut ctx.accounts.ido;

        match ido.state {
            IdoState::NotStarted => return err!(IdoError::NotSaleRound),
            IdoState::SaleRound => {}
            IdoState::TradeRound => return err!(IdoError::NotSaleRound),
            IdoState::Over => return err!(IdoError::IdoIsOver),
        }

        if acdm_amount == 0 {
            return err!(IdoError::OverflowingArgument);
        }

        let usdc_amount_to_ido = acdm_amount
            .checked_mul(ido.acdm_price)
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

        let seeds = &[b"ido".as_ref(), &[ido.bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.ido_acdm.to_account_info(),
            to: ctx.accounts.buyer_acdm.to_account_info(),
            authority: ctx.accounts.ido.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, acdm_amount)
    }

    pub fn start_trade_round(ctx: Context<StartTradeRound>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let ido = &mut ctx.accounts.ido;

        let sold_all = match ido.state {
            IdoState::NotStarted => return err!(IdoError::NotSaleRound),
            IdoState::SaleRound => {
                let sold_all = ctx.accounts.ido_acdm.amount == 0;

                if !sold_all && (ts - ido.current_state_start_ts < ido.round_time) {
                    return err!(IdoError::CannotEndSaleRound);
                }

                sold_all
            }
            IdoState::TradeRound => return err!(IdoError::RoundAlreadyStarted),
            IdoState::Over => return err!(IdoError::IdoIsOver),
        };

        ido.state = IdoState::TradeRound;
        ido.current_state_start_ts = ts;
        ido.usdc_traded = 0;

        if !sold_all {
            let seeds = &[b"ido".as_ref(), &[ido.bump]];
            let signer = &[&seeds[..]];
            let cpi_accounts = Burn {
                mint: ctx.accounts.acdm_mint.to_account_info(),
                from: ctx.accounts.ido_acdm.to_account_info(),
                authority: ctx.accounts.ido.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::burn(cpi_ctx, ctx.accounts.ido_acdm.amount)?;
        }

        Ok(())
    }

    pub fn end_ido(ctx: Context<EndIdo>) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let ido = &mut ctx.accounts.ido;

        match ido.state {
            IdoState::NotStarted => return err!(IdoError::NotTradeRound),
            IdoState::SaleRound => return err!(IdoError::NotTradeRound),
            IdoState::TradeRound => {
                if ts - ido.current_state_start_ts < ido.round_time {
                    return err!(IdoError::CannotEndTradeRound);
                }
            }
            IdoState::Over => return err!(IdoError::IdoIsOver),
        }

        ido.state = IdoState::Over;
        ido.current_state_start_ts = ts;

        Ok(())
    }

    pub fn add_order(ctx: Context<AddOrder>, acdm_amount: u64, acdm_price: u64) -> Result<()> {
        let ido = &mut ctx.accounts.ido;

        match ido.state {
            IdoState::NotStarted => return err!(IdoError::NotTradeRound),
            IdoState::SaleRound => return err!(IdoError::NotTradeRound),
            IdoState::TradeRound => {}
            IdoState::Over => return err!(IdoError::IdoIsOver),
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.seller_acdm.to_account_info(),
            to: ctx.accounts.order_acdm.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, acdm_amount)?;

        let order = &mut ctx.accounts.order;
        order.bump = *ctx.bumps.get("order").unwrap();
        order.bump_acdm = *ctx.bumps.get("order_acdm").unwrap();
        order.authority = ctx.accounts.seller.key();
        order.price = acdm_price;

        ido.orders += 1;

        emit!(OrderEvent { id: ido.orders - 1 });

        Ok(())
    }

    pub fn redeem_order<'a, 'b, 'info>(
        ctx: Context<'a, 'b, 'b, 'info, RedeemOrder<'info>>,
        id: u64,
        acdm_amount: u64,
    ) -> Result<()> {
        let ido = &mut ctx.accounts.ido;

        match ido.state {
            IdoState::NotStarted => return err!(IdoError::NotTradeRound),
            IdoState::SaleRound => return err!(IdoError::NotTradeRound),
            IdoState::TradeRound => {}
            IdoState::Over => return err!(IdoError::IdoIsOver),
        }

        if acdm_amount == 0 {
            return err!(IdoError::OverflowingArgument);
        }

        let usdc_amount_total = acdm_amount
            .checked_mul(ctx.accounts.order.price)
            .ok_or(IdoError::OverflowingArgument)?;
        ido.usdc_traded = ido
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

        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_usdc.to_account_info(),
            to: ctx.accounts.seller_usdc.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, usdc_amount_so_seller)?;

        let seeds = &[
            b"order".as_ref(),
            &id.to_le_bytes(),
            &[ctx.accounts.order.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.order_acdm.to_account_info(),
            to: ctx.accounts.buyer_acdm.to_account_info(),
            authority: ctx.accounts.order.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, acdm_amount)
    }

    pub fn withdraw_ido_usdc(ctx: Context<WithdrawIdoUsdc>) -> Result<()> {
        let seeds = &[b"ido".as_ref(), &[ctx.accounts.ido.bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.ido_usdc.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.ido.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.ido_usdc.amount)
    }

    pub fn remove_order(ctx: Context<RemoveOrder>, id: u64) -> Result<()> {
        let seeds = &[
            b"order".as_ref(),
            &id.to_le_bytes(),
            &[ctx.accounts.order.bump],
        ];
        let signer = &[&seeds[..]];

        let leftover_amount = ctx.accounts.order_acdm.amount;

        if leftover_amount != 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.order_acdm.to_account_info(),
                to: ctx.accounts.seller_acdm.to_account_info(),
                authority: ctx.accounts.order.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, leftover_amount)?;
        }

        let cpi_accounts = CloseAccount {
            account: ctx.accounts.order_acdm.to_account_info(),
            destination: ctx.accounts.seller.to_account_info(),
            authority: ctx.accounts.order.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::close_account(cpi_ctx)
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

    if usdc_amount_to_ido != 0 {
        let cpi_accounts = Transfer {
            from: buyer_usdc.to_account_info(),
            to: ido_usdc.to_account_info(),
            authority: buyer.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, usdc_amount_to_ido)?;
    }

    Ok(())
}
