use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use solana_program::program::invoke_signed;

// Import our modules
mod auth;
mod nft;
mod puzzle;

use puzzle::{generate_puzzle, verify_solution, PuzzleParameters};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod puzzle_nft {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    /// Mint a new NFT with an embedded puzzle
    pub fn mint_puzzle_nft(
        ctx: Context<MintPuzzleNft>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        // Generate puzzle parameters based on on-chain data
        let puzzle_params = generate_puzzle(
            ctx.accounts.payer.key(),
            Clock::get()?.slot,
        )?;

        // Create the NFT using mpl-core
        nft::mint_nft(
            &ctx.accounts.payer,
            &ctx.accounts.asset.to_account_info(),
            &ctx.accounts.mint,
            &ctx.accounts.token_account.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.sysvar_instructions.to_account_info(),
            name,
            symbol,
            uri,
            &puzzle_params,
        )?;

        Ok(())
    }

    /// Attempt to solve the puzzle for an NFT
    pub fn solve_puzzle(
        ctx: Context<SolvePuzzle>,
        solution: u64,
        new_uri: Option<String>,
    ) -> Result<()> {
        // Verify the solution
        let is_correct = verify_solution(
            &ctx.accounts.asset.to_account_info(),
            solution,
        )?;

        if !is_correct {
            return Err(PuzzleError::IncorrectSolution.into());
        }

        // Update the NFT metadata to reflect the solved state
        nft::update_nft_after_solve(
            &ctx.accounts.owner,
            &ctx.accounts.asset.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.sysvar_instructions.to_account_info(),
            Clock::get()?.slot,
            new_uri,
        )?;

        Ok(())
    }
}

/// Context for initializing the program
#[derive(Accounts)]
pub struct Initialize {}

/// Context for minting a new puzzle NFT
#[derive(Accounts)]
pub struct MintPuzzleNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// CHECK: This account will be initialized by mpl-core
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub mint: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        token::mint = mint,
        token::authority = payer,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    
    /// CHECK: Used by mpl-core
    pub sysvar_instructions: UncheckedAccount<'info>,
}

/// Context for solving a puzzle
#[derive(Accounts)]
pub struct SolvePuzzle<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// CHECK: This is the mpl-core asset account
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,
    
    #[account(
        constraint = token_account.owner == owner.key() @ PuzzleError::NotNftOwner,
        constraint = token_account.amount >= 1 @ PuzzleError::NotNftOwner,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    
    /// CHECK: Used by mpl-core
    pub sysvar_instructions: UncheckedAccount<'info>,
}

/// Error codes for the puzzle NFT program
#[error_code]
pub enum PuzzleError {
    #[msg("The provided solution is incorrect")]
    IncorrectSolution,
    
    #[msg("Puzzle not found in NFT metadata")]
    PuzzleNotFound,
    
    #[msg("Only the NFT owner can attempt to solve the puzzle")]
    NotNftOwner,
}