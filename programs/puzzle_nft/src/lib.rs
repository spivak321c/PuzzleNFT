use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::{
    instructions::{CreateV1CpiBuilder, UpdateV1CpiBuilder},
    types::{Collection, Creator, TokenStandard},
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod puzzle_nft {
    use super::*;

    pub fn mint_puzzle_nft(
        ctx: Context<MintPuzzleNft>,
        puzzle_description: String,
        modulus: u64,
        target: u64,
        uri: String,
    ) -> Result<()> {
        let puzzle_account = &mut ctx.accounts.puzzle_account;
        let mint = ctx.accounts.mint.key();
        let authority = ctx.accounts.authority.key();
        let clock = Clock::get()?;
        
        // Generate seed from minter's pubkey and current slot
        let seed = u64::from_le_bytes(
            authority.to_bytes()[0..8].try_into().unwrap()
        ) ^ clock.slot;
        
        // Initialize puzzle account
        puzzle_account.puzzle_description = puzzle_description;
        puzzle_account.seed = seed;
        puzzle_account.modulus = modulus;
        puzzle_account.target = target;
        puzzle_account.solved = false;
        puzzle_account.solver = None;
        puzzle_account.solved_at_slot = None;
        puzzle_account.uri = uri.clone();
        
        // Create metadata for the NFT
        let creators = vec![
            Creator {
                address: ctx.accounts.authority.key(),
                verified: true,
                share: 100,
            }
        ];
        
        // Create the NFT metadata
        let name = format!("Puzzle NFT #{}", seed);
        let symbol = "PUZZLE".to_string();
        
        // Create metadata account
        CreateV1CpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .metadata(&ctx.accounts.metadata.to_account_info())
            .mint(&ctx.accounts.mint.to_account_info(), false)
            .authority(&ctx.accounts.authority.to_account_info())
            .payer(&ctx.accounts.payer.to_account_info())
            .update_authority(&ctx.accounts.authority.to_account_info(), true)
            .name(name)
            .symbol(symbol)
            .uri(uri)
            .creators(creators)
            .seller_fee_basis_points(0)
            .token_standard(TokenStandard::NonFungible)
            .system_program(&ctx.accounts.system_program.to_account_info())
            .sysvar_instructions(&ctx.accounts.sysvar_instructions.to_account_info())
            .invoke()?;
        
        // Mint one token to the associated token account
        anchor_spl::token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            1,
        )?;
        
        // Emit event
        emit!(PuzzleMinted {
            mint,
            authority,
            seed,
            modulus,
            target,
            puzzle_description: puzzle_account.puzzle_description.clone(),
        });
        
        Ok(())
    }

    pub fn solve_puzzle(
        ctx: Context<SolvePuzzle>,
        solution: u64,
        new_uri: String,
    ) -> Result<()> {
        let puzzle_account = &mut ctx.accounts.puzzle_account;
        let solver = ctx.accounts.solver.key();
        let mint = ctx.accounts.mint.key();
        
        // Check if puzzle is already solved
        if puzzle_account.solved {
            return err!(PuzzleNftError::AlreadySolved);
        }
        
        // Verify solution: (solution * seed) % modulus == target
        let calculated = (solution * puzzle_account.seed) % puzzle_account.modulus;
        if calculated != puzzle_account.target {
            return err!(PuzzleNftError::InvalidSolution);
        }
        
        // Update puzzle state
        puzzle_account.solved = true;
        puzzle_account.solver = Some(solver);
        puzzle_account.solved_at_slot = Some(Clock::get()?.slot);
        puzzle_account.uri = new_uri.clone();
        
        // Update metadata URI
        UpdateV1CpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .metadata(&ctx.accounts.metadata.to_account_info())
            .update_authority(&ctx.accounts.authority.to_account_info())
            .mint(&ctx.accounts.mint.to_account_info())
            .uri(Some(new_uri.clone()))
            .name(None)
            .symbol(None)
            .creators(None)
            .seller_fee_basis_points(None)
            .primary_sale_happened(None)
            .is_mutable(None)
            .token_standard(None)
            .collection(None)
            .uses(None)
            .collection_details(None)
            .rule_set(None)
            .authorization_data(None)
            .token_program(&ctx.accounts.token_program.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .sysvar_instructions(&ctx.accounts.sysvar_instructions.to_account_info())
            .invoke()?;
        
        // Emit event
        emit!(PuzzleSolved {
            mint,
            solver,
            solution,
            solved_at_slot: puzzle_account.solved_at_slot.unwrap(),
        });
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintPuzzleNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority,
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = payer,
        seeds = [b"puzzle", mint.key().as_ref()],
        bump,
        space = 8 + PuzzleNFT::INIT_SPACE
    )]
    pub puzzle_account: Account<'info, PuzzleNFT>,
    
    /// CHECK: Handled by Metaplex CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    
    /// CHECK: Metaplex program
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,
    
    /// CHECK: Sysvar instructions
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SolvePuzzle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub solver: Signer<'info>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"puzzle", mint.key().as_ref()],
        bump,
    )]
    pub puzzle_account: Account<'info, PuzzleNFT>,
    
    /// CHECK: Handled by Metaplex CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    /// CHECK: Authority of the NFT
    pub authority: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    
    /// CHECK: Metaplex program
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,
    
    /// CHECK: Sysvar instructions
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct PuzzleNFT {
    #[max_len(255)]
    pub puzzle_description: String,
    pub seed: u64,
    pub modulus: u64,
    pub target: u64,
    pub solved: bool,
    pub solver: Option<Pubkey>,
    pub solved_at_slot: Option<u64>,
    #[max_len(255)]
    pub uri: String,
}

#[error_code]
pub enum PuzzleNftError {
    #[msg("Puzzle has already been solved")]
    AlreadySolved,
    #[msg("Proposed solution is incorrect")]
    InvalidSolution,
}

#[event]
pub struct PuzzleMinted {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub seed: u64,
    pub modulus: u64,
    pub target: u64,
    pub puzzle_description: String,
}

#[event]
pub struct PuzzleSolved {
    pub mint: Pubkey,
    pub solver: Pubkey,
    pub solution: u64,
    pub solved_at_slot: u64,
}