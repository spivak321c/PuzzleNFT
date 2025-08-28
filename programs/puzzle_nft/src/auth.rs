use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};
use mpl_core::state::Asset;

/// Module for ownership and authorization checks
pub mod auth {
    use super::*;

    /// Verify that the signer is the owner of the NFT
    pub fn verify_nft_owner(
        asset_account: &AccountInfo,
        token_account: &Account<TokenAccount>,
        owner_key: &Pubkey,
    ) -> Result<bool> {
        // First, verify the asset exists and is valid
        let asset_data = Asset::from_account_info(asset_account)?;
        
        // Check that the token account belongs to the owner
        if token_account.owner != *owner_key {
            return Ok(false);
        }
        
        // Check that the token account has at least 1 token (ownership)
        if token_account.amount < 1 {
            return Ok(false);
        }
        
        // Check that the token account's mint matches the asset's mint
        if token_account.mint != asset_data.mint {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Verify that the account has authority to update the asset
    pub fn verify_update_authority(
        asset_account: &AccountInfo,
        authority_key: &Pubkey,
    ) -> Result<bool> {
        let asset_data = Asset::from_account_info(asset_account)?;
        
        // Check if the provided authority matches the asset's update authority
        Ok(asset_data.authority == *authority_key)
    }
}