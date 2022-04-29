use anchor_lang::prelude::*;

#[event]
pub struct InitializeEvent {}

#[event]
pub struct RegisterMemberEvent {
    pub authority: Pubkey,
}

#[event]
pub struct StartSaleRoundEvent {
    pub acdm_price: u64,
    pub minted_amount: u64,
}

#[event]
pub struct BuyAcdmEvent {
    pub buyer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct StartTradeRoundEvent {}

#[event]
pub struct AddOrderEvent {
    pub id: u64,
    pub seller: Pubkey,
    pub amount: u64,
    pub price: u64,
}

#[event]
pub struct RedeemOrderEvent {
    pub id: u64,
    pub buyer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct RemoveOrderEvent {
    pub id: u64,
}

#[event]
pub struct WithdrawIdoUsdcEvent {}

#[event]
pub struct EndIdoEvent {}
