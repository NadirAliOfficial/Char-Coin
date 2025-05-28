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
  let [annualDonationWallet,] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('annual_donation')],
    program.programId
  );
  let [chaiFunds,] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('chai_funds')],
    program.programId
  );
    let [marketingWallet1,] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('marketing_wallet_1')],
    program.programId
  );
    let [marketingWallet2,] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('marketing_wallet_2')],
    program.programId
  );
    let [treasuryAuthority,] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('treasury_authority')],
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
      chaiFunds:chaiFunds,
      marketingWallet1: marketingWallet1,
      marketingWallet2: marketingWallet2,
      admin: admin.publicKey,
      monthlyRewardWallet: monthlyRewardWallet,
      annualRewardWallet: annualRewardWallet,
      monthlyDonationWallet: monthlyDonationWallet,
      annualDonationWallet: annualDonationWallet,
      treasuryAuthority: treasuryAuthority,
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
              configAccount: configAccount.publicKey,

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
 configAccount: configAccount.publicKey,       
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
      configAccount: configAccount.publicKey,    
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
        try {

      await program.methods
        .claimRewardHandler()
        .accounts({
       configAccount: configAccount.publicKey,   
          stakingPool: stakingPool,
          user: userStakePDA,
          userAuthority: user.publicKey,
          userTokenAccount: userAta.address,
          rewardTokenAccount:stakingPoolAta.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();
      }catch (e) {
       if (e instanceof anchor.AnchorError) {
        assert(e.message.includes("StakingPeriodNotMet"))
      } else {
        assert(false);
      }
      }
  });


     it("Emergency halt", async () => {
            let data = await program.account.configAccount.fetch(configAccount.publicKey)
            assert.equal(data.config.halted,false)
      await program.methods
        .changeEmergencyStateHandler(true)
        .accounts({
       configAccount: configAccount.publicKey,  
                 systemProgram: anchor.web3.SystemProgram.programId,
                 payer: admin.publicKey,
 
        })
        .signers([admin])
        .rpc();
            data = await program.account.configAccount.fetch(configAccount.publicKey)
            assert.equal(data.config.halted,true)
     
  });


     it("distribute marketing funds", async () => {
        
      await program.methods
        .distributeMarketingFundsHandler(1000e6)
        .accounts({
       configAccount: configAccount.publicKey,  
          tokenProgram: TOKEN_PROGRAM_ID,
          payer: admin.publicKey,
          destWallet1Ata:
          destWallet2Ata:
        })
        .signers([admin])
        .rpc();
    
     
  });


});


