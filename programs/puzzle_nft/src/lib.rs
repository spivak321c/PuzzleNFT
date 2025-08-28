use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use mpl_core::{
    instructions::{CreateArgs, UpdateArgs},
    state::{Asset, AssetArgs, AttributesConfig, AttributesExtension},
};
use sha2::{Digest, Sha256};
use std::str::FromStr;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod puzzle_nft {
    use super::*;

    pub fn initialize_mint(
        ctx: Context<InitializeMint>,
        name: String,
        symbol: String,
        uri: String,
        puzzle_type: u8,
        difficulty: u8,
        puzzle_seed: u64,
        puzzle_data: Vec<u8>,
        solution_hash: Vec<u8>,
    ) -> Result<()> {
        // Initialize the puzzle data account
        let puzzle_data_account = &mut ctx.accounts.puzzle_data;
        puzzle_data_account.mint = ctx.accounts.mint.key();
        puzzle_data_account.owner = ctx.accounts.payer.key();
        puzzle_data_account.authority = ctx.accounts.authority.key();
        puzzle_data_account.is_solved = false;
        puzzle_data_account.solved_at = 0;
        puzzle_data_account.solver = None;
        
        // Set puzzle parameters
        puzzle_data_account.parameters = PuzzleParameters {
            puzzle_type,
            difficulty,
            puzzle_seed: if puzzle_seed == 0 { 
                // Use a deterministic but unpredictable seed if none provided
                let clock = Clock::get()?;
                clock.slot as u64 ^ clock.unix_timestamp as u64
            } else {
                puzzle_seed
            },
            puzzle_data,
            solution_hash,
        };

        // Initialize the mint account
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0, // 0 decimals for NFT
            ctx.accounts.payer.key,
            Some(ctx.accounts.payer.key),
        )?;

        // Create the token account for the payer
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, 1)?;

        // Create the asset using mpl-core
        // This is a simplified version - in a real implementation, you would use
        // the mpl-core CPI to create the asset with attributes for the puzzle
        msg!("NFT minted with embedded puzzle");
        
        Ok(())
    }

    pub fn solve_puzzle(
        ctx: Context<SolvePuzzle>,
        solution: Vec<u8>,
    ) -> Result<()> {
        let puzzle_data = &mut ctx.accounts.puzzle_data;
        
        // Check if the puzzle is already solved
        if puzzle_data.is_solved {
            return err!(PuzzleError::PuzzleAlreadySolved);
        }
        
        // Verify that the owner is attempting to solve
        if puzzle_data.owner != ctx.accounts.owner.key() {
            return err!(PuzzleError::NotNFTOwner);
        }
        
        // Verify the solution
        let mut hasher = Sha256::new();
        hasher.update(&solution);
        let solution_hash = hasher.finalize().to_vec();
        
        if solution_hash != puzzle_data.parameters.solution_hash {
            return err!(PuzzleError::InvalidSolution);
        }
        
        // Update the puzzle state
        puzzle_data.is_solved = true;
        puzzle_data.solved_at = Clock::get()?.unix_timestamp as u64;
        puzzle_data.solver = Some(ctx.accounts.owner.key());
        
        // In a real implementation, you would update the NFT metadata here
        // using mpl-core to reflect the solved state
        msg!("Puzzle solved successfully!");
        
        Ok(())
    }

    pub fn update_metadata(
        ctx: Context<UpdateMetadata>,
        new_uri: String,
    ) -> Result<()> {
        let puzzle_data = &ctx.accounts.puzzle_data;
        
        // Verify that the authority is making the update
        if puzzle_data.authority != ctx.accounts.authority.key() {
            return err!(PuzzleError::UnauthorizedUpdate);
        }
        
        // In a real implementation, you would update the NFT metadata here
        // using mpl-core with the new URI
        msg!("Metadata updated with new URI: {}", new_uri);
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer.key(),
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = payer,
        seeds = [b"puzzle", mint.key().as_ref()],
        bump,
        space = 8 + PuzzleData::SPACE
    )]
    pub puzzle_data: Account<'info, PuzzleData>,
    
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SolvePuzzle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"puzzle", mint.key().as_ref()],
        bump,
    )]
    pub puzzle_data: Account<'info, PuzzleData>,
    
    pub mint: Account<'info, Mint>,
}

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"puzzle", mint.key().as_ref()],
        bump,
    )]
    pub puzzle_data: Account<'info, PuzzleData>,
    
    pub mint: Account<'info, Mint>,
}

#[account]
pub struct PuzzleData {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub authority: Pubkey,
    pub parameters: PuzzleParameters,
    pub is_solved: bool,
    pub solved_at: u64,
    pub solver: Option<Pubkey>,
}

impl PuzzleData {
    pub const SPACE: usize = 32 + // mint
                             32 + // owner
                             32 + // authority
                             PuzzleParameters::SPACE + // parameters
                             1 + // is_solved
                             8 + // solved_at
                             (1 + 32); // solver (Option<Pubkey>)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PuzzleParameters {
    pub puzzle_type: u8,
    pub difficulty: u8,
    pub puzzle_seed: u64,
    pub puzzle_data: Vec<u8>,
    pub solution_hash: Vec<u8>,
}

impl PuzzleParameters {
    pub const SPACE: usize = 1 + // puzzle_type
                             1 + // difficulty
                             8 + // puzzle_seed
                             (4 + 64) + // puzzle_data (Vec<u8> with max 64 bytes)
                             (4 + 32); // solution_hash (Vec<u8> with max 32 bytes)
}

#[error_code]
pub enum PuzzleError {
    #[msg("The provided solution is incorrect")]
    InvalidSolution,
    
    #[msg("Only the NFT owner can attempt to solve the puzzle")]
    NotNFTOwner,
    
    #[msg("This puzzle has already been solved")]
    PuzzleAlreadySolved,
    
    #[msg("Only the authority can update the NFT metadata")]
    UnauthorizedUpdate,
}
