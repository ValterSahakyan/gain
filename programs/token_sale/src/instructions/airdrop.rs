use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self as token22, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use sha2::{Digest, Sha256};

use crate::error::ErrorCode;
use crate::state::config::Config;

#[account]
#[derive(InitSpace)]
pub struct AirdropConfig {
    pub merkle_root: [u8; 32],
    pub airdrop_amount: u64,
    pub total_claimed: u64,
    pub max_claims: u64,
    #[max_len(50)]
    pub claimed: Vec<Pubkey>,
    pub airdrop_start_time: i64,
    pub airdrop_end_time: i64,
    pub bump: u8,
}

pub fn set_merkle_root_handler(
    ctx: Context<SetMerkleRoot>, 
    root: [u8; 32],
    airdrop_amount: u64,
    max_claims: u64,
    start_time: i64,
    end_time: i64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Check if this is a new config or update
    let is_new_config = config.merkle_root == [0u8; 32]; // Simple check

    config.merkle_root = root;
    config.airdrop_amount = airdrop_amount;
    config.max_claims = max_claims;
    config.airdrop_start_time = start_time;
    config.airdrop_end_time = end_time;

    if is_new_config {
        // Initialize for new airdrop
        config.total_claimed = 0;
        config.claimed = Vec::new();
        config.bump = ctx.bumps.config;
        msg!("New airdrop configured");
    } else {
        // Keep existing claimed data for updates
        // Or reset if you want fresh start:
        // config.total_claimed = 0;
        // config.claimed.clear();
        msg!("Existing airdrop updated");
    }

    msg!("Airdrop: amount={}, max_claims={}", airdrop_amount, max_claims);
    Ok(())
}

pub fn claim_handler(ctx: Context<Claim>, amount: u64, proof: Vec<[u8; 32]>) -> Result<()> {
    let claimer = ctx.accounts.claimer.key();
    let current_time = Clock::get()?.unix_timestamp;

    // Check if airdrop is active
    require!(
        current_time >= ctx.accounts.airdrop_config.airdrop_start_time && 
        current_time <= ctx.accounts.airdrop_config.airdrop_end_time,
        ErrorCode::SaleEnded
    );

    // Check max claims
    require!(
        ctx.accounts.airdrop_config.total_claimed < ctx.accounts.airdrop_config.max_claims,
        ErrorCode::RoundLimitExceeded
    );

    // Verify merkle proof
    require!(
        verify_merkle_proof(
            &ctx.accounts.airdrop_config.merkle_root, 
            &claimer, 
            amount, 
            &proof
        ),
        ErrorCode::Unauthorized
    );

    // Prevent double-claim
    require!(
        !ctx.accounts.airdrop_config.claimed.contains(&claimer),
        ErrorCode::Unauthorized
    );

    // Add to claimed list and update counters
    ctx.accounts.airdrop_config.claimed.push(claimer);
    ctx.accounts.airdrop_config.total_claimed += 1;

    // Adjust decimals
    let decimals = ctx.accounts.mint.decimals;
    let factor = 10u64.checked_pow(decimals as u32).unwrap();
    let base_units = amount.checked_mul(factor).unwrap();

    // Mint tokens
    let seeds = b"mint"; // Make sure this matches your sale.rs
    let bump = ctx.bumps.mint_authority_pda;
    let signer_seeds: &[&[&[u8]]] = &[&[seeds, &[bump]]];

    let cpi_accounts = token22::MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.claimer_ata.to_account_info(),
        authority: ctx.accounts.mint_authority_pda.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_2022_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    
    token22::mint_to(cpi_ctx, base_units)?;
    
    // Emit claim event
    emit!(ClaimEvent {
        claimer,
        amount: base_units,
        timestamp: current_time,
        merkle_root: ctx.accounts.airdrop_config.merkle_root,
    });
    
    msg!("Airdrop claimed by: {}, amount: {}", claimer, base_units);
    Ok(())
}

fn verify_merkle_proof(
    root: &[u8; 32],
    claimer: &Pubkey,
    amount: u64,
    proof: &Vec<[u8; 32]>
) -> bool {
    // Create leaf node: hash(claimer_pubkey || amount)
    let mut hasher = Sha256::new();
    hasher.update(claimer.as_ref());
    hasher.update(&amount.to_le_bytes());
    let mut leaf = hasher.finalize().to_vec();
    
    // Verify proof
    for proof_item in proof {
        let mut hasher = Sha256::new();
        
        // Compare to determine order (left or right)
        if leaf.as_slice() < proof_item {
            hasher.update(&leaf);
            hasher.update(proof_item);
        } else {
            hasher.update(proof_item);
            hasher.update(&leaf);
        }
        
        leaf = hasher.finalize().to_vec();
    }
    
    // Check if final hash matches root
    leaf.as_slice() == root
}

#[event]
pub struct ClaimEvent {
    pub claimer: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub merkle_root: [u8; 32],
}

// Context structs for airdrop
#[derive(Accounts)]
pub struct SetMerkleRoot<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        constraint = sale_config.owner == authority.key() @ ErrorCode::Unauthorized, // Fixed constraint
    )]
    pub sale_config: Account<'info, Config>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + AirdropConfig::INIT_SPACE,
        seeds = [b"airdrop-config"],
        bump
    )]
    pub config: Account<'info, AirdropConfig>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"airdrop-config"],
        bump = airdrop_config.bump
    )]
    pub airdrop_config: Account<'info, AirdropConfig>,
    
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: We'll initialize this ATA if needed
    #[account(
        init_if_needed,
        payer = claimer,
        associated_token::mint = mint,
        associated_token::authority = claimer,
        associated_token::token_program = token_2022_program,
    )]
    pub claimer_ata: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        seeds = [b"mint"],
        bump
    )]
    /// CHECK: This is the mint authority PDA
    pub mint_authority_pda: UncheckedAccount<'info>,
    
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// Utility functions
impl AirdropConfig {
    pub fn can_claim(&self, claimer: &Pubkey, amount: u64, proof: &Vec<[u8; 32]>) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        
        current_time >= self.airdrop_start_time && 
        current_time <= self.airdrop_end_time &&
        !self.claimed.contains(claimer) && 
        self.total_claimed < self.max_claims &&
        verify_merkle_proof(&self.merkle_root, claimer, amount, proof)
    }

    pub fn get_remaining_claims(&self) -> u64 {
        self.max_claims.saturating_sub(self.total_claimed)
    }

    pub fn get_airdrop_info(&self) -> AirdropInfo {
        AirdropInfo {
            merkle_root: self.merkle_root,
            airdrop_amount: self.airdrop_amount,
            total_claimed: self.total_claimed,
            max_claims: self.max_claims,
            airdrop_start_time: self.airdrop_start_time,
            airdrop_end_time: self.airdrop_end_time,
            remaining_claims: self.get_remaining_claims(),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AirdropInfo {
    pub merkle_root: [u8; 32],
    pub airdrop_amount: u64,
    pub total_claimed: u64,
    pub max_claims: u64,
    pub airdrop_start_time: i64,
    pub airdrop_end_time: i64,
    pub remaining_claims: u64,
}