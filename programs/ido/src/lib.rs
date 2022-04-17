use anchor_lang::prelude::*;

use account::*;
use context::*;
use error::*;

pub mod account;
pub mod context;
pub mod error;

declare_id!("Hxcws9iykaMYStaLJhHiz3RtxqrpgfjMxaarRoGVan5q");

#[program]
pub mod ido {
    use super::*;
    use anchor_spl::token::{self, Burn, CloseAccount, MintTo, TokenAccount, Transfer};

    pub fn initialize(
        ctx: Context<Initialize>,
        acdm_mint: Pubkey,
        usdc_mint: Pubkey,
        round_time: i64,
    ) -> Result<()> {
        let ts = Clock::get()?.unix_timestamp;

        let ido = &mut ctx.accounts.ido;

        ido.bump = *ctx.bumps.get("ido").unwrap();
        ido.authority = ctx.accounts.ido_authority.key();
        ido.state = IdoState::NotStarted;
        ido.acdm_mint = acdm_mint;
        ido.usdc_mint = usdc_mint;
        ido.usdc_traded = 1_000_000_000;
        if round_time < 0 {
            return err!(IdoError::RoundTimeInvalid);
        }
        ido.round_time = round_time;
        ido.current_state_start_ts = ts;

        Ok(())
    }

    pub fn set_referer(ctx: Context<SetReferer>, referer: Pubkey) -> Result<()> {
        ctx.accounts.user_referer.referer = referer;

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

        ido.acdm_price = sale_price_formula();

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

        let mut usdc_amount_to_ido = acdm_amount * ido.acdm_price;

        if ![0, 2, 4].contains(&ctx.remaining_accounts.len()) {
            return err!(IdoError::RefererAccountsAmount);
        }

        if ctx.remaining_accounts.len() >= 2 {
            let (referer_key, _) =
                Pubkey::find_program_address(&[b"referer", ctx.accounts.user.key().as_ref()], &ID);
            if ctx.remaining_accounts[0].key() != referer_key {
                return err!(IdoError::RefererPda);
            }

            let user_referer =
                Referer::try_deserialize(&mut &ctx.remaining_accounts[0].try_borrow_data()?[..])?;
            let user2_usdc = TokenAccount::try_deserialize(
                &mut &ctx.remaining_accounts[1].try_borrow_data()?[..],
            )?;
            if user2_usdc.owner != user_referer.referer {
                return err!(IdoError::RefererOwner);
            }

            msg!("sending fee to first referer");

            let usdc_amount_to_referer = usdc_amount_to_ido / 40; // 2.5%

            let cpi_accounts = Transfer {
                from: ctx.accounts.user_usdc.to_account_info(),
                to: ctx.remaining_accounts[1].clone(),
                authority: ctx.accounts.user.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, usdc_amount_to_referer)?;

            usdc_amount_to_ido -= usdc_amount_to_referer;

            if ctx.remaining_accounts.len() >= 4 {
                let (referer_key, _) =
                    Pubkey::find_program_address(&[b"referer", user_referer.referer.as_ref()], &ID);
                if ctx.remaining_accounts[2].key() != referer_key {
                    return err!(IdoError::RefererPda);
                }

                let user2_referer = Referer::try_deserialize(
                    &mut &ctx.remaining_accounts[2].try_borrow_data()?[..],
                )?;
                let user3_usdc = TokenAccount::try_deserialize(
                    &mut &ctx.remaining_accounts[3].try_borrow_data()?[..],
                )?;
                if user3_usdc.owner != user2_referer.referer {
                    return err!(IdoError::RefererOwner);
                }

                msg!("sending fee to second referer");

                let cpi_accounts = Transfer {
                    from: ctx.accounts.user_usdc.to_account_info(),
                    to: ctx.remaining_accounts[3].clone(),
                    authority: ctx.accounts.user.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                token::transfer(cpi_ctx, usdc_amount_to_referer)?;

                usdc_amount_to_ido -= usdc_amount_to_referer;
            }
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.ido_usdc.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, usdc_amount_to_ido)?;

        let seeds = &[b"ido".as_ref(), &[ido.bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.ido_acdm.to_account_info(),
            to: ctx.accounts.user_acdm.to_account_info(),
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

    pub fn add_order(ctx: Context<AddOrder>, acdm_amount: u64, acdm_price: u64) -> Result<u64> {
        let ido = &mut ctx.accounts.ido;

        match ido.state {
            IdoState::NotStarted => return err!(IdoError::NotTradeRound),
            IdoState::SaleRound => return err!(IdoError::NotTradeRound),
            IdoState::TradeRound => {}
            IdoState::Over => return err!(IdoError::IdoIsOver),
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_acdm.to_account_info(),
            to: ctx.accounts.order_acdm.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, acdm_amount)?;

        let order = &mut ctx.accounts.order;
        order.bump = *ctx.bumps.get("order").unwrap();
        order.authority = ctx.accounts.user.key();
        order.price = acdm_price;

        ido.orders += 1;

        Ok(ido.orders - 1)
    }

    pub fn redeem_order(ctx: Context<RedeemOrder>, id: u64, acdm_amount: u64) -> Result<()> {
        let ido = &mut ctx.accounts.ido;

        match ido.state {
            IdoState::NotStarted => return err!(IdoError::NotTradeRound),
            IdoState::SaleRound => return err!(IdoError::NotTradeRound),
            IdoState::TradeRound => {}
            IdoState::Over => return err!(IdoError::IdoIsOver),
        }

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
        token::transfer(cpi_ctx, acdm_amount)?;

        let usdc_amount = acdm_amount * ctx.accounts.order.price;
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_usdc.to_account_info(),
            to: ctx.accounts.seller_usdc.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, usdc_amount)?;

        ido.usdc_traded += usdc_amount;

        Ok(())
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
                to: ctx.accounts.user_acdm.to_account_info(),
                authority: ctx.accounts.order.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, leftover_amount)?;
        }

        let cpi_accounts = CloseAccount {
            account: ctx.accounts.order_acdm.to_account_info(),
            destination: ctx.accounts.user.to_account_info(),
            authority: ctx.accounts.order.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::close_account(cpi_ctx)
    }
}

const fn sale_price_formula() -> u64 {
    100_000
}
