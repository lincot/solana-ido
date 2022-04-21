use anchor_lang::prelude::*;

#[event]
pub struct InitializeEvent {
    pub ts: u32,
}

#[event]
pub struct RegisterMemberEvent {
    pub ts: u32,
}

#[event]
pub struct StartSaleRoundEvent {
    pub ts: u32,
}

#[event]
pub struct BuyAcdmEvent {
    pub ts: u32,
}

#[event]
pub struct StartTradeRoundEvent {
    pub ts: u32,
}

#[event]
pub struct AddOrderEvent {
    pub ts: u32,
    pub id: u64,
    pub amount: u64,
    pub price: u64,
    pub seller: Pubkey,
}

#[event]
pub struct RedeemOrderEvent {
    pub ts: u32,
    pub id: u64,
    pub amount: u64,
    pub buyer: Pubkey,
}

#[event]
pub struct RemoveOrderEvent {
    pub ts: u32,
    pub id: u64,
}

#[event]
pub struct WithdrawIdoUsdcEvent {
    pub ts: u32,
}

#[event]
pub struct EndIdoEvent {
    pub ts: u32,
}
