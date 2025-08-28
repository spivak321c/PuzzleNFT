use anchor_lang::prelude::*;
use sha2::{Digest, Sha256};

pub mod puzzle_utils {
    use super::*;

    // Puzzle types
    pub const PUZZLE_TYPE_MATH: u8 = 0;
    pub const PUZZLE_TYPE_HASH: u8 = 1;
    pub const PUZZLE_TYPE_PATTERN: u8 = 2;

    // Generate a puzzle based on the type and seed
    pub fn generate_puzzle(puzzle_type: u8, difficulty: u8, seed: u64) -> (Vec<u8>, Vec<u8>) {
        match puzzle_type {
            PUZZLE_TYPE_MATH => generate_math_puzzle(difficulty, seed),
            PUZZLE_TYPE_HASH => generate_hash_puzzle(difficulty, seed),
            PUZZLE_TYPE_PATTERN => generate_pattern_puzzle(difficulty, seed),
            _ => generate_math_puzzle(difficulty, seed), // Default to math puzzle
        }
    }

    // Generate a math puzzle (e.g., find factors of a number)
    fn generate_math_puzzle(difficulty: u8, seed: u64) -> (Vec<u8>, Vec<u8>) {
        // Use the seed to generate a number to factorize
        let base = match difficulty {
            0 => 100,
            1 => 1000,
            2 => 10000,
            _ => 100000,
        };
        
        let target_number = (seed % base as u64) as u32;
        
        // Find a factor (other than 1 and the number itself)
        let mut factor = 0;
        for i in 2..target_number {
            if target_number % i == 0 {
                factor = i;
                break;
            }
        }
        
        // If no factor found (prime number), use 1
        if factor == 0 {
            factor = 1;
        }
        
        // Create puzzle data (the number to factorize)
        let puzzle_data = target_number.to_le_bytes().to_vec();
        
        // Create solution hash (hash of the factor)
        let mut hasher = Sha256::new();
        hasher.update(factor.to_le_bytes());
        let solution_hash = hasher.finalize().to_vec();
        
        (puzzle_data, solution_hash)
    }

    // Generate a hash puzzle (e.g., find a string that hashes to a specific prefix)
    fn generate_hash_puzzle(difficulty: u8, seed: u64) -> (Vec<u8>, Vec<u8>) {
        // Create a target prefix based on difficulty
        let prefix_length = match difficulty {
            0 => 1,
            1 => 2,
            2 => 3,
            _ => 4,
        };
        
        // Use seed to generate a target prefix
        let mut target_prefix = Vec::new();
        let seed_bytes = seed.to_le_bytes();
        for i in 0..prefix_length {
            target_prefix.push(seed_bytes[i % 8] % 16);
        }
        
        // Find a solution (a simple example - in reality would be more complex)
        let solution = seed.to_string();
        
        // Create puzzle data (the target prefix)
        let puzzle_data = target_prefix;
        
        // Create solution hash
        let mut hasher = Sha256::new();
        hasher.update(solution.as_bytes());
        let solution_hash = hasher.finalize().to_vec();
        
        (puzzle_data, solution_hash)
    }

    // Generate a pattern puzzle (e.g., find the next number in a sequence)
    fn generate_pattern_puzzle(difficulty: u8, seed: u64) -> (Vec<u8>, Vec<u8>) {
        // Generate a sequence based on seed
        let sequence_length = match difficulty {
            0 => 3,
            1 => 4,
            2 => 5,
            _ => 6,
        };
        
        let mut sequence = Vec::new();
        let mut current = seed % 10;
        
        // Different pattern types based on seed
        let pattern_type = (seed / 10) % 3;
        
        for _ in 0..sequence_length {
            sequence.push(current as u8);
            
            // Apply pattern
            match pattern_type {
                0 => current = (current + 2) % 100, // Add 2
                1 => current = (current * 2) % 100, // Multiply by 2
                _ => current = (current * current) % 100, // Square
            }
        }
        
        // The next number in the sequence is the solution
        let solution = current as u32;
        
        // Create puzzle data (the sequence)
        let puzzle_data = sequence;
        
        // Create solution hash
        let mut hasher = Sha256::new();
        hasher.update(solution.to_le_bytes());
        let solution_hash = hasher.finalize().to_vec();
        
        (puzzle_data, solution_hash)
    }

    // Verify a solution for a given puzzle
    pub fn verify_solution(
        puzzle_type: u8,
        puzzle_data: &[u8],
        solution: &[u8],
        solution_hash: &[u8],
    ) -> bool {
        // Hash the provided solution
        let mut hasher = Sha256::new();
        hasher.update(solution);
        let calculated_hash = hasher.finalize();
        
        // Compare with the expected solution hash
        calculated_hash.as_slice() == solution_hash
    }
}