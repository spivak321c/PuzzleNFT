use anchor_lang::prelude::*;

/// Program ID for the Puzzle NFT program.
declare_id!("U8kQ8829D2TvdMEK4CxSoaDYbnRYXYgCEHAYVycHLaT");

mod instructions;
use instructions::*;

/// Puzzle NFT program for creating and managing NFTs with embedded puzzles.
#[program]
pub mod puzzle_nft {
    use super::*;

    /// Creates a new NFT collection using Metaplex Core.
    pub fn create_collection(ctx: Context<CreateCollection>) -> Result<()> {
        ctx.accounts.create_collection(ctx.bumps)
    }

    /// Mints a new puzzle NFT with specified attributes and puzzle data.
    pub fn mint_puzzle_nft(
        ctx: Context<MintPuzzleNft>,
        name: String,
        uri: String,
        puzzle_type: u8,
        difficulty: u8,
    ) -> Result<()> {
        ctx.accounts.mint_puzzle_nft(ctx.bumps, name, uri, puzzle_type, difficulty)
    }

    /// Solves the puzzle in an NFT, updating its attributes and optionally its URI.
    pub fn solve_puzzle(
        ctx: Context<SolvePuzzle>,
        solution: u64,
        new_uri: Option<String>,
    ) -> Result<()> {
        ctx.accounts.solve_puzzle(ctx.bumps, solution, new_uri)
    }
}

/// Event emitted when a puzzle NFT is minted.
#[event]
pub struct PuzzleMinted {
    /// The public key of the minted NFT asset.
    pub asset: Pubkey,
    /// The type of puzzle (e.g., math_factor, hash_riddle).
    pub puzzle_type: String,
    /// The unique puzzle number.
    pub puzzle_number: u64,
    /// The public key of the minter.
    pub minter: Pubkey,
}

/// Event emitted when a puzzle NFT is solved.
#[event]
pub struct PuzzleSolved {
    /// The public key of the solved NFT asset.
    pub asset: Pubkey,
    /// The public key of the solver.
    pub solver: Pubkey,
    /// The timestamp when the puzzle was solved.
    pub solution_time: i64,
    /// The rarity assigned to the solution (e.g., Legendary, Epic).
    pub rarity: String,
}

/// Custom errors for the Puzzle NFT program.
#[error_code]
pub enum PuzzleError {
    /// The provided solution is incorrect.
    #[msg("The provided solution is incorrect")]
    IncorrectSolution,
    
    /// Puzzle data not found in NFT attributes.
    #[msg("Puzzle not found in NFT attributes")]
    PuzzleNotFound,
    
    /// Only the NFT owner can attempt to solve the puzzle.
    #[msg("Only the NFT owner can attempt to solve the puzzle")]
    NotNftOwner,
    
    /// The NFT has already been solved.
    #[msg("NFT has already been solved")]
    AlreadySolved,
    
    /// The provided puzzle type is invalid.
    #[msg("Invalid puzzle type")]
    InvalidPuzzleType,
    
    /// Failed to parse puzzle data from attributes.
    #[msg("Failed to parse puzzle data")]
    FailedToParsePuzzleData,
    
    /// The asset data is invalid.
    #[msg("Invalid asset data")]
    InvalidAssetData,
    
    /// Unauthorized attempt to update the NFT.
    #[msg("Unauthorized update attempt")]
    UnauthorizedUpdate,
    
    /// The collection authority is invalid.
    #[msg("Invalid collection authority")]
    InvalidCollectionAuthority,
    
    /// An attribute was not found in the plugin.
    #[msg("Attribute not found")]
    AttributeNotFound,
}
