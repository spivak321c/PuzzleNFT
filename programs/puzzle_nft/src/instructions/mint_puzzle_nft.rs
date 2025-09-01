use anchor_lang::prelude::*;
use mpl_core::{
    instructions::CreateV1CpiBuilder,
    types::{
        Attribute,
        Attributes,
        DataState,
        PluginAuthorityPair,
        PluginAuthority,
    },
};
use solana_program::sysvar::instructions;

/// Accounts for the `mint_puzzle_nft` instruction.
#[derive(Accounts)]
pub struct MintPuzzleNft<'info> {
    /// The payer and owner of the new NFT.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// The NFT asset to be created (validated by Metaplex Core).
    #[account(mut)]
    pub asset: Signer<'info>,
    /// The collection the NFT belongs to (validated by Metaplex Core).
    /// CHECK: This account is validated by Metaplex Core during NFT creation
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// The collection authority PDA for signing.
    /// CHECK: This account is validated as a PDA with the correct seeds
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,
    /// The Solana system program.
    pub system_program: Program<'info, System>,
    /// Instructions sysvar for CPI validation.
    /// CHECK: This account is validated by checking its address matches instructions::ID
    #[account(address = instructions::ID)]
    pub sysvar_instructions: AccountInfo<'info>,
    /// The Metaplex Core program.
    /// CHECK: This account is validated by checking its address matches mpl_core::ID
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

/// Mints a new puzzle NFT with specified metadata and puzzle attributes.
///
/// Creates an NFT with an Attributes plugin containing puzzle data (type, difficulty,
/// puzzle number, solution hash). Emits a `PuzzleMinted` event on success.
impl<'info> MintPuzzleNft<'info> {
    pub fn mint_puzzle_nft(
        &mut self,
        bumps: MintPuzzleNftBumps,
        name: String,
        uri: String,
        puzzle_type: u8,
        difficulty: u8,
    ) -> Result<()> {
        let seeds = [b"authority".as_ref(), &[bumps.authority]];
        let signer_seeds = &[&seeds[..]];

        // Validate puzzle type
        let puzzle_type_name = get_puzzle_type_name(puzzle_type)?;

        // Generate simple puzzle data
        let slot = Clock::get()?.slot;
        let puzzle_number = generate_puzzle_number(slot, puzzle_type, difficulty);
        let solution_hash = generate_solution_hash(puzzle_number);

        // Create puzzle attributes
        let attributes_plugin = PluginAuthorityPair {
            plugin: mpl_core::types::Plugin::Attributes(Attributes {
                attribute_list: vec![
                    Attribute {
                        key: "puzzle_type".to_string(),
                        value: puzzle_type_name.clone(),
                    },
                    Attribute {
                        key: "difficulty".to_string(),
                        value: difficulty.to_string(),
                    },
                    Attribute {
                        key: "puzzle_number".to_string(),
                        value: puzzle_number.to_string(),
                    },
                    Attribute {
                        key: "solution_hash".to_string(),
                        value: solution_hash,
                    },
                    Attribute {
                        key: "solved".to_string(),
                        value: "false".to_string(),
                    },
                    Attribute {
                        key: "mint_slot".to_string(),
                        value: slot.to_string(),
                    },
                ],
            }),
            authority: None,
        };

        let plugins = vec![attributes_plugin];

        CreateV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .authority(Some(&self.authority.to_account_info()))
            .payer(&self.payer.to_account_info())
            .owner(Some(&self.payer.to_account_info()))
            .update_authority(None)
            .system_program(&self.system_program.to_account_info())
            .data_state(DataState::AccountState)
            .name(name)
            .uri(uri)
            .plugins(plugins)
            .add_remaining_account(&self.sysvar_instructions.to_account_info(), false, false)
            .invoke_signed(signer_seeds)?;

        emit!(crate::PuzzleMinted {
            asset: self.asset.key(),
            puzzle_type: puzzle_type_name,
            puzzle_number,
            minter: self.payer.key(),
        });

        msg!("Puzzle NFT minted successfully!");
        Ok(())
    }
}

/// Returns the name of the puzzle type based on its ID.
///
/// # Errors
/// Returns `InvalidPuzzleType` if the puzzle type is unknown.
fn get_puzzle_type_name(puzzle_type: u8) -> Result<String> {
    match puzzle_type {
        0 => Ok("math_factor".to_string()),
        1 => Ok("hash_riddle".to_string()),
        2 => Ok("pattern".to_string()),
        _ => Err(crate::PuzzleError::InvalidPuzzleType.into()),
    }
}

/// Generates a unique puzzle number based on slot, puzzle type, and difficulty.
fn generate_puzzle_number(slot: u64, puzzle_type: u8, difficulty: u8) -> u64 {
    let base = ((slot % 1000) + 1) * (difficulty as u64 + 1);
    let modifier = (puzzle_type as u64 + 1) * 100;
    base + modifier
}

/// Generates a solution hash for the puzzle number.
fn generate_solution_hash(puzzle_number: u64) -> String {
    let mut hash_value = puzzle_number;
    for _ in 0..3 {
        hash_value = hash_value.wrapping_mul(31).wrapping_add(17);
    }
    format!("{:x}", hash_value)
}