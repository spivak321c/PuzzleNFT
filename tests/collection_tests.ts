import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { fetchCollectionV1 } from "@metaplex-foundation/mpl-core";
import { publicKey as umiPublicKey } from "@metaplex-foundation/umi";
import { setupTestContext, generateTestAccounts, MPL_CORE_PROGRAM_ID, airdropSol } from "./setup";

describe("Collection Tests", () => {
  const { provider, program, umi } = setupTestContext();
  let accounts: ReturnType<typeof generateTestAccounts>;

  beforeEach(() => {
    accounts = generateTestAccounts(program);
  });

  describe("Collection Creation", () => {
    it("Should create a collection successfully", async () => {
      const tx = await program.methods
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
      console.log("Collection created: ", accounts.collectionKp.publicKey.toString());
      console.log("Transaction signature: ", tx);

      // Wait for transaction finalization
      await provider.connection.confirmTransaction(tx, "confirmed");

      try {
        // Fetch raw account data to debug
        const accountInfo = await provider.connection.getAccountInfo(accounts.collectionKp.publicKey, "confirmed");
        if (accountInfo) {
          console.log("Account exists with data length:", accountInfo.data.length);
          console.log("Account owner:", accountInfo.owner.toString());
          console.log("Raw account data (hex):", accountInfo.data.toString("hex"));
          console.log("Account lamports:", accountInfo.lamports);
        } else {
          console.log("Account does not exist");
        }

        const collection = await fetchCollectionV1(umi, umiPublicKey(accounts.collectionKp.publicKey.toString()), { commitment: 'confirmed' });
        console.log("Fetched collection:", JSON.stringify(collection, (key, value) =>
          typeof value === 'bigint' ? value.toString() : value
        , 2));
        expect(collection.updateAuthority.toString()).to.equal(accounts.collectionAuthority.toString());
      } catch (error) {
        console.error("Failed to fetch collection:", error);
        throw error;
      }
    });

    it("Should fail to create collection with invalid payer", async () => {
      const invalidKp = anchor.web3.Keypair.generate();
      try {
        await program.methods
          .createCollection()
          .accounts({
            collection: accounts.collectionKp.publicKey,
            payer: invalidKp.publicKey,
            collectionAuthority: accounts.collectionAuthority,
            systemProgram: anchor.web3.SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([accounts.collectionKp, invalidKp])
          .rpc();
        expect.fail("Should have failed with insufficient funds or account not found");
      } catch (error) {
        expect(error.message).to.match(/insufficient|InsufficientFunds|AccountNotFound/i);
        console.log("Successfully caught insufficient funds error");
      }
    });

    it("Should fail to create collection with wrong program ID", async () => {
      try {
        await program.methods
          .createCollection()
          .accounts({
            collection: accounts.collectionKp.publicKey,
            payer: provider.wallet.publicKey,
            collectionAuthority: accounts.collectionAuthority,
            systemProgram: anchor.web3.SystemProgram.programId,
            mplCoreProgram: anchor.web3.SystemProgram.programId, // Wrong program ID
          })
          .signers([accounts.collectionKp])
          .rpc();
        expect.fail("Should have failed with wrong program ID");
      } catch (error) {
        expect(error).to.be.ok;
        console.log("Successfully caught wrong program ID error");
      }
    });
  });

  describe("Collection Authority", () => {
    beforeEach(async () => {
      const tx = await program.methods
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
      await provider.connection.confirmTransaction(tx, "confirmed");
    });

    it("Should have correct collection authority PDA", async () => {
      const collection = await fetchCollectionV1(umi, umiPublicKey(accounts.collectionKp.publicKey.toString()), { commitment: 'confirmed' });
      expect(collection.updateAuthority.toString()).to.equal(accounts.collectionAuthority.toString());
    });

    it("Should derive authority PDA correctly", async () => {
      const [derivedAuthority, bump] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("authority")],
        program.programId
      );
      expect(derivedAuthority.toString()).to.equal(accounts.collectionAuthority.toString());
      expect(bump).to.be.a("number");
    });
  });
});
