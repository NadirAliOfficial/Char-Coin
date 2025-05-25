const anchor = require("@project-serum/anchor");
const fs = require("fs");
const { Connection, Keypair, PublicKey, SystemProgram } = require("@solana/web3.js");

const PROGRAM_ID = new PublicKey("F48XbbJCfQm2ie2gZMKey3d7Ldz9Y9MJiyNsuyBqftyN");

// Load wallet keypair
const walletKeypair = anchor.web3.Keypair.fromSecretKey(
  new Uint8Array(JSON.parse(fs.readFileSync("wallet.json", "utf8")))
);

// Load IDL
const idl = JSON.parse(fs.readFileSync("idl.json", "utf8"));

// Setup Solana connection
const connection = new anchor.web3.Connection(
  anchor.web3.clusterApiUrl("devnet"), // Change to 'mainnet-beta' or 'testnet' if needed
  "processed"
);

// Create provider
const provider = new anchor.AnchorProvider(
  connection,
  new anchor.Wallet(walletKeypair),
  { preflightCommitment: "processed" }
);



// Create program instance
const program = new anchor.Program(idl, PROGRAM_ID, provider);

async function stakeTokens(amount, lockupPeriod) {
  try {
    // Define staking pool and stake info accounts (Replace with actual PDA generation logic)
    const [stakingPoolPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("staking_pool")],
      program.programId
    );

    const stakeInfoKeypair = anchor.web3.Keypair.generate(); // Generating a new stake info account

    // Convert lockup period to enum index
    const lockupEnum = {
      "ThirtyDays": { thirtyDays: {} },
      "NinetyDays": { ninetyDays: {} },
      "OneEightyDays": { oneEightyDays: {} }
    };

    if (!lockupEnum[lockupPeriod]) {
      throw new Error("Invalid lockup period. Use 'ThirtyDays', 'NinetyDays', or 'OneEightyDays'.");
    }

    console.log("Staking tokens...");

    const tx = await program.methods
      .stakeTokensHandler(new anchor.BN(amount), lockupEnum[lockupPeriod])
      .accounts({
        stakingPool: stakingPoolPDA,
        stakeInfo: stakeInfoKeypair.publicKey,
        staker: walletKeypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([walletKeypair, stakeInfoKeypair])
      .rpc();

    console.log("Transaction successful! Signature:", tx);
  } catch (error) {
    console.error("Error staking tokens:", error);
  }
}

// Example usage:
stakeTokens(100, "NinetyDays"); // Stake 1000 tokens for 90 days
