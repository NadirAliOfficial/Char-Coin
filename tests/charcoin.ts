import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Charcoin } from "../target/types/charcoin";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert, use } from "chai";

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
  const program = anchor.workspace.charcoin as Program<Charcoin>;
  const configAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('config')],
    program.programId
  );




  // Derive monthly reward wallet PDA
  let monthlyTopTierWallet = anchor.web3.Keypair.generate()
  let annualTopTierWallet = anchor.web3.Keypair.generate()
  let monthlyCharityLotteryWallet = anchor.web3.Keypair.generate()
  let annualCharityLotteryWallet = anchor.web3.Keypair.generate()
  let monthlyOneTimeCausesWallet = anchor.web3.Keypair.generate()
  let monthlyInfiniteImpactCausesWallet = anchor.web3.Keypair.generate()
  let annualOneTimeCausesWallet = anchor.web3.Keypair.generate()
  let annualInfiniteImpactCausesWallet = anchor.web3.Keypair.generate()
 


  let charFunds = anchor.web3.Keypair.generate()
  let marketingWallet1 = anchor.web3.Keypair.generate()
  let marketingWallet2 = anchor.web3.Keypair.generate()
  let deathWallet = anchor.web3.Keypair.generate()
  let treasuryAuthority = anchor.web3.Keypair.generate()


  let tokenMint
  let userAta
  let stakingPoolAta
  let stakingPool
  let userStakePDA;
  let marketingWallet1Ata
  let marketingWallet2Ata
  let treasuryAuthorityAta
  let deathWalletAta
  let stakingRewardAccount;
  let stakingRewardAta;
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

    [stakingRewardAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('staking_reward'),tokenMint.toBuffer()],
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
    stakingRewardAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      stakingRewardAccount,
      true
    );
    [userStakePDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('user'), user.publicKey.toBuffer()],
      program.programId
    );

    marketingWallet1Ata = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      marketingWallet1.publicKey,
      false
    );

    marketingWallet2Ata = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      marketingWallet2.publicKey,
      false
    );
    deathWalletAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      deathWallet.publicKey,
      false
    );

    treasuryAuthorityAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      treasuryAuthority.publicKey,
      false
    );

    await mintTo(
      program.provider.connection,
      admin, // fee payer
      tokenMint,
      treasuryAuthorityAta.address, // destination ATA
      admin, // mint authority
      1_000_000_00000
    );
  })
  it("initialized", async () => {
    // Add your test here.
    const context = {
      user: admin.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      config: configAccount,
      mint: tokenMint
    }
    // Define configuration parameters
    const config = {
      charFunds: charFunds.publicKey,
      marketingWallet1: marketingWallet1.publicKey,
      marketingWallet2: marketingWallet2.publicKey,
      admin: admin.publicKey,
      monthlyTopTierWallet:monthlyTopTierWallet.publicKey,
      annualTopTierWallet:annualTopTierWallet.publicKey,
      monthlyCharityLotteryWallet:monthlyCharityLotteryWallet.publicKey,
      annualCharityLotteryWallet:annualCharityLotteryWallet.publicKey,
      monthlyOneTimeCausesWallet:monthlyOneTimeCausesWallet.publicKey,
      monthlyInfiniteImpactCausesWallet:monthlyInfiniteImpactCausesWallet.publicKey,
      annualOneTimeCausesWallet:annualOneTimeCausesWallet.publicKey,
      annualInfiniteImpactCausesWallet:annualInfiniteImpactCausesWallet.publicKey,
      deathWallet: deathWallet.publicKey,
      treasuryAuthority: treasuryAuthority.publicKey,
      charTokenMint:tokenMint,
    };
    await program.methods.initialize(config)
      .accounts(context)
      .signers([admin])
      .rpc();

    await program.methods
      .stakingInitialize()
      .accounts({
        stakingPool: stakingPool,
        stakingRewardAccount: stakingRewardAccount,
        authority: admin.publicKey,
        tokenMint: tokenMint,
        poolTokenAccount: stakingPoolAta.address,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([admin])
      .rpc();
    await program.methods
      .setRewardPercentageHandler(
        // reward       , lockup          ,   vote power      
        new anchor.BN(50),new anchor.BN(1),new anchor.BN(500), //  5 , 1, 0.5 
        new anchor.BN(70),new anchor.BN(90),new anchor.BN(1000), // 7, 90, 1
        new anchor.BN(150),new anchor.BN(180),new anchor.BN(3000),  // 15, 180, 3
        new anchor.BN(180),new anchor.BN(180),new anchor.BN(3000)  // 15, 180, 3

      )
      .accounts({
        stakingPool: stakingPool,
                configAccount: configAccount,
                admin:admin.publicKey

      })
      .signers([admin])
      .rpc();


      /**
       *  min_governance_stake:u64,
        min_stake_duration_voting:u64,
        early_unstake_penalty:u64
       */
    await program.methods
      .updateSettings(
        new anchor.BN(1e6),// min_governance_stake = 1 token
        new anchor.BN(1), // min_stake_duration_voting = 1 sec
        new anchor.BN(100) // early_unstake_penalty = 10%
      )
      .accounts({
                configAccount: configAccount,
                admin:admin.publicKey

      })
      .signers([admin])
      .rpc();






  });




  it("stake", async () => {
    // 1st time
      

    let [userStake] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('user_stake'), user.publicKey.toBuffer(), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    await program.methods
      .stakeTokensHandler(
        new anchor.BN(1e6), // 1 tokens
        new anchor.BN(1) // 1 days for devnet
      )
      .accounts({
        configAccount: configAccount,

        stakingPool: stakingPool,
        user: userStakePDA,
        userStake: userStake,
        userAuthority: user.publicKey,
        userTokenAccount: userAta.address,
        poolTokenAccount: stakingPoolAta.address,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const data = await program.account.userStakeInfo.fetch(userStakePDA)
    const stake_data = await program.account.userStakesEntry.fetch(userStake)

    assert.equal(1e6, Number(data.totalAmount));
    // // assert.equal(30, Number(stake_data.lockup));
    assert.equal(1, Number(stake_data.lockup));

  });



  it("request unstake", async () => {
// 1st
    let [userStake] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('user_stake'), user.publicKey.toBuffer(),new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    let now = Math.floor(Date.now() / 1000);
    await sleep(2000)
    await program.methods
      .requestUnstakeHandler(new anchor.BN(0)) // stake id
      .accounts({
        configAccount: configAccount,
        stakingPool: stakingPool,
        user: userStakePDA,
        userStake: userStake,
        userAuthority: user.publicKey,
      })
      .signers([user])
      .rpc();

    const data = await program.account.userStakesEntry.fetch(userStake)
    assert.isAbove(Number(data.unstakeRequestedAt), now)

  });


  


  it("claim reward", async () => {
        let balance = (await program.provider.connection.getTokenAccountBalance(userAta.address))
    console.log("balance",balance)
        await sleep(3000)

    await mintTo(
      program.provider.connection,
      admin, // fee payer
      tokenMint,
      stakingRewardAta.address, // destination ATA
      admin, // mint authority
      1_000_000_00000
    );
      const [userStake] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('user_stake'), user.publicKey.toBuffer(), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      await program.methods
        .claimRewardHandler(new anchor.BN(0))
        .accounts({
          configAccount: configAccount,
          stakingPool: stakingPool,
          user: userStakePDA,
          userAuthority: user.publicKey,
          userStake: userStake,

          userTokenAccount: userAta.address,
          stakingRewardAta: stakingRewardAta.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

            balance = (await program.provider.connection.getTokenAccountBalance(userAta.address))
    console.log("balance",balance)
   
  });
it("unstake", async () => {
   let balance = (await program.provider.connection.getTokenAccountBalance(userAta.address))
    console.log("balance",balance)
      const [userStake] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('user_stake'), user.publicKey.toBuffer(), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      await program.methods
        .unstakeTokensHandler(new anchor.BN(0))
        .accounts({
          configAccount: configAccount,
          stakingPool: stakingPool,
          userStake: userStake,

          user: userStakePDA,
          userAuthority: user.publicKey,
          userTokenAccount: userAta.address,
          poolTokenAccount: stakingPoolAta.address,
          stakingRewardAta: stakingRewardAta.address,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])

        .rpc();
       balance = (await program.provider.connection.getTokenAccountBalance(userAta.address))
    console.log("balance",balance)
  });

  it("Emergency halt", async () => {
    let data = await program.account.configAccount.fetch(configAccount[0])
    assert.equal(data.config.halted, false)
    await program.methods
      .changeEmergencyStateHandler(true)
      .accounts({
        configAccount: configAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        payer: admin.publicKey,

      })
      .signers([admin])
      .rpc();
    data = await program.account.configAccount.fetch(configAccount[0])
    assert.equal(data.config.halted, true)


  });


  it("halt distribute marketing funds", async () => {
    try {
      await program.methods
        .distributeMarketingFundsHandler(new anchor.BN(1000e6))
        .accounts({
          configAccount: configAccount,
          signer1: treasuryAuthority.publicKey,
          sourceAta: treasuryAuthorityAta.address,
          destWallet1Ata: marketingWallet1Ata.address,
          destWallet2Ata: marketingWallet2Ata.address,
          tokenProgram: TOKEN_PROGRAM_ID,
          deathWalletAta: deathWalletAta.address
        })
        .signers([treasuryAuthority])
        .rpc();
    } catch (e) {
      if (e instanceof anchor.AnchorError) {
        assert(e.message.includes("ProgramIsHalted"))
      } else {
        assert(false);
      }
    }


    await program.methods
      .changeEmergencyStateHandler(false)
      .accounts({
        configAccount: configAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        payer: admin.publicKey,

      })
      .signers([admin])
      .rpc();



  });
  it("distribute marketing funds", async () => {
    let total = 1000e6; // 1000 tokens
    let amount_wallet1 = (total * 425) / 1000; // 42.5%
    let amount_wallet2 = (total * 425) / 1000; // 42.5%
    let amount_death = (total * 150) / 1000; // 15%

    let balance = (await program.provider.connection.getTokenAccountBalance(marketingWallet1Ata.address))
    assert.equal(balance.value.amount, "0");
    balance = (await program.provider.connection.getTokenAccountBalance(marketingWallet2Ata.address))
    assert.equal(balance.value.amount, "0");
    balance = (await program.provider.connection.getTokenAccountBalance(deathWalletAta.address))
    assert.equal(balance.value.amount, "0");

    await program.methods
      .distributeMarketingFundsHandler(new anchor.BN(total))
      .accounts({
        configAccount: configAccount,
        signer1: treasuryAuthority.publicKey,
        sourceAta: treasuryAuthorityAta.address,
        destWallet1Ata: marketingWallet1Ata.address,
        destWallet2Ata: marketingWallet2Ata.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        deathWalletAta: deathWalletAta.address

      })
      .signers([treasuryAuthority])
      .rpc();
    balance = (await program.provider.connection.getTokenAccountBalance(marketingWallet1Ata.address))
    assert.equal(balance.value.amount, amount_wallet1.toString());
    balance = (await program.provider.connection.getTokenAccountBalance(marketingWallet2Ata.address))
    assert.equal(balance.value.amount, amount_wallet2.toString());
    balance = (await program.provider.connection.getTokenAccountBalance(deathWalletAta.address))
    assert.equal(balance.value.amount, amount_death.toString());
  })
  
  it("release Funds", async () => {


    let charFundsAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      charFunds.publicKey,
      false
    );
    let monthlyTopTierWalletAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      monthlyTopTierWallet.publicKey,
      false
    );
    let annualTopTierWalletAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      annualTopTierWallet.publicKey,
      false
    );
    let monthlyCharityLotteryWalletAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      monthlyCharityLotteryWallet.publicKey,
      false
    );
    let annualCharityLotteryWalletAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      annualCharityLotteryWallet.publicKey,
      false
    );
    let monthlyOneTimeCausesWalletAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      monthlyOneTimeCausesWallet.publicKey,
      false
    );
    let annualOneTimeCausesWalletAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      annualOneTimeCausesWallet.publicKey,
      false
    );
    let monthlyInfiniteImpactCausesWalletAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      monthlyInfiniteImpactCausesWallet.publicKey,
      false
    );
    let annualInfiniteImpactCausesWalletAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      annualInfiniteImpactCausesWallet.publicKey,
      false
    );

 
    let total = 1000e6; // 1000 tokens

    await program.methods
      .releaseFundsHandler(new anchor.BN(total))
      .accounts({
        configAccount: configAccount,
        stakingPool: stakingPool,
        treasuryAuthority: treasuryAuthority.publicKey,
        treasuryAta: treasuryAuthorityAta.address,
        charFundsAta: charFundsAta.address,
        monthlyTopTierAta:monthlyTopTierWalletAta.address,
        annualTopTierAta:annualTopTierWalletAta.address,
        monthlyCharityLotteryAta:monthlyCharityLotteryWalletAta.address,
        annualCharityLotteryAta:annualCharityLotteryWalletAta.address,
        monthlyOneTimeCausesAta:monthlyOneTimeCausesWalletAta.address,
        annualOneTimeCausesAta:annualOneTimeCausesWalletAta.address,
        monthlyInfiniteImpactCausesAta:monthlyInfiniteImpactCausesWalletAta.address,
        annualInfiniteImpactCausesAta:annualInfiniteImpactCausesWalletAta.address,
        stakingRewardAta: stakingRewardAta.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([treasuryAuthority])
      .rpc();
  })
it("buyback and burn", async () => {

    await program.methods
      .buybackBurnHandler()
      .accounts({
        configAccount: configAccount,
        mint: tokenMint,
        burnWalletAta: deathWalletAta.address,
        burnAuthority: deathWallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([deathWallet])
      .rpc();
  })

  it("submitProposal", async () => {
    let data = await program.account.configAccount.fetch(configAccount[0])

    const [proposalAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('proposal'), data.config.nextProposalId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    // Proposal details
    const proposalTitle = "Second Proposal";
    const proposalDescription = "Second Proposla : Testing proposal submission on localhost Solana.";
    const proposalDuration = 9; // 1 day in seconds
    await program.methods
      .submitProposalHandler(proposalTitle, proposalDescription, new anchor.BN(proposalDuration))
      .accounts({
        configAccount: configAccount,
        proposal: proposalAccount,
        creator: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();
  })

  it("voteOnProposal", async () => {
    try {
      const [proposalAccount] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('proposal'), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      const voteChoice = true;
      await program.methods
        .voteOnProposalHandler(new anchor.BN(0), voteChoice)
        .accounts({
          configAccount: configAccount,
          proposal: proposalAccount,
          voter: user.publicKey,
          user: userStakePDA,
          stakingPool: stakingPool,
          systemProgram: anchor.web3.SystemProgram.programId,

        })
        .signers([user])
        .rpc();
    } catch (e) {
      if (e instanceof anchor.AnchorError) {
        assert(e.message.includes("VotingNotEligible"))
      } else {
        assert(false);
      }

    }
  })

  it("finalizeProposal", async () => {
    await sleep(10000); // Wait for proposal duration to pass
    const [proposalAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('proposal'), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    await program.methods
      .finalizeProposalHandler(new anchor.BN(0))
      .accounts({
        configAccount: configAccount,
        proposal: proposalAccount,
        admin: user.publicKey,
      })
      .signers([user])
      .rpc();
  })


  it("register Charity", async () => {
    let data = await program.account.configAccount.fetch(configAccount[0])

    let name = "Water for All";
    let description = "Water for underserved communities";
    const [charityAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('charity'), data.config.nextCharityId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    let startTime = Math.floor(Date.now() / 1000) -1;
    let endTime = Math.floor(Date.now() / 1000) + 9;
    const charityWallet = anchor.web3.Keypair.generate()
    const tx = await program.methods
      .registerCharityHandler(
        name,
        description,
        charityWallet.publicKey,
        new anchor.BN(startTime),
        new anchor.BN(endTime)
      )
      .accounts({
        configAccount: configAccount,
        charity: charityAccount,
        registrar: admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
  })
  it("castVote", async () => {
    try {
      const [charityAccount] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('charity'), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [voteRecord] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('vote'), charityAccount.toBuffer(), user.publicKey.toBuffer()],
        program.programId
      );

      const tx = await program.methods
        .castVoteHandler(
          new anchor.BN(0)
        )
        .accounts({
          voteRecord: voteRecord,
          voter: user.publicKey,
          configAccount: configAccount,
          charity: charityAccount,
          user: userStakePDA,
          stakingPool: stakingPool,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user])
        .rpc();
    } catch (e) {
      if (e instanceof anchor.AnchorError) {
        assert(e.message.includes("VotingNotEligible"))
      } else {
        assert(false);
      }

    }
  })




  it("finalize Charity", async () => {
    await sleep(11000); // Wait for charity voting duration to pass
    const [charityAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('charity'), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const tx = await program.methods
      .finalizeCharityVoteHandler(new anchor.BN(0))
      .accounts({
        configAccount: configAccount,
        charity: charityAccount,
        admin: admin.publicKey,
      })
      .signers([admin])
      .rpc();
  })

 


 


});


