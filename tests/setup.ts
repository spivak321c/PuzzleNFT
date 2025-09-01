import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PuzzleNft } from "../target/types/puzzle_nft";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { mplCore } from "@metaplex-foundation/mpl-core";
import { publicKey as umiPublicKey, createNoopSigner, signerIdentity } from "@metaplex-foundation/umi";
import { web3JsRpc } from "@metaplex-foundation/umi-rpc-web3js";


export interface TestContext {
  provider: anchor.AnchorProvider;
  program: Program<PuzzleNft>;
  umi: any;
  walletSigner: any;
}

export interface TestAccounts {
  collectionKp: anchor.web3.Keypair;
  assetKp: anchor.web3.Keypair;
  nonOwnerKp: anchor.web3.Keypair;
  collectionAuthority: anchor.web3.PublicKey;
  authority: anchor.web3.PublicKey;
}

export const MPL_CORE_PROGRAM_ID = new anchor.web3.PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

export function setupTestContext(): TestContext {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  
  const program = anchor.workspace.PuzzleNft as Program<PuzzleNft>;
  //const umi = createUmi(provider.connection.rpcEndpoint).use(mplCore());
    const umi = createUmi(provider.connection.rpcEndpoint).use(web3JsRpc(provider.connection)).use(mplCore());

  
  const walletSigner = createNoopSigner(umiPublicKey(provider.wallet.publicKey.toString()));
  umi.use(signerIdentity(walletSigner));
  
  return { provider, program, umi, walletSigner };
}

export function generateTestAccounts(program: Program<PuzzleNft>): TestAccounts {
  const collectionKp = anchor.web3.Keypair.generate();
  const assetKp = anchor.web3.Keypair.generate();
  const nonOwnerKp = anchor.web3.Keypair.generate();

  // Fixed: Use general authority seeds as per the updated smart contract
  const [collectionAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("authority")],
    program.programId
  );

  // Fixed: Use general authority seeds instead of collection-specific
  const [authority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("authority")],
    program.programId
  );

  return {
    collectionKp,
    assetKp,
    nonOwnerKp,
    collectionAuthority,
    authority
  };
}

export async function airdropSol(connection: anchor.web3.Connection, publicKey: anchor.web3.PublicKey, amount: number) {
  const signature = await connection.requestAirdrop(publicKey, amount * anchor.web3.LAMPORTS_PER_SOL);
  await connection.confirmTransaction(signature);
}

export function expectError(error: any, expectedErrorCode: string) {
  if (error.message && error.message.includes(expectedErrorCode)) {
    return true;
  }
  if (error.error && error.error.errorCode && error.error.errorCode.code === expectedErrorCode) {
    return true;
  }
  return false;
}