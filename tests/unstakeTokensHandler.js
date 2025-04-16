
const { Connection, PublicKey, Keypair } = require('@solana/web3.js');
const anchor = require("@project-serum/anchor");
const { Program, AnchorProvider, web3, BN } = require('@project-serum/anchor');
const { TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } = require('@solana/spl-token');
const { program, connection, provider, wallet, PROGRAM_ID } = require("./config");

anchor.setProvider(provider);



// Function to stake tokens
async function unstakeTokensHandler(tokenMint) {
    try {
        const userTokenAccount = await getTokenAccount(wallet.publicKey, tokenMint);
    // Derive PDA for staking pool
    const [stakingPool, bump] = await PublicKey.findProgramAddress(
        [Buffer.from('staking_pool'), tokenMint.toBuffer()],
        program.programId
      );

      // Create and initialize the pool token account
      const poolTokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        wallet,
        tokenMint,
        stakingPool,
        true // allowOwnerOffCurve: true since PDA is not on curve
      );


        // Derive the user stake account PDA
        const [userStakePDA] = await PublicKey.findProgramAddressSync(
            [Buffer.from("user"), stakingPool.toBuffer(), wallet.publicKey.toBuffer()],
            PROGRAM_ID
        );


        const _accounts = {
            stakingPool: stakingPool,
            user: userStakePDA,
            userAuthority: wallet.publicKey,
            userTokenAccount: userTokenAccount,
            poolTokenAccount: poolTokenAccount.address,
            rewardTokenAccount: poolTokenAccount.address,
            stokenProgram: TOKEN_PROGRAM_ID
        }        


        const tx = await program.methods
        .unstakeTokensHandler()
        .accounts(_accounts)
        .rpc();

        console.log("Staking Transaction Hash : ", tx);

    } catch (error) {
        console.error("Error staking tokens:", error);
    }
}

// Helper function to find user's token account
async function getTokenAccount(owner, mint) {
    const accounts = await connection.getParsedTokenAccountsByOwner(owner, { mint });
    if (accounts.value.length === 0) throw new Error("Token account not found");
    return accounts.value[0].pubkey;
}


module.exports = unstakeTokensHandler;
