use anchor_lang::prelude::*;
use mpl_core::state::{Asset, PluginAttribute, PluginType};
use solana_program::hash::{hash, Hash};

/// Puzzle parameters structure
#[derive(Debug, Clone)]
pub struct PuzzleParameters {
    pub number: u64,
    pub hash: Hash,
    pub puzzle_type: PuzzleType,
}

/// Types of puzzles that can be generated
#[derive(Debug, Clone, PartialEq)]
pub enum PuzzleType {
    MathFactor,
    HashRiddle,
}

impl PuzzleType {
    pub fn to_string(&self) -> String {
        match self {
            PuzzleType::MathFactor => "math_factor".to_string(),
            PuzzleType::HashRiddle => "hash_riddle".to_string(),
        }
    }
}

/// Generate a puzzle based on on-chain data
pub fn generate_puzzle(
    pubkey: &Pubkey,
    slot: u64,
) -> Result<PuzzleParameters> {
    // Create a deterministic seed from the slot and pubkey
    let seed = format!("{}{}", slot, pubkey);
    let hash_result = hash(seed.as_bytes());
    
    // Use the first byte to determine puzzle type
    let puzzle_type = if hash_result.to_bytes()[0] % 2 == 0 {
        PuzzleType::MathFactor
    } else {
        PuzzleType::HashRiddle
    };
    
    // Generate a number between 10 and 100 that has multiple factors
    let hash_bytes = hash_result.to_bytes();
    let mut number = (u64::from(hash_bytes[0]) % 45 + 10) * 2; // Even number between 10 and 100
    
    // Ensure the number has at least 3 factors for an interesting puzzle
    if number < 20 {
        number = 20; // 20 has factors 1, 2, 4, 5, 10, 20
    }
    
    Ok(PuzzleParameters {
        number,
        hash: hash_result,
        puzzle_type,
    })
}

/// Create puzzle attributes for the NFT
pub fn create_puzzle_attributes(puzzle_params: &PuzzleParameters) -> Vec<PluginAttribute> {
    let mut attributes = vec![
        PluginAttribute {
            trait_type: "puzzle_type".to_string(),
            value: puzzle_params.puzzle_type.to_string(),
        },
        PluginAttribute {
            trait_type: "puzzle_number".to_string(),
            value: puzzle_params.number.to_string(),
        },
        PluginAttribute {
            trait_type: "solved".to_string(),
            value: "false".to_string(),
        },
        PluginAttribute {
            trait_type: "hidden_trait".to_string(),
            value: "???".to_string(),
        },
    ];
    
    // Add hash value for hash riddle puzzles
    if puzzle_params.puzzle_type == PuzzleType::HashRiddle {
        attributes.push(PluginAttribute {
            trait_type: "puzzle_hash".to_string(),
            value: format!("{}", puzzle_params.hash),
        });
    }
    
    attributes
}

/// Verify if the provided solution is correct
pub fn verify_solution(
    asset_account: &AccountInfo,
    solution: u64,
) -> Result<bool> {
    // Read the asset data
    let asset_data = Asset::from_account_info(asset_account)?;
    
    // Extract the puzzle data from attributes
    let mut puzzle_number = None;
    let mut puzzle_type = None;
    let mut puzzle_hash = None;
    
    for plugin in asset_data.plugins {
        if plugin.plugin_type == PluginType::Attributes {
            for attr in plugin.data.iter() {
                match attr.trait_type.as_str() {
                    "puzzle_number" => {
                        puzzle_number = attr.value.parse::<u64>().ok();
                    },
                    "puzzle_type" => {
                        puzzle_type = Some(attr.value.clone());
                    },
                    "puzzle_hash" => {
                        puzzle_hash = Some(attr.value.clone());
                    },
                    _ => {}
                }
            }
        }
    }
    
    let number = puzzle_number.ok_or(ProgramError::InvalidAccountData)?;
    let ptype = puzzle_type.ok_or(ProgramError::InvalidAccountData)?;
    
    match ptype.as_str() {
        "math_factor" => {
            // Check if the solution is a factor of the puzzle number
            if solution > 0 && number % solution == 0 {
                return Ok(true);
            }
        },
        "hash_riddle" => {
            // For hash riddles, the solution is a seed that, when hashed,
            // produces a value with specific properties
            let hash_str = puzzle_hash.ok_or(ProgramError::InvalidAccountData)?;
            let solution_hash = hash(&solution.to_le_bytes());
            
            // Check if the first 4 bytes match
            if solution_hash.to_bytes()[0..4] == hash_str.as_bytes()[0..4] {
                return Ok(true);
            }
        },
        _ => {
            return Err(ProgramError::InvalidAccountData.into());
        }
    }
    
    Ok(false)
}