pub mod instructions;
pub mod state;
pub mod error;

use anchor_lang::prelude::*;
use instructions::*;
use state::config::{RoundInfo, AllRoundsInfo}; // Add this import

declare_id!("Bv8sK4AN9bTrcLfH8zvCs8yiScbQZtT8nnss5uEticDe");

#[program]
pub mod simple_token_sale {
    use super::*;

    // Admin functions
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::admin::initialize(ctx)
    }

    pub fn set_paused(ctx: Context<OnlyOwner>, paused: bool) -> Result<()> {
        instructions::admin::set_paused(ctx, paused)
    }

    pub fn update_mint(ctx: Context<OnlyOwner>, new_mint: Pubkey) -> Result<()> {
        instructions::admin::update_mint(ctx, new_mint)
    }

    // Sale functions
    pub fn buy(ctx: Context<Buy>, amount: u64) -> Result<()> {
        instructions::sale::buy(ctx, amount)
    }

    pub fn buy_with_eurc(ctx: Context<BuyWithEurc>, amount: u64) -> Result<()> {
        instructions::sale::buy_with_eurc(ctx, amount)
    }

    pub fn buy_with_usdc(ctx: Context<BuyWithUsdc>, amount: u64) -> Result<()> {
        instructions::sale::buy_with_usdc(ctx, amount)
    }

    // Airdrop functions
    pub fn set_merkle_root(
        ctx: Context<SetMerkleRoot>, 
        root: [u8; 32],
        airdrop_amount: u64,
        max_claims: u64,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
        instructions::airdrop::set_merkle_root_handler(ctx, root, airdrop_amount, max_claims, start_time, end_time)
    }

    pub fn claim(ctx: Context<Claim>, amount: u64, proof: Vec<[u8; 32]>) -> Result<()> {
        instructions::airdrop::claim_handler(ctx, amount, proof)
    }

    // Info functions
    pub fn get_round_info(ctx: Context<GetRoundInfo>) -> Result<RoundInfo> {
        instructions::sale::get_round_info(ctx)
    }

    pub fn get_all_rounds_info(ctx: Context<GetRoundInfo>) -> Result<AllRoundsInfo> {
        instructions::sale::get_all_rounds_info(ctx)
    }
    pub fn update_sale_start_time(ctx: Context<OnlyOwner>) -> Result<()> {
        instructions::admin::update_sale_start_time(ctx)
    }
}