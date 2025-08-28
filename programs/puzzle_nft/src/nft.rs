use anchor_lang::prelude::*;
use mpl_core::{
    instructions::{CreateArgs, UpdateArgs},
    state::{Asset, Plugin, PluginAttribute, PluginType},
};
use solana_program::program::invoke_signed;

use crate::puzzle::{PuzzleParameters, create_puzzle_attributes};

/// Mint a new NFT with puzzle attributes
pub fn mint_nft<'info>(
    payer: &Signer<'info>,
    asset: &AccountInfo<'info>,
    mint: &Signer<'info>,
    token_account: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    sysvar_instructions: &AccountInfo<'info>,
    name: String,
    symbol: String,
    uri: String,
    puzzle_params: &PuzzleParameters,
) -> Result<()> {
    // Create attributes plugin with puzzle data
    let attributes = create_puzzle_attributes(puzzle_params);

    // Prepare the asset creation arguments
    let create_args = CreateArgs {
        name,
        symbol,
        uri,
        plugins: vec![Plugin {
            plugin_type: PluginType::Attributes,
            is_mutable: true,
            data: attributes,
        }],
        ..CreateArgs::default()
    };

    // Create the asset using mpl-core
    let create_ix = mpl_core::instructions::create(
        mpl_core::instructions::CreateAccounts {
            payer: payer.key(),
            asset: asset.key(),
            mint: mint.key(),
            token_account: token_account.key(),
            authority: payer.key(),
            system_program: system_program.key(),
            token_program: token_program.key(),
            sysvar_instructions: sysvar_instructions.key(),
        },
        create_args,
    );

    // Execute the instruction
    invoke_signed(
        &create_ix,
        &[
            payer.to_account_info(),
            asset.clone(),
            mint.to_account_info(),
            token_account.clone(),
            system_program.clone(),
            token_program.clone(),
            sysvar_instructions.clone(),
        ],
        &[],
    )?;

    Ok(())
}

/// Update NFT metadata after puzzle is solved
pub fn update_nft_after_solve<'info>(
    owner: &Signer<'info>,
    asset: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    sysvar_instructions: &AccountInfo<'info>,
    current_slot: u64,
    new_uri: Option<String>,
) -> Result<()> {
    // Read the current asset data
    let asset_data = Asset::from_account_info(asset)?;
    
    // Prepare updated attributes
    let mut updated_attributes = Vec::new();
    
    // Find and update the attributes plugin
    for plugin in asset_data.plugins {
        if plugin.plugin_type == PluginType::Attributes {
            for attr in plugin.data.iter() {
                if attr.trait_type == "solved" {
                    updated_attributes.push(PluginAttribute {
                        trait_type: "solved".to_string(),
                        value: "true".to_string(),
                    });
                } else if attr.trait_type == "hidden_trait" {
                    updated_attributes.push(PluginAttribute {
                        trait_type: "hidden_trait".to_string(),
                        value: "Legendary Solver".to_string(),
                    });
                } else if attr.trait_type == "solve_time" {
                    // Skip if exists, we'll add a new one
                } else {
                    // Keep other attributes unchanged
                    updated_attributes.push(attr.clone());
                }
            }
        }
    }
    
    // Add solve timestamp
    updated_attributes.push(PluginAttribute {
        trait_type: "solve_time".to_string(),
        value: current_slot.to_string(),
    });

    // Create update instruction
    let update_args = UpdateArgs {
        plugins: Some(vec![Plugin {
            plugin_type: PluginType::Attributes,
            is_mutable: true,
            data: updated_attributes,
        }]),
        uri: new_uri,
        ..UpdateArgs::default()
    };

    // Update the asset using mpl-core
    let update_ix = mpl_core::instructions::update(
        mpl_core::instructions::UpdateAccounts {
            payer: owner.key(),
            asset: asset.key(),
            authority: owner.key(),
            system_program: system_program.key(),
            sysvar_instructions: sysvar_instructions.key(),
        },
        update_args,
    );

    // Execute the instruction
    invoke_signed(
        &update_ix,
        &[
            owner.to_account_info(),
            asset.clone(),
            system_program.clone(),
            sysvar_instructions.clone(),
        ],
        &[],
    )?;

    Ok(())
}