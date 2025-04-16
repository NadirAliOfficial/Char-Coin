// const anchor = require("@project-serum/anchor");
// const fs = require("fs");
// const { PublicKey, Keypair, SystemProgram } = anchor.web3;

// // === Provider and Program Setup ===

// const walletKey = JSON.parse(fs.readFileSync("wallet.json"));
// const wallet = Keypair.fromSecretKey(Uint8Array.from(walletKey));

// const connection = new anchor.web3.Connection("https://api.devnet.solana.com", "confirmed");
// const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), {});
// anchor.setProvider(provider);

// const idl = JSON.parse(fs.readFileSync("idl.json"));
// const programId = new PublicKey(idl.metadata?.address || "REPLACE_WITH_YOUR_PROGRAM_ID");
// const program = new anchor.Program(idl, programId, provider);




const anchor = require("@coral-xyz/anchor");
const { Connection, PublicKey, Keypair, SystemProgram } = require("@solana/web3.js");
const { program, connection, wallet, provider } = require("./config");



// // === Function: Register Charity ===
async function registerCharity({
  id,
  name,
  description,
  charityWallet,
  registrarKeypair = wallet, // default: main wallet
  startTime,
  endTime,
}) {

  const charityKeypair = anchor.web3.Keypair.generate();

  const tx = await program.methods
    .registerCharityHandler(
      new anchor.BN(id),
      name,
      description,
      new PublicKey(charityWallet),
      new anchor.BN(startTime),
      new anchor.BN(endTime)
    )
    .accounts({
      charity: charityKeypair.publicKey,
      registrar: registrarKeypair.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([registrarKeypair, charityKeypair])
    .rpc();

  return {
    tx,
    charityPubkey: charityKeypair.publicKey,
  };
}



async function castVote({
  charityPubkey,
  voterKeypair,
  voteWeight,
}) {
  const [voteRecordPDA] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("vote"),
      new PublicKey(charityPubkey).toBuffer(),
      voterKeypair.publicKey.toBuffer(),
    ],
    program.programId
  );

  const tx = await program.methods
    .castVoteHandler(new anchor.BN(voteWeight))
    .accounts({
      charity: new PublicKey(charityPubkey),
      voteRecord: voteRecordPDA,
      voter: voterKeypair.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([voterKeypair])
    .rpc();

  return {
    tx,
    voteRecordPDA,
  };
}


async function finalizeCharityVote({
  charityPubkey,
  adminKeypair = wallet,
}) {
  const tx = await program.methods
    .finalizeCharityVoteHandler()
    .accounts({
      charity: new PublicKey(charityPubkey),
      admin: adminKeypair.publicKey,
    })
    .signers([adminKeypair])
    .rpc();

  return {
    tx,
  };
}



module.exports = {
  registerCharity,
  castVote,
  finalizeCharityVote,
};
