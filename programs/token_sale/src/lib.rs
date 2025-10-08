pub mod instructions;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::{self as token22, MintTo, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};
use anchor_spl::token::{self, Transfer, Token}; // For EURC transfers

declare_id!("6ZK4hFGen61b83NHsNTAMq71r3QJCTwknvj4CYfLxdBj");

#[event]
pub struct TokenPurchasedWithSol {
    pub buyer: Pubkey,
    pub token_amount: u64,
    pub sol_amount: u64,
    pub treasury: Pubkey,
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
    pub treasury: Pubkey,
    pub timestamp: i64,
    pub payment_method: String,
    pub tier: String,
    pub round: u8,
}

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

// Base prices for third round (in EURC lamports - 6 decimals)
const BASE_PRICE_TIER_1: u64 = 290_000;    // 0.290 EURC
const BASE_PRICE_TIER_2: u64 = 280_000;    // 0.280 EURC
const BASE_PRICE_TIER_3: u64 = 270_000;    // 0.270 EURC
const BASE_PRICE_TIER_4: u64 = 240_000;    // 0.240 EURC
const BASE_PRICE_TIER_5: u64 = 180_000;    // 0.180 EURC

const TOKENS_PER_ROUND: u64 = 1_000_000; // 1 million tokens per round

// Tiered pricing calculation with round discounts
pub fn get_eurc_price_per_token(amount: u64, round: Round) -> u64 {
    let base_price = match amount {
        a if a < 1000 => BASE_PRICE_TIER_1,
        a if a < 5000 => BASE_PRICE_TIER_2,
        a if a < 10000 => BASE_PRICE_TIER_3,
        a if a < 25000 => BASE_PRICE_TIER_4,
        _ => BASE_PRICE_TIER_5,
    };
    
    // Apply round discount
    let discount_multiplier = round.get_discount_multiplier();
    let discounted_price = (base_price as f64 * discount_multiplier) as u64;
    
    discounted_price
}

// Get tier name for events
pub fn get_tier_name(amount: u64) -> String {
    match amount {
        a if a < 1000 => "Tier 1 (<1K)".to_string(),
        a if a < 5000 => "Tier 2 (1K-5K)".to_string(),
        a if a < 10000 => "Tier 3 (5K-10K)".to_string(),
        a if a < 25000 => "Tier 4 (10K-25K)".to_string(),
        _ => "Tier 5 (25K+)".to_string(),
    }
}

// Get current round based on sale start time and round duration
pub fn get_current_round(config: &Config) -> Result<Round> {
    let current_time = Clock::get()?.unix_timestamp;
    let round_duration: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
    
    let elapsed_time = current_time - config.sale_start_time;
    
    if elapsed_time < round_duration {
        Ok(Round::First)
    } else if elapsed_time < round_duration * 2 {
        Ok(Round::Second)
    } else if elapsed_time < round_duration * 3 {
        Ok(Round::Third)
    } else {
        // Sale ended after 90 days
        err!(ErrorCode::SaleEnded)
    }
}

// Check if purchase exceeds round limit
pub fn check_round_limit(config: &Config, amount: u64) -> Result<()> {
    let current_round = get_current_round(config)?;
    let round_tokens_sold = get_round_tokens_sold(config, current_round);
    
    let remaining_tokens = TOKENS_PER_ROUND.checked_sub(round_tokens_sold)
        .ok_or(ErrorCode::Overflow)?;
    
    require!(
        amount <= remaining_tokens,
        ErrorCode::RoundLimitExceeded
    );
    
    Ok(())
}

// Get tokens sold in a specific round
pub fn get_round_tokens_sold(config: &Config, round: Round) -> u64 {
    match round {
        Round::First => config.round1_tokens_sold,
        Round::Second => config.round2_tokens_sold,
        Round::Third => config.round3_tokens_sold,
    }
}

// Update tokens sold for a round
pub fn update_round_tokens_sold(config: &mut Config, round: Round, amount: u64) -> Result<()> {
    match round {
        Round::First => {
            config.round1_tokens_sold = config.round1_tokens_sold
                .checked_add(amount)
                .ok_or(ErrorCode::Overflow)?;
        }
        Round::Second => {
            config.round2_tokens_sold = config.round2_tokens_sold
                .checked_add(amount)
                .ok_or(ErrorCode::Overflow)?;
        }
        Round::Third => {
            config.round3_tokens_sold = config.round3_tokens_sold
                .checked_add(amount)
                .ok_or(ErrorCode::Overflow)?;
        }
    }
    Ok(())
}

#[program]
pub mod simple_token_sale {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.owner = ctx.accounts.payer.key();
        config.treasury = ctx.accounts.treasury.key();
        config.mint = ctx.accounts.mint.key();
        config.price_lamports_per_token = 100000;
        config.paused = false;
        config.initialized = true;
        config.bump = ctx.bumps.config;
        config.sale_start_time = Clock::get()?.unix_timestamp;
        
        // Initialize round counters
        config.round1_tokens_sold = 0;
        config.round2_tokens_sold = 0;
        config.round3_tokens_sold = 0;

        msg!("✅ Token sale initialized with 3 rounds (90 days total)");
        msg!("🎯 Round 1: 30 days - 10% discount - 1M tokens");
        msg!("🎯 Round 2: 30 days - 5% discount - 1M tokens"); 
        msg!("🎯 Round 3: 30 days - Base price - 1M tokens");
        msg!("💰 Total sale: 3M tokens");
        Ok(())
    }

    pub fn set_paused(ctx: Context<OnlyOwner>, paused: bool) -> Result<()> {
        ctx.accounts.config.paused = paused;
        Ok(())
    }

    pub fn set_price(ctx: Context<OnlyOwner>, new_price: u64) -> Result<()> {
        ctx.accounts.config.price_lamports_per_token = new_price;
        Ok(())
    }

    pub fn update_mint(ctx: Context<OnlyOwner>, new_mint: Pubkey) -> Result<()> {
        ctx.accounts.config.mint = new_mint;
        msg!("Mint updated to: {}", new_mint);
        Ok(())
    }

    pub fn buy(ctx: Context<Buy>, amount: u64) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(config.initialized, ErrorCode::NotInitialized);
        require!(!config.paused, ErrorCode::SalePaused);
        require!(amount > 0, ErrorCode::InvalidAmount);
    
        let current_round = get_current_round(config)?;
        
        // Check round limit
        check_round_limit(config, amount)?;
        
        let total_price = amount
            .checked_mul(config.price_lamports_per_token)
            .ok_or(ErrorCode::Overflow)?;
    
        // Transfer SOL
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.treasury.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, total_price)?;
        
        // Mint tokens with PDA signing
        let seeds = b"mint";
        let bump = ctx.bumps.mint_authority_pda;
        let signer_seeds: &[&[&[u8]]] = &[&[seeds, &[bump]]];
    
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.buyer_ata.to_account_info(),
            authority: ctx.accounts.mint_authority_pda.to_account_info(),
        };
    
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_2022_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        
        let decimals = ctx.accounts.mint.decimals;
        let base_units = amount
            .checked_mul(10u64.pow(decimals as u32))
            .ok_or(ErrorCode::Overflow)?;
            
        token22::mint_to(cpi_ctx, base_units)?;
        
        // Update round tokens sold
        update_round_tokens_sold(config, current_round, amount)?;
        
        emit!(TokenPurchasedWithSol {
            buyer: ctx.accounts.buyer.key(),
            token_amount: amount,
            sol_amount: total_price,
            treasury: config.treasury,
            timestamp: Clock::get()?.unix_timestamp,
            payment_method: "SOL".to_string(),
            tier: get_tier_name(amount),
            round: current_round as u8,
        });
        
        let remaining = TOKENS_PER_ROUND - get_round_tokens_sold(config, current_round);
        msg!("✅ Purchase successful! Round: {:?}, Remaining: {} tokens", current_round, remaining);
        Ok(())
    }

    // ✅ AIRDROP FUNCTIONS - call the airdrop module functions directly
    pub fn set_merkle_root(ctx: Context<SetMerkleRoot>, root: [u8; 32]) -> Result<()> {
        instructions::airdrop::set_merkle_root_handler(ctx, root)
    }

    pub fn claim(ctx: Context<Claim>, amount: u64, proof: Vec<[u8; 32]>) -> Result<()> {
        instructions::airdrop::claim_handler(ctx, amount, proof)
    }

    // ✅ EURC FUNCTION
    pub fn buy_with_eurc(
        ctx: Context<BuyWithEurc>,
        amount: u64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(config.initialized, ErrorCode::NotInitialized);
        require!(!config.paused, ErrorCode::SalePaused);
        require!(amount > 0, ErrorCode::InvalidAmount);

        let current_round = get_current_round(config)?;

        // Check round limit
        check_round_limit(config, amount)?;

        // Calculate EURC price based on tiers and current round
        let eurc_per_token = get_eurc_price_per_token(amount, current_round);
        let total_eurc_price = amount
            .checked_mul(eurc_per_token)
            .ok_or(ErrorCode::Overflow)?;

        msg!("🛒 Buying {} tokens for {} EURC ({} EURC/token) - Round: {:?}", 
             amount, total_eurc_price, eurc_per_token, current_round);

        // Transfer EURC from buyer to treasury
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_eurc_ata.to_account_info(),
            to: ctx.accounts.treasury_eurc_ata.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
        );

        token::transfer(cpi_ctx, total_eurc_price)?;

        // Mint tokens
        let seeds = b"mint";
        let bump = ctx.bumps.mint_authority_pda;
        let signer_seeds: &[&[&[u8]]] = &[&[seeds, &[bump]]];

        let mint_cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.buyer_ata.to_account_info(),
            authority: ctx.accounts.mint_authority_pda.to_account_info(),
        };

        let mint_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_2022_program.to_account_info(),
            mint_cpi_accounts,
            signer_seeds,
        );

        let decimals = ctx.accounts.mint.decimals;
        let base_units = amount
            .checked_mul(10u64.pow(decimals as u32))
            .ok_or(ErrorCode::Overflow)?;

        token22::mint_to(mint_cpi_ctx, base_units)?;

        // Update round tokens sold
        update_round_tokens_sold(config, current_round, amount)?;

        // EMIT EVENT
        emit!(TokenPurchasedWithEurc {
            buyer: ctx.accounts.buyer.key(),
            token_amount: amount,
            eurc_amount: total_eurc_price,
            eurc_per_token: eurc_per_token,
            treasury: config.treasury,
            timestamp: Clock::get()?.unix_timestamp,
            payment_method: "EURC".to_string(),
            tier: get_tier_name(amount),
            round: current_round as u8,
        });

        let remaining = TOKENS_PER_ROUND - get_round_tokens_sold(config, current_round);
        msg!("✅ EURC purchase successful! {} tokens at round: {:?}, tier: {}, remaining: {} tokens", 
             amount, current_round, get_tier_name(amount), remaining);
        Ok(())
    }

    // Add function to get current round info
    pub fn get_round_info(ctx: Context<GetRoundInfo>) -> Result<RoundInfo> {
        let config = &ctx.accounts.config;
        let current_round = get_current_round(config)?;
        let current_time = Clock::get()?.unix_timestamp;
        let round_duration: i64 = 30 * 24 * 60 * 60;
        
        let round_start = config.sale_start_time + ((current_round as i64 - 1) * round_duration);
        let round_end = round_start + round_duration;
        let round_tokens_sold = get_round_tokens_sold(config, current_round);
        let remaining_tokens = TOKENS_PER_ROUND.checked_sub(round_tokens_sold).unwrap_or(0);
        
        Ok(RoundInfo {
            current_round: current_round as u8,
            round_start_time: round_start,
            round_end_time: round_end,
            sale_start_time: config.sale_start_time,
            total_duration: round_duration * 3,
            round_tokens_sold,
            remaining_tokens,
            tokens_per_round: TOKENS_PER_ROUND,
        })
    }

    // Add function to get all rounds info
    pub fn get_all_rounds_info(ctx: Context<GetRoundInfo>) -> Result<AllRoundsInfo> {
        let config = &ctx.accounts.config;
        
        Ok(AllRoundsInfo {
            round1: RoundDetails {
                tokens_sold: config.round1_tokens_sold,
                remaining: TOKENS_PER_ROUND.checked_sub(config.round1_tokens_sold).unwrap_or(0),
                total: TOKENS_PER_ROUND,
            },
            round2: RoundDetails {
                tokens_sold: config.round2_tokens_sold,
                remaining: TOKENS_PER_ROUND.checked_sub(config.round2_tokens_sold).unwrap_or(0),
                total: TOKENS_PER_ROUND,
            },
            round3: RoundDetails {
                tokens_sold: config.round3_tokens_sold,
                remaining: TOKENS_PER_ROUND.checked_sub(config.round3_tokens_sold).unwrap_or(0),
                total: TOKENS_PER_ROUND,
            },
            total_tokens_sold: config.round1_tokens_sold
                .checked_add(config.round2_tokens_sold)
                .and_then(|sum| sum.checked_add(config.round3_tokens_sold))
                .unwrap_or(0),
            total_tokens: TOKENS_PER_ROUND * 3,
        })
    }
}

// Add new account for round info
#[derive(Accounts)]
pub struct GetRoundInfo<'info> {
    #[account(seeds = [b"token_sale_config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

// Add RoundInfo struct for returning round information
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

// Add struct for all rounds information
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

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Config::SIZE,
        seeds = [b"token_sale_config"],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Treasury wallet to receive SOL
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: Token mint address
    pub mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OnlyOwner<'info> {
    #[account(
        mut,
        seeds = [b"token_sale_config"],
        bump = config.bump,
        constraint = config.owner == payer.key() @ ErrorCode::Unauthorized
    )]
    pub config: Account<'info, Config>,

    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(
        seeds = [b"token_sale_config"],
        bump = config.bump,
        constraint = config.initialized @ ErrorCode::NotInitialized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    /// CHECK: Treasury account
    #[account(
        mut, 
        address = config.treasury @ ErrorCode::InvalidTreasury
    )]
    pub treasury: UncheckedAccount<'info>,
    
    #[account(
        mut,
        address = config.mint @ ErrorCode::InvalidMint
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: Mint authority - constrained by seeds
    #[account(
        seeds = [b"mint"], 
        bump
    )]
    pub mint_authority_pda: UncheckedAccount<'info>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_2022_program,
    )]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyWithEurc<'info> {
    #[account(
        seeds = [b"token_sale_config"],
        bump = config.bump,
        constraint = config.initialized @ ErrorCode::NotInitialized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    /// CHECK: Your token mint
    #[account(mut, address = config.mint @ ErrorCode::InvalidMint)]
    pub mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: EURC mint address (fixed for EURC)
    pub eurc_mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: Mint authority PDA - USE CONSISTENT SEEDS
    #[account(seeds = [b"mint"], bump)]
    pub mint_authority_pda: UncheckedAccount<'info>,
    
    /// CHECK: Buyer's YOUR token account
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_2022_program,
    )]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>,
    
    /// CHECK: Buyer's EURC token account
    #[account(
        mut,
        associated_token::mint = eurc_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_eurc_ata: InterfaceAccount<'info, TokenAccount>,
    
    /// CHECK: Treasury's EURC token account
    #[account(
        mut,
        associated_token::mint = eurc_mint,
        associated_token::authority = config.treasury,
        associated_token::token_program = token_program,
    )]
    pub treasury_eurc_ata: InterfaceAccount<'info, TokenAccount>,
    
    pub token_2022_program: Program<'info, Token2022>,
    pub token_program: Program<'info, Token>,  // For EURC transfers
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// ✅ Define wrapper account structs for airdrop in the main module
#[derive(Accounts)]
pub struct SetMerkleRoot<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + instructions::airdrop::AirdropConfig::INIT_SPACE,
        seeds = [b"airdrop_config"],
        bump
    )]
    pub config: Account<'info, instructions::airdrop::AirdropConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [b"airdrop_config"],
        bump
    )]
    pub config: Account<'info, instructions::airdrop::AirdropConfig>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = claimer,
        associated_token::token_program = token_2022_program,
    )]
    pub claimer_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(seeds = [b"mint"], bump)]
    /// CHECK: PDA as mint authority
    pub mint_authority_pda: UncheckedAccount<'info>,

    pub claimer: Signer<'info>,

    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

// Update Config account to include round token counters and sale start time
#[account]
pub struct Config {
    pub initialized: bool,
    pub owner: Pubkey,
    pub treasury: Pubkey,
    pub mint: Pubkey,
    pub price_lamports_per_token: u64,
    pub paused: bool,
    pub bump: u8,
    pub sale_start_time: i64,
    pub round1_tokens_sold: u64, // Tokens sold in round 1
    pub round2_tokens_sold: u64, // Tokens sold in round 2
    pub round3_tokens_sold: u64, // Tokens sold in round 3
}

impl Config {
    pub const SIZE: usize = 1 + 32 + 32 + 32 + 8 + 1 + 1 + 8 + 8 + 8 + 8;
}

#[error_code]
pub enum ErrorCode {
    #[msg("Not initialized")]
    NotInitialized,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Sale is paused")]
    SalePaused,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid treasury address")]
    InvalidTreasury,
    #[msg("Invalid mint address")]
    InvalidMint,
    #[msg("Insufficient payment")]
    InsufficientPayment,
    #[msg("Sale has ended")]
    SaleEnded,
    #[msg("Round limit exceeded")]
    RoundLimitExceeded,
}