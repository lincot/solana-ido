use crate::account::*;
use crate::error::*;
use crate::ID;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

pub fn get_referer_member<'info>(
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
pub fn send_to_referers_and_ido<'info>(
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
