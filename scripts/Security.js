const anchor = require("@coral-xyz/anchor");
const { Connection, PublicKey, Keypair, SystemProgram } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } = require('@solana/spl-token');
const { program, wallet } = require("./config");


async function initializeMultisigHandler() 
{

    const multisig = Keypair.generate(); 

const owners = [
    wallet.publicKey, // you can add more test public keys here
    Keypair.generate().publicKey,
    Keypair.generate().publicKey,
  ];

  const threshold = 2;
  const walletType = { marketing: {} }; 
  const _accounts = {
    multisig: multisig.publicKey,
    payer: wallet.publicKey,
    systemProgram: SystemProgram.programId,
}

  try {
      const tx = await program.methods
      .initializeMultisigHandler({
        owners,
        threshold,
        walletType,
    })
      .accounts(_accounts)
      .signers([multisig])
      .rpc();
      console.log("Transaction Signature : ", tx);
  } catch (error) {
    console.error("Error initializing security:", error);
    throw error;
  }

  return { multisig, owners };

}



async function callVerifyMultisig(multisigPubkey, signerKeypairs) {
 const tx = await program.methods
    .verifyMultisigHandler()
    .accounts({
      multisig: multisigPubkey.publicKey,
      signer1: signerKeypairs[0].publicKey,
      signer2: signerKeypairs[1].publicKey,
      signer3: signerKeypairs[2].publicKey,    
    })
    .signers([wallet])
    .rpc();

  console.log("✅ Verified successfully ", tx);
  console.log("✅ verify_multisig executed on", multisigPubkey.publicKey);
}


async function initializeEmergencyStateHandler() 
{
  const emergencyState = Keypair.generate(); 
  const tx = await program.methods
    .initializeEmergencyStateHandler(false)
    .accounts({
      emergencyState:emergencyState.publicKey,
      payer: wallet.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([wallet, emergencyState])
    .rpc();
  console.log("Transaction : ", tx);
  console.log("✅ Emergency State initialized successfully.");
  return emergencyState.publicKey;
}



async function emergencyHaltHandler(emergencyStatePubkey) {
const tx = await program.methods
    .emergencyHaltHandler()
    .accounts({
      emergencyState: emergencyStatePubkey,
      authority: wallet.publicKey,
    })
    .signers([wallet])
    .rpc();
  console.log("Transaction : ", tx);
  console.log("⛔ Emergency halt triggered successfully.");
}


async function emergencyUnhaltHandler(emergencyStatePubkey) {
  const tx = await program.methods
      .emergencyUnhaltHandler()
      .accounts({
        emergencyState: emergencyStatePubkey,
        authority: wallet.publicKey,
      })
      .signers([wallet])
      .rpc();
    console.log("Transaction : ", tx);
    console.log("⛔ Emergency unhalt triggered successfully.");
  }


module.exports = {
    initializeMultisigHandler,
    callVerifyMultisig,
    initializeEmergencyStateHandler,
    emergencyHaltHandler,
    emergencyUnhaltHandler
};