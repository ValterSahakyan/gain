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
}
// Tiered pricing calculation
pub fn get_eurc_price_per_token(amount: u64) -> u64 {
    match amount {
        a if a < 1000 => 261_000,    // 0.261 EURC
        a if a < 5000 => 252_000,    // 0.252 EURC  
        a if a < 10000 => 243_000,   // 0.243 EURC
        a if a < 25000 => 216_000,   // 0.216 EURC
        _ => 162_000,                // 0.162 EURC
    }
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

        msg!("✅ Token sale initialized");
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
        
        emit!(TokenPurchasedWithSol {
            buyer: ctx.accounts.buyer.key(),
            token_amount: amount,
            sol_amount: total_price,
            treasury: config.treasury,
            timestamp: Clock::get()?.unix_timestamp,
            payment_method: "SOL".to_string(),
            tier: get_tier_name(amount),
        });
        
        msg!("✅ Purchase successful!");
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

        // Calculate EURC price based on tiers and current round
        let eurc_per_token = get_eurc_price_per_token(amount);
        let total_eurc_price = amount
            .checked_mul(eurc_per_token)
            .ok_or(ErrorCode::Overflow)?;

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
        });

        msg!("✅ EURC purchase successful! {} tokens minted at tier: {}", 
             amount, get_tier_name(amount));
        Ok(())
    }

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
}

impl Config {
    pub const SIZE: usize = 1 + 32 + 32 + 32 + 8 + 1 + 1 + 8;
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
}