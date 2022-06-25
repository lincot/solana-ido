use crate::{account::*, referral::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RegisterMember<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [b"member", authority.key().as_ref()],
        bump,
        space = 8 + Member::LEN,
    )]
    member: Account<'info, Member>,
    #[account(mut)]
    authority: Signer<'info>,
    system_program: Program<'info, System>,
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

#[event]
struct RegisterMemberEvent {
    authority: Pubkey,
}
