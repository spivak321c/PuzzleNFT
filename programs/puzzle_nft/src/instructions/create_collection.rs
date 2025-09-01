use anchor_lang::prelude::*;
use mpl_core::{
    instructions::CreateCollectionV1CpiBuilder,
    ID as MPL_CORE_PROGRAM_ID,
};

/// Accounts for the `create_collection` instruction.
#[derive(Accounts)]
pub struct CreateCollection<'info> {
    /// The payer creating the collection.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// The collection account to be created.
    #[account(mut)]
    pub collection: Signer<'info>,
    /// The collection authority PDA.
    /// CHECK: This account is validated as a PDA with the correct seeds
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub collection_authority: UncheckedAccount<'info>,
    /// The Solana system program.
    pub system_program: Program<'info, System>,
    /// The Metaplex Core program.
    /// CHECK: This account is validated by checking its address matches MPL_CORE_PROGRAM_ID
    #[account(address = MPL_CORE_PROGRAM_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

/// Creates a new NFT collection using Metaplex Core.
impl<'info> CreateCollection<'info> {
    pub fn create_collection(&mut self, bumps: CreateCollectionBumps) -> Result<()> {
        msg!("Starting create_collection for collection: {}", self.collection.key());
        msg!("MPL Core Program ID: {}", MPL_CORE_PROGRAM_ID);
        let seeds = [b"authority".as_ref(), &[bumps.collection_authority]];
        let signer_seeds = &[&seeds[..]];

        msg!("Invoking CreateCollectionV1CpiBuilder");
        CreateCollectionV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.payer.to_account_info())
            .update_authority(Some(&self.collection_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name("Puzzle NFT Collection".to_string())
            .uri("https://example.com/collection-metadata.json".to_string())
            .invoke_signed(signer_seeds)?;
        msg!("CreateCollectionV1CpiBuilder succeeded");

        msg!("Puzzle NFT Collection created successfully!");
        Ok(())
    }
}