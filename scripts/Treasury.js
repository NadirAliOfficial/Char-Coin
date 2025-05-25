const anchor = require("@coral-xyz/anchor");
const { Connection, PublicKey, Keypair, SystemProgram } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } = require('@solana/spl-token');
const { program, connection, wallet } = require("./config");


// Treasury initialization function
async function initializeTreasury() 
{


  const treasury = Keypair.generate();

  const _accounts = {
    treasury: treasury.publicKey,
    signer: wallet.publicKey,
    systemProgram: SystemProgram.programId
  }

  try {
      const tx = await program.methods
      .initializeTreasuryHandler([wallet.publicKey, treasury.publicKey], new anchor.BN(2))
      .accounts(_accounts)
      .signers([treasury, wallet])
      .rpc();
      console.log("Transaction Signature : ", tx);
  } catch (error) {
    console.error("Error initializing treasury:", error);
    throw error;
  }
}




async function createWithdrawal(treasuryPubkey, amount) 
  {

  const withdrawal = Keypair.generate();
  const _accounts = {
    treasury: treasuryPubkey,
    withdrawal: withdrawal.publicKey,
    signer: wallet.publicKey,
    systemProgram: SystemProgram.programId
  };

  try {
    const tx = await program.methods
      .createWithdrawalHandler(amount, withdrawal.publicKey)
      .accounts(_accounts)
      .signers([withdrawal, wallet])
      .rpc();
      console.log("withdrawalPubkey: ", withdrawal.publicKey);
      console.log("Transaction Signature : ", tx); 

  } catch (error) {
    console.error("Error creating withdrawal proposal:", error);
    throw error;
  }
}


async function approveWithdrawal(treasuryPubkey, withdrawalPubkey) 
  {
  const _accounts = {
    treasury: treasuryPubkey,
    withdrawal: withdrawalPubkey,
    signer: wallet.publicKey,
  };

  try {
    const tx = await program.methods
      .approveWithdrawalHandler()
      .accounts(_accounts)
      .signers([wallet])
      .rpc();
      console.log("Transaction Signature : ", tx); 

  } catch (error) {
    console.error("Error creating withdrawal proposal:", error);
    throw error;
  }
}



async function executeWithdrawal(treasuryPubkey, withdrawalPubkey) 
  {
  const _accounts = {
    treasury: treasuryPubkey,
    withdrawal: withdrawalPubkey,
    recipient: wallet.publicKey,
  };

  console.log(_accounts);

  try {
    const tx = await program.methods
      .executeWithdrawalHandler()
      .accounts(_accounts)
      .signers([wallet])
      .rpc();
      console.log("Transaction Signature : ", tx); 

  } catch (error) {
    console.error("Error creating withdrawal proposal:", error);
    throw error;
  }
}


// // Approve withdrawal function
// async function approveWithdrawal(
//   connection: Connection,
//   wallet: Wallet,
//   treasuryPubkey: PublicKey,
//   withdrawalPubkey: PublicKey
// ) {
//   const provider = new AnchorProvider(connection, wallet, {});
//   anchor.setProvider(provider);

//   const program = new Program(IDL, PROGRAM_ID, provider);

//   try {
//     const tx = await program.methods
//       .approveWithdrawal()
//       .accounts({
//         treasury: treasuryPubkey,
//         withdrawal: withdrawalPubkey,
//         signer: wallet.publicKey
//       })
//       .rpc();

//     return {
//       transactionSignature: tx
//     };
//   } catch (error) {
//     console.error("Error approving withdrawal:", error);
//     throw error;
//   }
// }

// // Execute withdrawal function
// async function executeWithdrawal(
//   connection: Connection,
//   wallet: Wallet,
//   treasuryPubkey: PublicKey,
//   withdrawalPubkey: PublicKey,
//   recipient: PublicKey
// ) {
//   const provider = new AnchorProvider(connection, wallet, {});
//   anchor.setProvider(provider);

//   const program = new Program(IDL, PROGRAM_ID, provider);

//   try {
//     const tx = await program.methods
//       .executeWithdrawal()
//       .accounts({
//         treasury: treasuryPubkey,
//         withdrawal: withdrawalPubkey,
//         recipient: recipient
//       })
//       .rpc();

//     return {
//       transactionSignature: tx
//     };
//   } catch (error) {
//     console.error("Error executing withdrawal:", error);
//     throw error;
//   }
// }

// // Example usage
// async function main() {
//   const connection = new Connection("YOUR_SOLANA_RPC_ENDPOINT", "confirmed");
//   const wallet = new Wallet(Keypair.generate()); // Replace with your actual wallet

//   // Example: Initialize Treasury
//   const owners = [
//     wallet.publicKey, 
//     Keypair.generate().publicKey, 
//     Keypair.generate().publicKey
//   ];
//   const threshold = 2;

//   const { treasuryPubkey } = await initializeTreasury(
//     connection, 
//     wallet, 
//     owners, 
//     threshold
//   );

//   // Example: Create Withdrawal
//   const recipient = Keypair.generate().publicKey;
//   const { withdrawalPubkey } = await createWithdrawal(
//     connection, 
//     wallet, 
//     treasuryPubkey, 
//     new anchor.BN(1000000), // 0.001 SOL 
//     recipient
//   );

//   // Example: Approve Withdrawal
//   await approveWithdrawal(
//     connection, 
//     wallet, 
//     treasuryPubkey, 
//     withdrawalPubkey
//   );

//   // Example: Execute Withdrawal
//   await executeWithdrawal(
//     connection, 
//     wallet, 
//     treasuryPubkey, 
//     withdrawalPubkey, 
//     recipient
//   );
// }



module.exports = {
  initializeTreasury,
  createWithdrawal,
  approveWithdrawal,
  executeWithdrawal
};