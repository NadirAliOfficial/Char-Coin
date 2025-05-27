import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Charcoin } from "../target/types/charcoin";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";

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
const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
describe("char coin test", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const admin = anchor.web3.Keypair.generate()
  const user = anchor.web3.Keypair.generate();
  const configAccount = anchor.web3.Keypair.generate();

  const program = anchor.workspace.charcoin as Program<Charcoin>;

  // Derive monthly reward wallet PDA
  let [monthlyRewardWallet,] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('monthly_reward')],
    program.programId
  );

  // Derive annual reward wallet PDA
  let [annualRewardWallet,] = anchor.web3.PublicKey.findProgramAddressSync(
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
  let stakingPoolAta
  let stakingPool
  let userStakePDA;
  before(async () => {
    await airdropSol(admin.publicKey, 20 * 1e9); // 20 SOL
    await airdropSol(user.publicKey, 5 * 1e9);

    tokenMint = await createMint(
      program.provider.connection,
      admin,
      admin.publicKey,
      null,
      6 // decimals
    );

    [stakingPool] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('staking_pool'), tokenMint.toBuffer()],
      program.programId
    );
    userAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      user,
      tokenMint,
      user.publicKey
    );
    await mintTo(
      program.provider.connection,
      admin, // fee payer
      tokenMint,
      userAta.address, // destination ATA
      admin, // mint authority
      1_000_000_00000
    );

    stakingPoolAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      stakingPool,
      true
    );
    [userStakePDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('user'), stakingPool.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );

  })
  it("initialized", async () => {
    // Add your test here.
    const context = {
      user: admin.publicKey,
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
      admin: admin.publicKey,
      mintAuthorityBump: 0, // Adjust as needed
      monthlyRewardWallet: monthlyRewardWallet,
      annualRewardWallet: annualRewardWallet,
      monthlyDonationWallet: monthlyDonationWallet,
      annualCharityWallet: annualCharityWallet,
    };
    await program.methods.initialize(config)
      .accounts(context)
      .signers([admin, configAccount])
      .rpc();

    await program.methods
      .stakingInitialize()
      .accounts({
        stakingPool: stakingPool,
        authority: admin.publicKey,
        tokenMint: tokenMint,
        poolTokenAccount: stakingPoolAta.address,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([admin])
      .rpc();

  });




  it("stake", async () => {

    await program.methods
      .stakeTokensHandler(
        new anchor.BN(10e6), // 10 tokens
        new anchor.BN(30) // 30 days
      )
      .accounts({
        stakingPool: stakingPool,
        user: userStakePDA,
        userAuthority: user.publicKey,
        userTokenAccount: userAta.address,
        poolTokenAccount: stakingPoolAta.address,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const data = await program.account.userStakeInfo.fetch(userStakePDA)

    assert.equal(10e6, Number(data.amount));
    assert.equal(30, Number(data.lockup));

  });



  it("request unstake", async () => {


    const now = Math.floor(Date.now() / 1000);
    await sleep(2000)
    await program.methods
      .requestUnstakeHandler()
      .accounts({
        stakingPool: stakingPool,
        user: userStakePDA,
        userAuthority: user.publicKey,
      })
      .signers([user])
      .rpc();

    const data = await program.account.userStakeInfo.fetch(userStakePDA)
    assert.isAbove(Number(data.unstakeRequestedAt), now)

  });


  it("unstake", async () => {
    try {
      await program.methods
        .unstakeTokensHandler()
        .accounts({
          stakingPool: stakingPool,
          user: userStakePDA,
          userAuthority: user.publicKey,
          userTokenAccount: userAta.address,
          poolTokenAccount: stakingPoolAta.address,

          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])

        .rpc();
    } catch (e) {
      if (e instanceof anchor.AnchorError) {
        assert(e.message.includes("WaitFor48Hours"))
      } else {
        assert(false);
      }
    }
  });


   it("claim reward", async () => {
      await program.methods
        .claimRewardHandler()
        .accounts({
          stakingPool: stakingPool,
          user: userStakePDA,
          userAuthority: user.publicKey,
          userTokenAccount: userAta.address,
          rewardTokenAccount:stakingPoolAta.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])

        .rpc();
  });


});


