import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, Wallet } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair, SystemProgram } from "@solana/web3.js";

// This should be replaced with your actual program ID
const PROGRAM_ID = new PublicKey("YOUR_PROGRAM_ID_HERE");

// Treasury initialization function
async function initializeTreasury(
  connection: Connection, 
  wallet: Wallet, 
  owners: PublicKey[], 
  threshold: number
) {
  const provider = new AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);

  const program = new Program(IDL, PROGRAM_ID, provider);

  // Generate a new keypair for the treasury account
  const treasury = Keypair.generate();

  try {
    const tx = await program.methods
      .initializeTreasury(owners, threshold)
      .accounts({
        treasury: treasury.publicKey,
        signer: wallet.publicKey,
        systemProgram: SystemProgram.programId
      })
      .signers([treasury, wallet.payer])
      .rpc();

    return {
      treasuryPubkey: treasury.publicKey,
      transactionSignature: tx
    };
  } catch (error) {
    console.error("Error initializing treasury:", error);
    throw error;
  }
}

// Create withdrawal proposal function
async function createWithdrawal(
  connection: Connection,
  wallet: Wallet,
  treasuryPubkey: PublicKey,
  amount: anchor.BN,
  recipient: PublicKey
) {
  const provider = new AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);

  const program = new Program(IDL, PROGRAM_ID, provider);

  // Generate a new keypair for the withdrawal proposal
  const withdrawal = Keypair.generate();

  try {
    const tx = await program.methods
      .createWithdrawal(amount, recipient)
      .accounts({
        treasury: treasuryPubkey,
        withdrawal: withdrawal.publicKey,
        signer: wallet.publicKey,
        systemProgram: SystemProgram.programId
      })
      .signers([withdrawal, wallet.payer])
      .rpc();

    return {
      withdrawalPubkey: withdrawal.publicKey,
      transactionSignature: tx
    };
  } catch (error) {
    console.error("Error creating withdrawal proposal:", error);
    throw error;
  }
}

// Approve withdrawal function
async function approveWithdrawal(
  connection: Connection,
  wallet: Wallet,
  treasuryPubkey: PublicKey,
  withdrawalPubkey: PublicKey
) {
  const provider = new AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);

  const program = new Program(IDL, PROGRAM_ID, provider);

  try {
    const tx = await program.methods
      .approveWithdrawal()
      .accounts({
        treasury: treasuryPubkey,
        withdrawal: withdrawalPubkey,
        signer: wallet.publicKey
      })
      .rpc();

    return {
      transactionSignature: tx
    };
  } catch (error) {
    console.error("Error approving withdrawal:", error);
    throw error;
  }
}

// Execute withdrawal function
async function executeWithdrawal(
  connection: Connection,
  wallet: Wallet,
  treasuryPubkey: PublicKey,
  withdrawalPubkey: PublicKey,
  recipient: PublicKey
) {
  const provider = new AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);

  const program = new Program(IDL, PROGRAM_ID, provider);

  try {
    const tx = await program.methods
      .executeWithdrawal()
      .accounts({
        treasury: treasuryPubkey,
        withdrawal: withdrawalPubkey,
        recipient: recipient
      })
      .rpc();

    return {
      transactionSignature: tx
    };
  } catch (error) {
    console.error("Error executing withdrawal:", error);
    throw error;
  }
}

// Example usage
async function main() {
  const connection = new Connection("YOUR_SOLANA_RPC_ENDPOINT", "confirmed");
  const wallet = new Wallet(Keypair.generate()); // Replace with your actual wallet

  // Example: Initialize Treasury
  const owners = [
    wallet.publicKey, 
    Keypair.generate().publicKey, 
    Keypair.generate().publicKey
  ];
  const threshold = 2;

  const { treasuryPubkey } = await initializeTreasury(
    connection, 
    wallet, 
    owners, 
    threshold
  );

  // Example: Create Withdrawal
  const recipient = Keypair.generate().publicKey;
  const { withdrawalPubkey } = await createWithdrawal(
    connection, 
    wallet, 
    treasuryPubkey, 
    new anchor.BN(1000000), // 0.001 SOL 
    recipient
  );

  // Example: Approve Withdrawal
  await approveWithdrawal(
    connection, 
    wallet, 
    treasuryPubkey, 
    withdrawalPubkey
  );

  // Example: Execute Withdrawal
  await executeWithdrawal(
    connection, 
    wallet, 
    treasuryPubkey, 
    withdrawalPubkey, 
    recipient
  );
}

// Note: You'll need to replace this with your actual IDL
const IDL = {
  // Your program's IDL goes here
};

main().catch(console.error);

export {
  initializeTreasury,
  createWithdrawal,
  approveWithdrawal,
  executeWithdrawal
};