use anchor_lang::prelude::*;

// Round configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum Round {
    First = 1,  // 10% cheaper than third round
    Second = 2, // 5% cheaper than third round  
    Third = 3,  // Base prices
}

impl Round {
    pub fn get_discount_multiplier(&self) -> f64 {
        match self {
            Round::First => 0.90,  // 10% cheaper = 90% of base price
            Round::Second => 0.95, // 5% cheaper = 95% of base price
            Round::Third => 1.00,  // Base price = 100%
        }
    }
}

// Base prices for third round (in lamports - 6 decimals)
pub const BASE_PRICE_TIER_1: u64 = 290_000;    // 0.290
pub const BASE_PRICE_TIER_2: u64 = 280_000;    // 0.280
pub const BASE_PRICE_TIER_3: u64 = 270_000;    // 0.270
pub const BASE_PRICE_TIER_4: u64 = 240_000;    // 0.240
pub const BASE_PRICE_TIER_5: u64 = 180_000;    // 0.180

pub const TOKENS_PER_ROUND: u64 = 1_000_000; // 1 million tokens per round

// USDC and EURC mint addresses (devnet)
pub const USDC_MINT: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";
pub const EURC_MINT: &str = "HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr";

#[account]
pub struct Config {
    pub initialized: bool,
    pub owner: Pubkey,
    pub treasury: Pubkey,
    pub mint: Pubkey,
    pub paused: bool,
    pub bump: u8,
    pub sale_start_time: i64,
    pub round1_tokens_sold: u64,
    pub round2_tokens_sold: u64,
    pub round3_tokens_sold: u64,
}

impl Config {
    pub const SIZE: usize = 1 + 32 + 32 + 32 + 8 + 1 + 1 + 8 + 8 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RoundInfo {
    pub current_round: u8,
    pub round_start_time: i64,
    pub round_end_time: i64,
    pub sale_start_time: i64,
    pub total_duration: i64,
    pub round_tokens_sold: u64,
    pub remaining_tokens: u64,
    pub tokens_per_round: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AllRoundsInfo {
    pub round1: RoundDetails,
    pub round2: RoundDetails,
    pub round3: RoundDetails,
    pub total_tokens_sold: u64,
    pub total_tokens: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RoundDetails {
    pub tokens_sold: u64,
    pub remaining: u64,
    pub total: u64,
}