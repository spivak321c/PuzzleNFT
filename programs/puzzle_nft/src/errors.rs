use anchor_lang::prelude::*;

#[error_code]
pub enum PuzzleError {
    #[msg("The provided solution is incorrect")]
    IncorrectSolution,
    
    #[msg("Puzzle not found in NFT metadata")]
    PuzzleNotFound,
    
    #[msg("Only the NFT owner can attempt to solve the puzzle")]
    NotNftOwner,
    
    #[msg("NFT has already been solved")]
    AlreadySolved,
    
    #[msg("Invalid puzzle type")]
    InvalidPuzzleType,
    
    #[msg("Failed to parse puzzle data")]
    FailedToParsePuzzleData,
    
    #[msg("Invalid asset data")]
    InvalidAssetData,
    
    #[msg("Unauthorized update attempt")]
    UnauthorizedUpdate,
}