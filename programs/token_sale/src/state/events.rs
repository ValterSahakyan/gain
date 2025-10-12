use anchor_lang::prelude::*;

#[event]
pub struct TokenPurchasedWithSol {
    pub buyer: Pubkey,
    pub token_amount: u64,
    pub sol_amount: u64,
    pub sol_per_token: u64,
    pub eur_usd_rate: u64,
    pub sol_usd_rate: u64,
    pub equivalent_eurc_price: u64,
    pub timestamp: i64,
    pub payment_method: String,
    pub tier: String,
    pub round: u8,
}

#[event]
pub struct TokenPurchasedWithEurc {
    pub buyer: Pubkey,
    pub token_amount: u64,
    pub eurc_amount: u64,
    pub eurc_per_token: u64,
    pub timestamp: i64,
    pub payment_method: String,
    pub tier: String,
    pub round: u8,
}

#[event]
pub struct TokenPurchasedWithUsdc {
    pub buyer: Pubkey,
    pub token_amount: u64,
    pub usdc_amount: u64,
    pub usdc_per_token: u64,
    pub eur_usd_rate: u64,
    pub equivalent_eurc_price: u64,
    pub timestamp: i64,
    pub payment_method: String,
    pub tier: String,
    pub round: u8,
}