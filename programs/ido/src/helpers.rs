use crate::{account::*, error::*};
use anchor_lang::prelude::*;

pub(crate) fn round_time_over(ido: &Ido, ts: u32) -> Result<()> {
    if ts - ido.current_state_start_ts < ido.round_time {
        err!(IdoError::CannotEndRound)
    } else {
        Ok(())
    }
}

pub(crate) fn is_sale_round(ido: &Ido) -> Result<()> {
    match ido.state {
        IdoState::NotStarted => err!(IdoError::NotSaleRound),
        IdoState::SaleRound => Ok(()),
        IdoState::TradeRound => err!(IdoError::NotSaleRound),
        IdoState::Over => err!(IdoError::IdoIsOver),
    }
}

pub(crate) fn is_trade_round(ido: &Ido) -> Result<()> {
    match ido.state {
        IdoState::NotStarted => err!(IdoError::NotTradeRound),
        IdoState::SaleRound => err!(IdoError::NotTradeRound),
        IdoState::TradeRound => Ok(()),
        IdoState::Over => err!(IdoError::IdoIsOver),
    }
}
