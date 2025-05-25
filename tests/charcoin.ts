import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Charcoin } from "../target/types/charcoin";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";

async function confirmTransaction(tx: string) {
  const latestBlockHash = await anchor.getProvider().connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction({
    blockhash: latestBlockHash.blockhash,
    lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    signature: tx,
  });
}

async function airdropSol(publicKey: anchor.web3.PublicKey, amount: number) {
  let airdropTx = await anchor.getProvider().connection.requestAirdrop(publicKey, amount);
  await confirmTransaction(airdropTx);
}

describe("char coin test", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const payer = anchor.web3.Keypair.generate()
  const user = anchor.web3.Keypair.generate();
    const configAccount = anchor.web3.Keypair.generate();

  const program = anchor.workspace.charcoin as Program<Charcoin>;

    // Derive monthly reward wallet PDA
  let [monthlyRewardWallet,] =  anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('monthly_reward')],
    program.programId
  );
  
  // Derive annual reward wallet PDA
  let [annualRewardWallet,] =  anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('annual_reward')],
    program.programId
  );
  
  // Derive monthly donation wallet PDA
  let [monthlyDonationWallet,] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('monthly_donation')],
    program.programId
  );
  
  // Derive annual charity wallet PDA
  let [annualCharityWallet,] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('annual_charity')],
    program.programId
  );
  let tokenMint
  let userAta
  before(async () => {
    await airdropSol(payer.publicKey, 20 * 1e9); // 20 SOL
    await airdropSol(user.publicKey, 5 * 1e9);

    tokenMint = await createMint(
      program.provider.connection,
      payer,
      payer.publicKey,
      null,
      6 // decimals
    );


    userAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      user,
      tokenMint,
      user.publicKey
    );
    await mintTo(
      program.provider.connection,
      user,
      tokenMint,
      userAta.address,
      payer,
      1_000_000_00000
    );
  })
  it("initialized", async () => {
    // Add your test here.
    const context = {
        user: payer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configAccount.publicKey,
        mint: tokenMint
    }
     // Define configuration parameters
    const config = {
        tokenSupply: new anchor.BN(1000000000), // Example: 1 billion tokens
        feePercentage: 2,
        buybackPercentage: 5,
        donationPercentage: 3,
        stakingPercentage: 10,
        admin: payer.publicKey,
        mintAuthorityBump: 0, // Adjust as needed
        monthlyRewardWallet:monthlyRewardWallet,
        annualRewardWallet: annualRewardWallet,
        monthlyDonationWallet: monthlyDonationWallet,
        annualCharityWallet: annualCharityWallet,
    };
    const tx = await program.methods.initialize(config)
      .accounts(context)
      .signers([payer,configAccount])
      .rpc();

        
  });


  it("stake initialize", async () => {

   const [stakingPool, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('staking_pool'), tokenMint.toBuffer()],
      program.programId
    );

    const poolTokenAccount = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      payer,
      tokenMint,
      stakingPool,
      true 
    );

     await program.methods
      .stakingInitialize()
      .accounts({
        stakingPool: stakingPool,
        authority: payer.publicKey,
        tokenMint: tokenMint,
        poolTokenAccount: poolTokenAccount.address,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([payer]) 
      .rpc();

  });

});
