# Puzzle NFT Smart Contract for Solana

## Project Overview

`Puzzle NFT` is a Solana program built with [Anchor](https://www.anchor-lang.com/) and [Metaplex Core](https://github.com/metaplex-foundation/mpl-core) to create and manage Non-Fungible Tokens (NFTs) with embedded puzzles. Users can create a collection, mint NFTs with puzzles (math_factor, hash_riddle, or pattern), and solve them to update attributes and earn rarity scores. The program uses Anchor for Rust-based smart contracts and TypeScript for client-side testing, leveraging Metaplex Core for NFT functionality. Comprehensive TypeScript unit tests ensure functionality and robustness, making the codebase reusable for Solana-based NFT projects.

### Features
- **Collection Creation**: Initializes a Metaplex collection for grouping Puzzle NFTs.
- **NFT Minting**: Mints NFTs with custom attributes (puzzle type, difficulty, puzzle number, solution hash, etc.) stored in Metaplex's Attributes plugin.
- **Puzzle Solving**: Allows NFT owners to submit solutions, updating the NFT's attributes (e.g., marking as solved, assigning rarity) and optionally updating the metadata URI.
- **Creative Features**:
  - **Puzzle Attributes**: Embeds puzzle-related data (type, difficulty, solution hash) as on-chain attributes, enabling dynamic gameplay.
  - **Rarity System**: Assigns rarity (Legendary, Epic, Rare, Common) based on solve time, enhancing NFT value.
  - **Event Emission**: Emits `PuzzleMinted` and `PuzzleSolved` events for transparency and off-chain tracking.
- **Unit Tests**: Comprehensive TypeScript tests verify collection creation, NFT minting, and puzzle solving, including attribute validation and edge cases.
- **Error Handling**: Robust error codes (e.g., `IncorrectSolution`, `NotNftOwner`, `AlreadySolved`) ensure secure operations.

The program is designed to be secure, extensible, and reusable, serving as a template for gamified NFT projects on Solana.

## Setup Instructions

### Prerequisites
- A Solana wallet with a public address for deployment and testing ( Localnet recommended).
- Internet connection for downloading dependencies.

### Installation
1. **Clone the Repository**:
   ```bash
   git clone https://github.com/your-username/puzzlenft1.git
   cd puzzlenft1
   ```

2. **Run Setup Script**:
   - For **Linux** (including WSL on Windows) or **macOS**:
     ```bash
     chmod +x setup.sh start-validator.sh
     ./setup.sh
     ```
   - For **Windows**:
     - Install WSL (Windows Subsystem for Linux) by running `wsl --install` in PowerShell, then use Ubuntu.
     - In the Ubuntu terminal, run the commands above.
   - The `setup.sh` script (if provided) installs Rust, Solana CLI, Anchor CLI (version 0.31.1), Node.js, and Yarn, configuring the environment for Solana development.

3. **Set Up Solana Environment**:
   - Configure Solana CLI for Devnet:
     ```bash
     solana config set --url localhost
     ```
   - Generate or use an existing keypair:
     ```bash
     solana-keygen new
     ```

4. **Install Dependencies**:
   - Install Node.js dependencies for testing:
     ```bash
     yarn install
     ```

5. **Build the Program**:
   - Compile the Rust program:
     ```bash
     anchor build
     ```
   - The output will be in `target/deploy/puzzle_nft.so`.

6. **Run Unit Tests**:
   - Run the Solana validator script (optional for Devnet):
     ```bash
     ./start-validator.sh
     ```
   Execute TypeScript tests open a new terminal and Run the test suite to verify functionality:
    ```bash
   anchor test --skip-local-validator
    ```
   - Tests verify collection creation, NFT minting, and puzzle solving, including attribute validation.

7. **Deploy to Devnet**:
   - Deploy the program:
     ```bash
     solana program deploy target/deploy/puzzle_nft.so
     ```
   - Note the program ID for use in client applications or further testing.

## Project Structure
- `Anchor.toml`: Anchor configuration file.
- `Cargo.toml` & `Cargo.lock`: Rust dependency management for the smart contract.
- `programs/puzzle_nft/`: Rust source code for the Puzzle NFT program.
  - `src/lib.rs`: Main program entry point defining the instruction handlers.
  - `src/instructions/`: Instruction logic for creating collections, minting NFTs, and solving puzzles.
  - `Xargo.toml`: Configuration for cross-compilation to Solana's BPF target.
- `tests/`: TypeScript unit tests for all program functionality.
- `tsconfig.json` & `package.json`: Configuration for TypeScript and Node.js dependencies.


## Edge Case Testing
The TypeScript unit tests in `tests/puzzle_nft.ts` cover the following edge cases:
- **Collection Creation**:
  - Non-signer attempting to create a collection.
  - Invalid collection authority.
- **NFT Minting**:
  - Invalid puzzle type or difficulty values.
  - Missing or invalid collection.
  - Non-authorized authority signing.
- **Puzzle Solving**:
  - Non-owner attempting to solve the puzzle.
  - Attempting to solve an already solved NFT.
  - Invalid or missing collection in the solve instruction.
  - Verification of updated attributes (solved status, solver, rarity).

These tests ensure the program handles errors securely and maintains data integrity.

## Extra Creative Features
To meet the DevQuest’s imaginativity criterion, the following features were added:
- **Puzzle Attributes**: NFTs store puzzle data (type, difficulty, solution hash) as on-chain attributes, enabling dynamic puzzle-based gameplay.
- **Rarity System**: Assigns rarity based on solve time, adding a gamified element to increase NFT value and engagement.
- **Event Emission**: Emits `PuzzleMinted` and `PuzzleSolved` events for off-chain applications to track minting and solving activities.
- **Flexible URI Updates**: Allows optional URI updates upon solving, enabling dynamic metadata changes (e.g., new artwork for solved puzzles).

These features make the program engaging and adaptable for gamified NFT applications.


## Usage
A frontend client is not included. To interact with the deployed program:
1. Use the program ID from deployment in a TypeScript client or Anchor CLI.
2. Example interactions:
   - **Create Collection**: Initialize a Metaplex collection for Puzzle NFTs.
   - **Mint Puzzle NFT**: Mint an NFT with puzzle attributes (e.g., math_factor, difficulty=1).
   - **Solve Puzzle**: Submit a solution to update the NFT’s attributes and assign rarity.
3. Example client code can be created in `program_client/app.ts` using the Anchor client library.

## Contributing
This program is designed for reuse by the Solana community. To contribute:
- Fork the repository and submit pull requests with enhancements.
- Suggested improvements:
  - Add puzzle validation logic for different puzzle types (e.g., math-based or hash-based).
  - Support for multi-puzzle NFTs with progressive solving.
  - Integration with off-chain puzzle generators for dynamic challenges.

## License
This project is licensed under the MIT License, making it freely reusable for Solana developers and projects.
