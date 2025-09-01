import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { fetchAsset } from "@metaplex-foundation/mpl-core";
import { publicKey as umiPublicKey } from "@metaplex-foundation/umi";
import { setupTestContext, generateTestAccounts, MPL_CORE_PROGRAM_ID } from "./setup";

describe("Minting Tests", () => {
  const { provider, program, umi } = setupTestContext();
  let accounts: ReturnType<typeof generateTestAccounts>;

  beforeEach(async () => {
    accounts = generateTestAccounts(program);
    
    // Create collection first
    await program.methods
      .createCollection()
      .accounts({
        collection: accounts.collectionKp.publicKey,
        payer: provider.wallet.publicKey,
        collectionAuthority: accounts.collectionAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([accounts.collectionKp])
      .rpc();
  });

  describe("Successful Minting", () => {
    it("Should mint a math_factor puzzle NFT", async () => {
      const name = "Math Factor Puzzle";
      const uri = "https://example.com/math-puzzle.json";
      const puzzleType = 0; // math_factor
      const difficulty = 1;
      const assetKp = anchor.web3.Keypair.generate();

      const tx = await program.methods
        .mintPuzzleNft(name, uri, puzzleType, difficulty)
        .accounts({
          asset: assetKp.publicKey,
          collection: accounts.collectionKp.publicKey,
          payer: provider.wallet.publicKey,
          authority: accounts.authority,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
        })
        .signers([assetKp])
        .rpc();

      expect(tx).to.be.ok;
      console.log("Math puzzle NFT minted:", assetKp.publicKey.toString());

      // Verify asset properties
      const asset = await fetchAsset(umi, umiPublicKey(assetKp.publicKey.toString()));
      expect(asset.name).to.equal(name);
      expect(asset.uri).to.equal(uri);
      expect(asset.owner).to.equal(umiPublicKey(provider.wallet.publicKey.toString()));

      // Verify puzzle attributes
      const attributes = asset.attributes?.attributeList || [];
      expect(attributes.find(attr => attr.key === "puzzle_type")?.value).to.equal("math_factor");
      expect(attributes.find(attr => attr.key === "difficulty")?.value).to.equal("1");
      expect(attributes.find(attr => attr.key === "solved")?.value).to.equal("false");
      expect(attributes.find(attr => attr.key === "puzzle_number")).to.be.ok;
      expect(attributes.find(attr => attr.key === "solution_hash")).to.be.ok;
      expect(attributes.find(attr => attr.key === "mint_slot")).to.be.ok;
    });

    it("Should mint a hash_riddle puzzle NFT", async () => {
      const name = "Hash Riddle Puzzle";
      const uri = "https://example.com/hash-puzzle.json";
      const puzzleType = 1; // hash_riddle
      const difficulty = 2;
      const assetKp = anchor.web3.Keypair.generate();

      const tx = await program.methods
        .mintPuzzleNft(name, uri, puzzleType, difficulty)
        .accounts({
          asset: assetKp.publicKey,
          collection: accounts.collectionKp.publicKey,
          payer: provider.wallet.publicKey,
          authority: accounts.authority,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
        })
        .signers([assetKp])
        .rpc();

      expect(tx).to.be.ok;

      const asset = await fetchAsset(umi, umiPublicKey(assetKp.publicKey.toString()));
      const attributes = asset.attributes?.attributeList || [];
      expect(attributes.find(attr => attr.key === "puzzle_type")?.value).to.equal("hash_riddle");
      expect(attributes.find(attr => attr.key === "difficulty")?.value).to.equal("2");
    });

    it("Should mint a pattern puzzle NFT", async () => {
      const name = "Pattern Puzzle";
      const uri = "https://example.com/pattern-puzzle.json";
      const puzzleType = 2; // pattern
      const difficulty = 3;
      const assetKp = anchor.web3.Keypair.generate();

      const tx = await program.methods
        .mintPuzzleNft(name, uri, puzzleType, difficulty)
        .accounts({
          asset: assetKp.publicKey,
          collection: accounts.collectionKp.publicKey,
          payer: provider.wallet.publicKey,
          authority: accounts.authority,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
        })
        .signers([assetKp])
        .rpc();

      expect(tx).to.be.ok;

      const asset = await fetchAsset(umi, umiPublicKey(assetKp.publicKey.toString()));
      const attributes = asset.attributes?.attributeList || [];
      expect(attributes.find(attr => attr.key === "puzzle_type")?.value).to.equal("pattern");
      expect(attributes.find(attr => attr.key === "difficulty")?.value).to.equal("3");
    });
  });

  describe("Minting Validation", () => {
    it("Should fail with invalid puzzle type", async () => {
      const name = "Invalid Puzzle";
      const uri = "https://example.com/invalid.json";
      const puzzleType = 99; // Invalid
      const difficulty = 1;
      const assetKp = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .mintPuzzleNft(name, uri, puzzleType, difficulty)
          .accounts({
            asset: assetKp.publicKey,
            collection: accounts.collectionKp.publicKey,
            payer: provider.wallet.publicKey,
            authority: accounts.authority,
            systemProgram: anchor.web3.SystemProgram.programId,
            sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([assetKp])
          .rpc();

        expect.fail("Should have failed with InvalidPuzzleType error");
      } catch (error) {
        expect(error.message).to.include("InvalidPuzzleType");
        console.log("Successfully caught invalid puzzle type error");
      }
    });

    it("Should fail with wrong authority", async () => {
      const name = "Test Puzzle";
      const uri = "https://example.com/test.json";
      const puzzleType = 0;
      const difficulty = 1;
      const assetKp = anchor.web3.Keypair.generate();
      const wrongAuthority = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .mintPuzzleNft(name, uri, puzzleType, difficulty)
          .accounts({
            asset: assetKp.publicKey,
            collection: accounts.collectionKp.publicKey,
            payer: provider.wallet.publicKey,
            authority: wrongAuthority.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([assetKp])
          .rpc();

        expect.fail("Should have failed with constraint error");
      } catch (error) {
        expect(error.message).to.include("constraint");
        console.log("Successfully caught wrong authority error");
      }
    });

    it("Should fail without collection", async () => {
      const name = "No Collection Puzzle";
      const uri = "https://example.com/no-collection.json";
      const puzzleType = 0;
      const difficulty = 1;
      const assetKp = anchor.web3.Keypair.generate();
      const fakeCollection = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .mintPuzzleNft(name, uri, puzzleType, difficulty)
          .accounts({
            asset: assetKp.publicKey,
            collection: fakeCollection.publicKey,
            payer: provider.wallet.publicKey,
            authority: accounts.authority,
            systemProgram: anchor.web3.SystemProgram.programId,
            sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([assetKp])
          .rpc();

        expect.fail("Should have failed with invalid collection");
      } catch (error) {
        expect(error).to.be.ok;
        console.log("Successfully caught invalid collection error");
      }
    });
  });

  describe("Puzzle Generation", () => {
    it("Should generate unique puzzle numbers", async () => {
      const puzzleNumbers: string[] = [];
      
      for (let i = 0; i < 3; i++) {
        const assetKp = anchor.web3.Keypair.generate();
        
        await program.methods
          .mintPuzzleNft(`Puzzle ${i}`, `https://example.com/${i}.json`, 0, 1)
          .accounts({
            asset: assetKp.publicKey,
            collection: accounts.collectionKp.publicKey,
            payer: provider.wallet.publicKey,
            authority: accounts.authority,
            systemProgram: anchor.web3.SystemProgram.programId,
            sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([assetKp])
          .rpc();

        const asset = await fetchAsset(umi, umiPublicKey(assetKp.publicKey.toString()));
        const attributes = asset.attributes?.attributeList || [];
        const puzzleNumber = attributes.find(attr => attr.key === "puzzle_number")?.value;
        
        expect(puzzleNumber).to.be.ok;
        expect(puzzleNumbers).to.not.include(puzzleNumber!);
        puzzleNumbers.push(puzzleNumber!);
        
        // Small delay to ensure different slots
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    });

    it("Should generate different puzzle numbers for different difficulties", async () => {
      const assetKp1 = anchor.web3.Keypair.generate();
      const assetKp2 = anchor.web3.Keypair.generate();
      
      await program.methods
        .mintPuzzleNft("Easy Puzzle", "https://example.com/easy.json", 0, 1)
        .accounts({
          asset: assetKp1.publicKey,
          collection: accounts.collectionKp.publicKey,
          payer: provider.wallet.publicKey,
          authority: accounts.authority,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
        })
        .signers([assetKp1])
        .rpc();

      await program.methods
        .mintPuzzleNft("Hard Puzzle", "https://example.com/hard.json", 0, 5)
        .accounts({
          asset: assetKp2.publicKey,
          collection: accounts.collectionKp.publicKey,
          payer: provider.wallet.publicKey,
          authority: accounts.authority,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
        })
        .signers([assetKp2])
        .rpc();

      const asset1 = await fetchAsset(umi, umiPublicKey(assetKp1.publicKey.toString()));
      const asset2 = await fetchAsset(umi, umiPublicKey(assetKp2.publicKey.toString()));
      
      const puzzleNumber1 = asset1.attributes?.attributeList.find(attr => attr.key === "puzzle_number")?.value;
      const puzzleNumber2 = asset2.attributes?.attributeList.find(attr => attr.key === "puzzle_number")?.value;
      
      expect(puzzleNumber1).to.not.equal(puzzleNumber2);
    });
  });
});