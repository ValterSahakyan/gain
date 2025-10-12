use anchor_lang::prelude::*;
use crate::state::Config;
use crate::error::ErrorCode;

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


pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.owner = ctx.accounts.payer.key();
    config.treasury = ctx.accounts.treasury.key();
    config.mint = ctx.accounts.mint.key();
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
    msg!("💰 Payment methods: SOL, EURC, USDC");
    msg!("💰 Total sale: 3M tokens");
    Ok(())
}

pub fn set_paused(ctx: Context<OnlyOwner>, paused: bool) -> Result<()> {
    ctx.accounts.config.paused = paused;
    Ok(())
}

pub fn update_mint(ctx: Context<OnlyOwner>, new_mint: Pubkey) -> Result<()> {
    ctx.accounts.config.mint = new_mint;
    msg!("Mint updated to: {}", new_mint);
    Ok(())
}
pub fn update_sale_start_time(ctx: Context<OnlyOwner>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let current_time = Clock::get()?.unix_timestamp;
    
    config.sale_start_time = current_time;
    
    // Optionally reset token counters if you want to restart the sale
    config.round1_tokens_sold = 0;
    config.round2_tokens_sold = 0;
    config.round3_tokens_sold = 0;
    
    msg!("Sale start time updated to current time: {}", current_time);
    Ok(())
}