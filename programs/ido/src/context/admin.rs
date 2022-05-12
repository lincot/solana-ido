use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = ido_authority, seeds = [b"ido"], bump, space = 8 + Ido::LEN)]
    pub ido: Account<'info, Ido>,
    #[account(mut)]
    pub ido_authority: Signer<'info>,
    pub acdm_mint: Account<'info, Mint>,
    #[account(associated_token::authority = ido, associated_token::mint = acdm_mint)]
    pub ido_acdm: Account<'info, TokenAccount>,
    pub usdc_mint: Account<'info, Mint>,
    #[account(associated_token::authority = ido, associated_token::mint = usdc_mint)]
    pub ido_usdc: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
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
    #[account(mut, associated_token::authority = ido, associated_token::mint = acdm_mint)]
    pub ido_acdm: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
impl<'info> StartSaleRound<'info> {
    pub fn mint_acdm(&self, amount: u64) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: self.acdm_mint.to_account_info(),
            to: self.ido_acdm.to_account_info(),
            authority: self.acdm_mint_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
pub struct StartTradeRound<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(mut, address = ido.authority)]
    pub ido_authority: Signer<'info>,
    #[account(mut, address = ido.acdm_mint)]
    pub acdm_mint: Account<'info, Mint>,
    #[account(mut, associated_token::authority = ido, associated_token::mint = acdm_mint)]
    pub ido_acdm: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
impl<'info> StartTradeRound<'info> {
    pub fn burn_acdm(&self) -> Result<()> {
        if self.ido_acdm.amount == 0 {
            return Ok(());
        }

        let signer: &[&[&[u8]]] = &[&[b"ido".as_ref(), &[self.ido.bump]]];
        let cpi_accounts = Burn {
            mint: self.acdm_mint.to_account_info(),
            from: self.ido_acdm.to_account_info(),
            authority: self.ido.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::burn(cpi_ctx, self.ido_acdm.amount)
    }
}

#[derive(Accounts)]
pub struct WithdrawIdoUsdc<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(mut, address = ido.authority)]
    pub ido_authority: Signer<'info>,
    #[account(mut, associated_token::authority = ido, associated_token::mint = ido.usdc_mint)]
    pub ido_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
impl<'info> WithdrawIdoUsdc<'info> {
    pub fn transfer(&self) -> Result<()> {
        let signer: &[&[&[u8]]] = &[&[b"ido".as_ref(), &[self.ido.bump]]];
        let cpi_accounts = Transfer {
            from: self.ido_usdc.to_account_info(),
            to: self.to.to_account_info(),
            authority: self.ido.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, self.ido_usdc.amount)
    }
}

#[derive(Accounts)]
pub struct EndIdo<'info> {
    #[account(mut, seeds = [b"ido"], bump = ido.bump)]
    pub ido: Account<'info, Ido>,
    #[account(address = ido.authority)]
    pub ido_authority: Signer<'info>,
}
