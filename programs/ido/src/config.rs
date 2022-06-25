pub const INITIAL_ISSUE: u64 = 10_000;
pub const INITIAL_PRICE: u64 = 100_000;

pub const fn sale_price_formula(prev_price: u64) -> u64 {
    prev_price * 103 / 100 + INITIAL_PRICE * 2 / 5
}
