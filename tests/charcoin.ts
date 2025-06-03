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
  const program = anchor.workspace.charcoin as Program<Charcoin>;
  const configAccount =  anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('config')],
      program.programId
    );

    


  // Derive monthly reward wallet PDA
  let monthlyRewardWallet = anchor.web3.Keypair.generate()
  // Derive annual reward wallet PDA
  let annualRewardWallet = anchor.web3.Keypair.generate()

  // Derive monthly donation wallet PDA
  let monthlyDonationWallet = anchor.web3.Keypair.generate()

  // Derive annual charity wallet PDA
  let annualDonationWallet = anchor.web3.Keypair.generate()
  let chaiFunds = anchor.web3.Keypair.generate()
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

      [stakingRewardAccount] =  anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('staking_reward'),stakingPool.toBuffer()],
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
      [Buffer.from('user'), stakingPool.toBuffer(), user.publicKey.toBuffer()],
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
      chaiFunds: chaiFunds.publicKey,
      marketingWallet1: marketingWallet1.publicKey,
      marketingWallet2: marketingWallet2.publicKey,
      admin: admin.publicKey,
      monthlyRewardWallet: monthlyRewardWallet.publicKey,
      annualRewardWallet: annualRewardWallet.publicKey,
      monthlyDonationWallet: monthlyDonationWallet.publicKey,
      annualDonationWallet: annualDonationWallet.publicKey,
      deathWallet: deathWallet.publicKey,
      treasuryAuthority: treasuryAuthority.publicKey,
    };
    await program.methods.initialize(config)
      .accounts(context)
      .signers([admin])
      .rpc();

    await program.methods
      .stakingInitialize(new anchor.BN(0.01e6))
      .accounts({
        stakingPool: stakingPool,
        stakingRewardAccount:stakingRewardAccount,
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
        let config_data = await program.account.configAccount.fetch(configAccount[0])

 const [userStake] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('user_stake'), stakingPool.toBuffer(), user.publicKey.toBuffer(),config_data.config.nextStakingId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    await program.methods
      .stakeTokensHandler(
        new anchor.BN(10e6), // 10 tokens
        // new anchor.BN(30) // 30 days
        new anchor.BN(1) // 1 days for devnet
      )
      .accounts({
        configAccount: configAccount,

        stakingPool: stakingPool,
        user: userStakePDA,
        userStake:userStake,
        userAuthority: user.publicKey,
        userTokenAccount: userAta.address,
        poolTokenAccount: stakingPoolAta.address,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const data = await program.account.userStakeInfo.fetch(userStakePDA)
    const stake_data = await program.account.userStakes.fetch(userStake)

    assert.equal(10e6, Number(data.totalAmount));
    // assert.equal(30, Number(data.lockup));
    assert.equal(1, Number(stake_data.lockup));

  });



  it("request unstake", async () => {

const [userStake] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('user_stake'), stakingPool.toBuffer(), user.publicKey.toBuffer(),new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const now = Math.floor(Date.now() / 1000);
    await sleep(2000)
    await program.methods
      .requestUnstakeHandler(new anchor.BN(0)) // stake id
      .accounts({
        configAccount: configAccount,
        stakingPool: stakingPool,
        user: userStakePDA,
        userStake:userStake,
        userAuthority: user.publicKey,
      })
      .signers([user])
      .rpc();

    const data = await program.account.userStakes.fetch(userStake)
    assert.isAbove(Number(data.unstakeRequestedAt), now)

  });


  it("unstake", async () => {
    try {
      const [userStake] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('user_stake'), stakingPool.toBuffer(), user.publicKey.toBuffer(),new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
      await program.methods
        .unstakeTokensHandler(new anchor.BN(0))
        .accounts({
          configAccount: configAccount,
          stakingPool: stakingPool,
                  userStake:userStake,

          user: userStakePDA,
          userAuthority: user.publicKey,
          userTokenAccount: userAta.address,
          poolTokenAccount: stakingPoolAta.address,
          stakingRewardAta:stakingRewardAta.address,
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
  const [userStake] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('user_stake'), stakingPool.toBuffer(), user.publicKey.toBuffer(),new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
      await program.methods
        .claimRewardHandler(new anchor.BN(0))
        .accounts({
          configAccount: configAccount,
          stakingPool: stakingPool,
          user: userStakePDA,
          userAuthority: user.publicKey,
                            userStake:userStake,

          userTokenAccount: userAta.address,
          stakingRewardAta: stakingRewardAta.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();
    } catch (e) {

      if (e instanceof anchor.AnchorError) {
        assert(e.message.includes("StakingPeriodNotMet"))
      } else {
        assert(false);
      }
    }
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
          deathWalletAta:deathWalletAta.address
        })
        .signers([treasuryAuthority])
        .rpc();
    } catch (e) {
      if (e instanceof anchor.AnchorError) {
        assert(e.message.includes("ProgramIsHalted"))
      } else {
        console.log(e)
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
  it(" distribute marketing funds", async () => {
    let total = 1000e6; // 1000 tokens
    let amount_wallet1 = (total * 425) / 1000; // 42.5%
    let amount_wallet2 = (total * 425) / 1000; // 42.5%
    let amount_death = (total * 150) / 1000; // 15%

    let balance = (await program.provider.connection.getTokenAccountBalance(marketingWallet1Ata.address))
    assert.equal(balance.value.amount, "0");
    balance = (await program.provider.connection.getTokenAccountBalance(marketingWallet2Ata.address))
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
        deathWalletAta:deathWalletAta.address

      })
      .signers([treasuryAuthority])
      .rpc();
    balance = (await program.provider.connection.getTokenAccountBalance(marketingWallet1Ata.address))
    assert.equal(balance.value.amount, amount_wallet1.toString());
    balance = (await program.provider.connection.getTokenAccountBalance(marketingWallet2Ata.address))
    assert.equal(balance.value.amount, amount_wallet2.toString());
  })
  it("buyback and burn",async()=>{

      await program.methods
      .buybackBurnHandler(new anchor.BN(1e6),new anchor.BN(1))
      .accounts({
        configAccount: configAccount,
        mint:tokenMint,
        burnWalletAta:deathWalletAta.address,
        burnAuthority:deathWallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([deathWallet])
      .rpc();
  })
  it("release Funds", async () => {


    let chaiFundsAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      chaiFunds.publicKey,
      false
    );
    let annualDonationAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      annualDonationWallet.publicKey,
      false
    );

    let monthlyDonationAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      monthlyDonationWallet.publicKey,
      false
    );
    let annualRewardAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      annualRewardWallet.publicKey,
      false
    );
    let monthlyRewardAta = await getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      admin,
      tokenMint,
      monthlyRewardWallet.publicKey,
      false
    );
    let total = 1000e6; // 1000 tokens

    await program.methods
      .releaseFundsHandler(new anchor.BN(total))
      .accounts({
        configAccount: configAccount,
        stakingPool:stakingPool,
        treasuryAuthority: treasuryAuthority.publicKey,
        treasuryAta: treasuryAuthorityAta.address,
        chaiFundsAta: chaiFundsAta.address,
        annualDonationAta: annualDonationAta.address,
        monthlyDonationAta: monthlyDonationAta.address,
        annualRewardAta: annualRewardAta.address,
        monthlyRewardAta: monthlyRewardAta.address,
        stakingRewardAta: stakingRewardAta.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([treasuryAuthority])
      .rpc();
  })

 
  it("submitProposal", async () => {
    let data = await program.account.configAccount.fetch(configAccount[0])

    const [proposalAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('proposal'), user.publicKey.toBuffer(), data.config.nextProposalId.toArrayLike(Buffer, "le", 8)],
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
        [Buffer.from('proposal'), user.publicKey.toBuffer(), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
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
      [Buffer.from('proposal'), user.publicKey.toBuffer(), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    await program.methods
      .finalizeProposalHandler()
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

    let name =  "Water for All";
    let description =  "Water for underserved communities";
    const [charityAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('charity'),data.config.nextCharityId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    let startTime =  Math.floor(Date.now() / 1000) ; 
    let endTime =  Math.floor(Date.now() / 1000) + 9;
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
      registrar: user.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([user])
    .rpc();
})
it("castVote", async () => {
  try{
    const [charityAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('charity'),new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
        const [voteRecord] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('vote'),charityAccount.toBuffer(), user.publicKey.toBuffer()], 
      program.programId
    );

  const tx = await program.methods
    .castVoteHandler(
      new anchor.BN(500),//voteWeight
    )
    .accounts({
      voteRecord:voteRecord,
      voter: user.publicKey,
      configAccount: configAccount,
      charity: charityAccount,
      user:userStakePDA,
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
      [Buffer.from('charity'),new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

  const tx = await program.methods
    .finalizeCharityVoteHandler()
    .accounts({
      configAccount: configAccount,
      charity: charityAccount,
      admin: user.publicKey,
    })
    .signers([user])
    .rpc();
})

  it("init dao treasury", async () => {
     const [treasuryAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('treasury'), ],
      program.programId
    );
    const owners = [
      admin.publicKey,
treasuryAuthority.publicKey
    ]
       const tx = await program.methods
          .initializeTreasuryHandler(owners, 2)
          .accounts({
            treasury: treasuryAccount,
            signer: admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([admin])
          .rpc();
  })


    it("create withdrawal ", async () => {
     const [treasuryAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('treasury'), ],
      program.programId
    );
      const [withdrawalAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('withdrawal'), ],
      program.programId
    );
       const tx = await program.methods
          .createWithdrawalHandler(new anchor.BN(2),admin.publicKey)
          .accounts({
                  configAccount: configAccount,
            treasury: treasuryAccount,
            withdrawal:withdrawalAccount,
            signer: admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([admin])
          .rpc();
  })


      it("approve withdrawal ", async () => {
     const [treasuryAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('treasury'), ],
      program.programId
    );
      const [withdrawalAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('withdrawal'), ],
      program.programId
    );
       const tx = await program.methods
          .approveWithdrawalHandler()
          .accounts({
                  configAccount: configAccount,
            treasury: treasuryAccount,
            withdrawal:withdrawalAccount,
            signer: admin.publicKey,
          })
          .signers([admin])
          .rpc();

            await program.methods
          .approveWithdrawalHandler()
          .accounts({
                  configAccount: configAccount,
            treasury: treasuryAccount,
            withdrawal:withdrawalAccount,
            signer: treasuryAuthority.publicKey,
          })
          .signers([treasuryAuthority])
          .rpc();
  })



      it("execute withdrawal ", async () => {
     const [treasuryAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('treasury'), ],
      program.programId
    );
    await airdropSol(treasuryAccount, 1 * 1e9); 
      const [withdrawalAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('withdrawal'), ],
      program.programId
    );
       const tx = await program.methods
          .executeWithdrawalHandler()
          .accounts({
                  configAccount: configAccount,
            treasury: treasuryAccount,
            withdrawal:withdrawalAccount,
            recipient: admin.publicKey,
            signer: admin.publicKey,
          })
          .signers([admin])
          .rpc();
  })

  

});


