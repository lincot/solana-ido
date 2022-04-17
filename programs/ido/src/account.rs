use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq)]
pub enum IdoState {
    NotStarted,
    SaleRound,
    TradeRound,
    Over,
}

#[account]
pub struct Ido {
    pub bump: u8,
    pub bump_acdm: u8,
    pub bump_usdc: u8,
    pub authority: Pubkey,
    pub state: IdoState,
    pub acdm_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub acdm_price: u64,
    pub usdc_traded: u64,
    pub orders: u64,
    pub round_time: i64,
    pub current_state_start_ts: i64,
    pub sale_rounds_started: u8,
}
impl Ido {
    pub const LEN: usize = 1 + 1 + 1 + 32 + 1 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1;
}

#[account]
pub struct Order {
    pub bump: u8,
    pub bump_acdm: u8,
    pub authority: Pubkey,
    pub price: u64,
}
impl Order {
    pub const LEN: usize = 1 + 1 + 32 + 8;
}

#[account]
pub struct Referer {
    pub bump: u8,
    pub referer: Pubkey,
}
impl Referer {
    pub const LEN: usize = 1 + 32;
}
