use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::{self as token22, MintTo, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

declare_id!("6ZK4hFGen61b83NHsNTAMq71r3QJCTwknvj4CYfLxdBj");

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
        let config = &ctx.accounts.config;
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
        // ✅ FIXED: Add PDA signing for minting
        let seeds = b"mint";
        let bump = ctx.bumps.mint_authority_pda;  // Get the bump
        let signer_seeds: &[&[&[u8]]] = &[&[seeds, &[bump]]];
    
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.buyer_ata.to_account_info(),
            authority: ctx.accounts.mint_authority_pda.to_account_info(),
        };
    
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_2022_program.to_account_info(),
            cpi_accounts,
            signer_seeds,  // Pass the PDA signer
        );
        let decimals = ctx.accounts.mint.decimals; // Get token decimals
        let base_units = amount
            .checked_mul(10u64.pow(decimals as u32))
            .ok_or(ErrorCode::Overflow)?;
        token22::mint_to(cpi_ctx, base_units)?;
    
        msg!("✅ Purchase successful!");
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
pub struct UpdateMint<'info> {
    #[account(
        mut,  // Add mut here since we're modifying the config
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
        bump,
        mut
    )]
    pub mint_authority_pda: AccountInfo<'info>,
    
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
    pub const SIZE: usize = 1 + 32 + 32 + 32 + 8 + 1 + 1;
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
}