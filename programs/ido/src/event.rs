use anchor_lang::prelude::*;

#[event]
pub struct OrderEvent {
    pub id: u64,
}
