
const { Connection, PublicKey, Keypair } = require('@solana/web3.js');
const { Program, AnchorProvider, web3, BN } = require('@project-serum/anchor');
const { TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } = require('@solana/spl-token');
const { program, connection, wallet } = require("./config");

async function initialize(tokenMint) {
  try {
    const [stakingPool, bump] = await PublicKey.findProgramAddress(
      [Buffer.from('staking_pool'), tokenMint.toBuffer()],
      program.programId
    );

    const poolTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet,
      tokenMint,
      stakingPool,
      true 
    );

    console.log('Pool token account:', poolTokenAccount.address.toString());

    const tx = await program.methods
      .stakingInitialize()
      .accounts({
        stakingPool: stakingPool,
        authority: wallet.publicKey,
        tokenMint: tokenMint,
        poolTokenAccount: poolTokenAccount.address,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log('Staking pool initialized! Transaction:', tx);
    
  } catch (error) {
    console.error('Error initializing staking pool:', error);
  }
}

module.exports = initialize;