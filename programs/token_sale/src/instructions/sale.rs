use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::{self as token22, MintTo, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};
use anchor_spl::token::{self, Transfer, Token};
use pyth_sdk_solana::state::SolanaPriceAccount;
use crate::state::*;
use crate::error::ErrorCode;

// Hardcoded Pyth mainnet addresses for security (we'll use these as keys for mock data)
pub const PYTH_EUR_USD_MAINNET: Pubkey = Pubkey::new_from_array([
    0x45, 0x5f, 0x8d, 0x18, 0x19, 0x3a, 0x9c, 0x1a,
    0x6b, 0x0e, 0x3c, 0x7a, 0x7f, 0x8d, 0x1f, 0x8a,
    0x2c, 0x9c, 0x8e, 0x8f, 0x1a, 0x3c, 0x7a, 0x7f,
    0x8d, 0x1f, 0x8a, 0x2c, 0x9c, 0x8e, 0x8f, 0x1a,
]);

pub const PYTH_SOL_USD_MAINNET: Pubkey = Pubkey::new_from_array([
    0xef, 0x0d, 0x8b, 0x6f, 0x87, 0xda, 0x2b, 0x0e,
    0x6f, 0x9a, 0x1c, 0x8d, 0x8f, 0x8e, 0x8a, 0x2c,
    0x9c, 0x8e, 0x8f, 0x1a, 0x3c, 0x7a, 0x7f, 0x8d,
    0x1f, 0x8a, 0x2c, 0x9c, 0x8e, 0x8f, 0x1a, 0x3c,
]);

// Account structs
#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(
        mut,
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

    // Hardcoded Pyth accounts - we'll use these addresses to determine which mock price to use
    /// CHECK: Pyth EUR/USD price account (hardcoded address)
    #[account(address = PYTH_EUR_USD_MAINNET @ ErrorCode::InvalidPythAccount)]
    pub pyth_eur_usd_account: AccountInfo<'info>,

    /// CHECK: Pyth SOL/USD price account (hardcoded address)
    #[account(address = PYTH_SOL_USD_MAINNET @ ErrorCode::InvalidPythAccount)]
    pub pyth_sol_usd_account: AccountInfo<'info>,

    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyWithEurc<'info> {
    #[account(
        mut,
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

    /// CHECK: Mint authority PDA
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

#[derive(Accounts)]
pub struct BuyWithUsdc<'info> {
    #[account(
        mut,
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

    /// CHECK: USDC mint address (fixed for USDC)
    pub usdc_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Mint authority PDA
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

    /// CHECK: Buyer's USDC token account
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_usdc_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Treasury's USDC token account
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = config.treasury,
        associated_token::token_program = token_program,
    )]
    pub treasury_usdc_ata: InterfaceAccount<'info, TokenAccount>,

    // Hardcoded Pyth account for EUR/USD conversion
    /// CHECK: Pyth EUR/USD price account (hardcoded address)
    #[account(address = PYTH_EUR_USD_MAINNET @ ErrorCode::InvalidPythAccount)]
    pub pyth_eur_usd_account: AccountInfo<'info>,

    pub token_2022_program: Program<'info, Token2022>,
    pub token_program: Program<'info, Token>,  // For USDC transfers
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetRoundInfo<'info> {
    #[account(seeds = [b"token_sale_config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

// Mock price data - using realistic current prices
pub const MOCK_EUR_USD_RATE: u64 = 1_080_000; // 1.08 EUR/USD (6 decimals)
pub const MOCK_SOL_USD_RATE: u64 = 140_000_000; // 140.00 SOL/USD (6 decimals)
pub const MOCK_USDC_USD_RATE: u64 = 1_000_000; // 1.00 USDC/USD (6 decimals)

// Mock price function - uses account addresses to determine which price to return
pub fn get_mock_price(price_account: &AccountInfo) -> Result<u64> {
    msg!("🧪 Using mock price data");

    // Determine which price to return based on the account address
    if price_account.key == &PYTH_EUR_USD_MAINNET {
        msg!("   EUR/USD rate: {}", MOCK_EUR_USD_RATE);
        Ok(MOCK_EUR_USD_RATE)
    } else if price_account.key == &PYTH_SOL_USD_MAINNET {
        msg!("   SOL/USD rate: {}", MOCK_SOL_USD_RATE);
        Ok(MOCK_SOL_USD_RATE)
    } else {
        // Default to USDC rate for any other account
        msg!("   USDC/USD rate: {}", MOCK_USDC_USD_RATE);
        Ok(MOCK_USDC_USD_RATE)
    }
}

// Original Pyth function kept for reference but not used
pub fn _get_validated_price(_price_account: &AccountInfo) -> Result<u64> {
    // This is the original function - we're not using it anymore
    Err(error!(ErrorCode::PriceNotAvailable))
}

// Helper functions
pub fn get_stablecoin_price_per_token(amount: u64, round: Round) -> u64 {
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

// Convert your EURC price to USDC (USDC = $1.00)
pub fn convert_eurc_price_to_usdc(eurc_price: u64, eur_usd_rate: u64) -> Result<u64> {
    // Convert EURC price to USDC: eurc_price * eur_usd_rate / 1_000_000
    eurc_price
        .checked_mul(eur_usd_rate)
        .ok_or(error!(ErrorCode::Overflow))?
        .checked_div(1_000_000)
        .ok_or(error!(ErrorCode::Overflow))
}

// Convert your EURC price to SOL
pub fn convert_eurc_price_to_sol(eurc_price: u64, eur_usd_rate: u64, sol_usd_rate: u64) -> Result<u64> {
    // First convert EURC to USD
    let usd_price = eurc_price
        .checked_mul(eur_usd_rate)
        .ok_or(error!(ErrorCode::Overflow))?
        .checked_div(1_000_000)
        .ok_or(error!(ErrorCode::Overflow))?;

    // Then convert USD to SOL (SOL has 9 decimals, but we want 6 decimal precision)
    usd_price
        .checked_mul(1_000_000_000)
        .ok_or(error!(ErrorCode::Overflow))?
        .checked_div(sol_usd_rate)
        .ok_or(error!(ErrorCode::Overflow))
}

pub fn get_tier_name(amount: u64) -> String {
    match amount {
        a if a < 1000 => "Tier 1 (<1K)".to_string(),
        a if a < 5000 => "Tier 2 (1K-5K)".to_string(),
        a if a < 10000 => "Tier 3 (5K-10K)".to_string(),
        a if a < 25000 => "Tier 4 (10K-25K)".to_string(),
        _ => "Tier 5 (25K+)".to_string(),
    }
}

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

pub fn get_round_tokens_sold(config: &Config, round: Round) -> u64 {
    match round {
        Round::First => config.round1_tokens_sold,
        Round::Second => config.round2_tokens_sold,
        Round::Third => config.round3_tokens_sold,
    }
}

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

// Main sale functions - UPDATED TO USE MOCK PRICES
pub fn buy(ctx: Context<Buy>, amount: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(config.initialized, ErrorCode::NotInitialized);
    require!(!config.paused, ErrorCode::SalePaused);
    require!(amount > 0, ErrorCode::InvalidAmount);

    let current_round = get_current_round(config)?;

    // Check round limit
    check_round_limit(config, amount)?;

    // Get your existing EURC price per token with tiers and rounds
    let eurc_per_token = get_stablecoin_price_per_token(amount, current_round);

    // Use MOCK prices instead of Pyth
    let eur_usd_rate = get_mock_price(&ctx.accounts.pyth_eur_usd_account)?;
    let sol_usd_rate = get_mock_price(&ctx.accounts.pyth_sol_usd_account)?;
    let sol_per_token = convert_eurc_price_to_sol(eurc_per_token, eur_usd_rate, sol_usd_rate)?;

    let total_price = amount
        .checked_mul(sol_per_token)
        .ok_or(ErrorCode::Overflow)?;

    msg!("🛒 Buying {} tokens for {} SOL ({} SOL/token) - Round: {:?}",
         amount, total_price, sol_per_token, current_round);
    msg!("💰 Using mock prices - EUR/USD: {}, SOL/USD: {}", eur_usd_rate, sol_usd_rate);

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
        sol_per_token: sol_per_token,
        eur_usd_rate: eur_usd_rate,
        sol_usd_rate: sol_usd_rate,
        equivalent_eurc_price: eurc_per_token,
        timestamp: Clock::get()?.unix_timestamp,
        payment_method: "SOL".to_string(),
        tier: get_tier_name(amount),
        round: current_round as u8,
    });

    let remaining = TOKENS_PER_ROUND - get_round_tokens_sold(config, current_round);
    msg!("✅ SOL purchase successful! Round: {:?}, Remaining: {} tokens", current_round, remaining);
    Ok(())
}

pub fn buy_with_eurc(ctx: Context<BuyWithEurc>, amount: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(config.initialized, ErrorCode::NotInitialized);
    require!(!config.paused, ErrorCode::SalePaused);
    require!(amount > 0, ErrorCode::InvalidAmount);

    let current_round = get_current_round(config)?;

    // Check round limit
    check_round_limit(config, amount)?;

    // Calculate EURC price based on tiers and current round
    let eurc_per_token = get_stablecoin_price_per_token(amount, current_round);
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

pub fn buy_with_usdc(ctx: Context<BuyWithUsdc>, amount: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(config.initialized, ErrorCode::NotInitialized);
    require!(!config.paused, ErrorCode::SalePaused);
    require!(amount > 0, ErrorCode::InvalidAmount);

    let current_round = get_current_round(config)?;

    // Check round limit
    check_round_limit(config, amount)?;

    // Get your existing EURC price per token with tiers and rounds
    let eurc_per_token = get_stablecoin_price_per_token(amount, current_round);

    // Use MOCK price for EUR/USD conversion
    let eur_usd_rate = get_mock_price(&ctx.accounts.pyth_eur_usd_account)?;
    let usdc_per_token = convert_eurc_price_to_usdc(eurc_per_token, eur_usd_rate)?;

    let total_usdc_price = amount
        .checked_mul(usdc_per_token)
        .ok_or(ErrorCode::Overflow)?;

    msg!("🛒 Buying {} tokens for {} USDC ({} USDC/token) - Round: {:?}",
         amount, total_usdc_price, usdc_per_token, current_round);
    msg!("💰 Using mock EUR/USD rate: {}", eur_usd_rate);

    // Transfer USDC from buyer to treasury
    let cpi_accounts = Transfer {
        from: ctx.accounts.buyer_usdc_ata.to_account_info(),
        to: ctx.accounts.treasury_usdc_ata.to_account_info(),
        authority: ctx.accounts.buyer.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );

    token::transfer(cpi_ctx, total_usdc_price)?;

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
    emit!(TokenPurchasedWithUsdc {
        buyer: ctx.accounts.buyer.key(),
        token_amount: amount,
        usdc_amount: total_usdc_price,
        usdc_per_token: usdc_per_token,
        eur_usd_rate: eur_usd_rate,
        equivalent_eurc_price: eurc_per_token,
        timestamp: Clock::get()?.unix_timestamp,
        payment_method: "USDC".to_string(),
        tier: get_tier_name(amount),
        round: current_round as u8,
    });

    let remaining = TOKENS_PER_ROUND - get_round_tokens_sold(config, current_round);
    msg!("✅ USDC purchase successful! {} tokens at round: {:?}, tier: {}, remaining: {} tokens", 
         amount, current_round, get_tier_name(amount), remaining);
    Ok(())
}

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