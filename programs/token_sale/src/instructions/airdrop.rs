use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self as token22, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

#[account]
#[derive(InitSpace)]
pub struct AirdropConfig {
    pub merkle_root: [u8; 32],
    #[max_len(1000)]
    pub claimed: Vec<Pubkey>,
}

// Remove the #[derive(Accounts)] structs from airdrop.rs
// We'll only keep the account struct and handler functions

pub fn set_merkle_root_handler(ctx: Context<crate::SetMerkleRoot>, root: [u8; 32]) -> Result<()> {
    ctx.accounts.config.merkle_root = root;
    Ok(())
}

pub fn claim_handler(ctx: Context<crate::Claim>, amount: u64, proof: Vec<[u8; 32]>) -> Result<()> {
    let claimer = ctx.accounts.claimer.key();

    // Verify merkle proof
    require!(
        verify_merkle_proof(ctx.accounts.config.merkle_root, claimer, amount, proof),
        ErrorCode::InvalidProof
    );

    // Prevent double-claim
    require!(
        !ctx.accounts.config.claimed.contains(&claimer),
        ErrorCode::AlreadyClaimed
    );
    ctx.accounts.config.claimed.push(claimer);

    // Adjust decimals
    let decimals = ctx.accounts.mint.decimals;
    let factor = 10u64.checked_pow(decimals.into()).unwrap();
    let base_units = amount.checked_mul(factor).unwrap();

    // Mint tokens
    let seeds = &[b"mint".as_ref(), &[ctx.bumps.mint_authority_pda]];
    let signer_seeds: &[&[&[u8]]] = &[seeds];

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
    Ok(())
}

fn verify_merkle_proof(
    _root: [u8; 32],
    _claimer: Pubkey,
    _amount: u64,
    _proof: Vec<[u8; 32]>
) -> bool {
    // TODO: implement real merkle verification
    true
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid merkle proof")]
    InvalidProof,
    #[msg("Already claimed")]
    AlreadyClaimed,
}