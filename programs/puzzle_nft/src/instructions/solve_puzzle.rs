use anchor_lang::prelude::*;
use mpl_core::{
    instructions::{UpdatePluginV1CpiBuilder, UpdateV1CpiBuilder},
    types::{Plugin, Attributes, Attribute, PluginType},
    fetch_plugin,
    accounts::BaseAssetV1,
};
use solana_program::sysvar::instructions;

/// Accounts for the `solve_puzzle` instruction.
#[derive(Accounts)]
pub struct SolvePuzzle<'info> {
    /// The NFT owner attempting to solve the puzzle.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// The NFT asset to be updated (validated by Metaplex Core).
    /// CHECK: This account is validated by checking its owner is the MPL Core program
    #[account(mut, owner = mpl_core::ID @ crate::PuzzleError::InvalidAssetData)]
    pub asset: UncheckedAccount<'info>,
    /// The collection the asset belongs to (optional, validated by Metaplex Core).
    /// CHECK: This account is validated by Metaplex Core when present
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,
    /// The collection authority PDA for signing updates.
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

/// Solves the puzzle embedded in an NFT, updating its attributes and optionally its URI.
///
/// This instruction validates the provided solution against the puzzle's stored data,
/// checks ownership, and updates the NFT's attributes to mark it as solved with a rarity score.
/// If a new URI is provided, the asset's metadata URI is updated.
impl<'info> SolvePuzzle<'info> {
    pub fn solve_puzzle(&mut self, bumps: SolvePuzzleBumps, solution: u64, new_uri: Option<String>) -> Result<()> {
        // Deserialize base asset using BaseAssetV1
        let asset: BaseAssetV1 = {
            let asset_data = self.asset.data.borrow();
            BaseAssetV1::from_bytes(&asset_data).map_err(|_| crate::PuzzleError::InvalidAssetData)?
        };

        // Verify ownership - BaseAssetV1 has the owner field directly
        require_eq!(
            asset.owner,
            self.owner.key(),
            crate::PuzzleError::NotNftOwner
        );

        // Fetch attributes plugin - use BaseAssetV1 and Attributes as type parameters
        let (_plugin_authority, attributes, _) = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .map_err(|_| crate::PuzzleError::AttributeNotFound)?;
        
        let attribute_list = &attributes.attribute_list;

        // Check if already solved
        let solved_attr = attribute_list
            .iter()
            .find(|attr| attr.key == "solved")
            .ok_or(crate::PuzzleError::AttributeNotFound)?;
        if solved_attr.value == "true" {
            return Err(crate::PuzzleError::AlreadySolved.into());
        }

        // Validate solution
        let puzzle_number = attribute_list
            .iter()
            .find(|attr| attr.key == "puzzle_number")
            .ok_or(crate::PuzzleError::PuzzleNotFound)?
            .value
            .parse::<u64>()
            .map_err(|_| crate::PuzzleError::FailedToParsePuzzleData)?;
        let solution_hash = attribute_list
            .iter()
            .find(|attr| attr.key == "solution_hash")
            .ok_or(crate::PuzzleError::PuzzleNotFound)?
            .value
            .clone();

        // Simple validation: for math_factor, assume solution is a factor of puzzle_number
        let puzzle_type = attribute_list
            .iter()
            .find(|attr| attr.key == "puzzle_type")
            .ok_or(crate::PuzzleError::PuzzleNotFound)?
            .value
            .clone();
        if puzzle_type == "math_factor" {
            if puzzle_number % solution != 0 {
                return Err(crate::PuzzleError::IncorrectSolution.into());
            }
        } else {
            // For other puzzle types, compare solution_hash (simplified for demo)
            let computed_hash = format!("{:x}", solution.wrapping_mul(31).wrapping_add(17));
            if computed_hash != solution_hash {
                return Err(crate::PuzzleError::IncorrectSolution.into());
            }
        }

        // Determine rarity based on timestamp (simplified, not secure)
        let current_timestamp = Clock::get()?.unix_timestamp;
        let rarity = if current_timestamp % 100 < 10 {
            "Legendary"
        } else if current_timestamp % 100 < 30 {
            "Epic"
        } else if current_timestamp % 100 < 60 {
            "Rare"
        } else {
            "Common"
        };

        // Create new attributes list (merge existing with new)
        let mut updated_attributes = attribute_list.clone();
        
        // Create string values to avoid temporary borrowing issues
        let solver_string = self.owner.key().to_string();
        let solution_string = solution.to_string();
        let timestamp_string = current_timestamp.to_string();
        
        // Update or add the solution attributes
        let solution_attributes = vec![
            ("solved", "true"),
            ("solver", &solver_string),
            ("solution", &solution_string),
            ("solve_timestamp", &timestamp_string),
            ("rarity", rarity),
        ];

        for (key, value) in solution_attributes {
            if let Some(attr) = updated_attributes.iter_mut().find(|attr| attr.key == key) {
                attr.value = value.to_string();
            } else {
                updated_attributes.push(Attribute {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }
        }

        // Handle seeds for collection authority
        let seeds = [b"authority".as_ref(), &[bumps.authority]];
        let signer_seeds = &[&seeds[..]];

        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(
                self.collection
                    .as_ref()
                    .map(|c| c.to_account_info())
                    .as_ref()
            )
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes {
                attribute_list: updated_attributes,
            }))
            .add_remaining_account(&self.sysvar_instructions.to_account_info(), false, false)
            .invoke_signed(signer_seeds)?;

        // Update URI if provided
        if let Some(uri) = new_uri {
            UpdateV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .asset(&self.asset.to_account_info())
                .collection(
                    self.collection
                        .as_ref()
                        .map(|c| c.to_account_info())
                        .as_ref()
                )
                .payer(&self.owner.to_account_info())
                .authority(Some(&self.authority.to_account_info()))
                .system_program(&self.system_program.to_account_info())
                .new_uri(uri.clone())
                .invoke_signed(signer_seeds)?;
            msg!("URI updated to: {}", uri);
        }

        emit!(crate::PuzzleSolved {
            asset: self.asset.key(),
            solver: self.owner.key(),
            solution_time: current_timestamp,
            rarity: rarity.to_string(),
        });

        msg!("Puzzle solved successfully! Rarity: {}", rarity);
        Ok(())
    }
}
