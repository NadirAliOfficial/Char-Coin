const { Connection, PublicKey, Keypair } = require('@solana/web3.js');
const { BN } = require('@project-serum/anchor');
const { wallet } = require("./config");
// const initialize = require("./initialize");
// const submitProposal = require("./proposal_submit");
// const voteProposal = require("./proposal_vote");
// const finalizeProposal = require("./proposal_finalize"); 
// const createSPLToken = require("./createSPLToken");
// const stakingInitialize = require("./stakingInitialize");
// const stakeTokensHandler = require("./stakeTokensHandler");
// const unstakeTokensHandler = require("./unstakeTokensHandler");
// const claimRewardHandler = require("./claimRewardHandler");
// const treasury = require("./Treasury");
const security = require("./Security");
// const donation = require("./Donation");


const run=async()=>{

    // console.log("# _____  Testing Started _____");
    // const configAccount = await initialize();

    // console.log("⏳ Waiting for 5 seconds!");
    // await new Promise((resolve)=>{
    //     setTimeout(()=>resolve(), 5000);
    // })
    // console.log("🕒 Wait finished!");

    // const proposalAccount = await submitProposal();


    // console.log("⏳ Waiting for 5 seconds!");
    // await new Promise((resolve)=>{
    //     setTimeout(()=>resolve(), 5000);
    // })
    // console.log("🕒 Wait finished!");


    // await voteProposal(proposalAccount);  

    
    // console.log("⏳ Waiting for 5 seconds!");
    // await new Promise((resolve)=>{
    //     setTimeout(()=>resolve(), 5000);
    // })
    // console.log("🕒 Wait finished!");

    // await finalizeProposal(proposalAccount);
    // console.log("");

    // const _tokenMint = await createSPLToken();
    // console.log("Token Mint Address : ", _tokenMint);
    // const tokenMint = new PublicKey(_tokenMint);

    //const tokenMint = new PublicKey("8NbD6bL6NmH1toUF5uYRawZFznuykMTTK3Yz4VzuGTnz");

    // const stakingPool = await stakingInitialize(tokenMint);
    //await stakeTokensHandler(tokenMint);
    //await unstakeTokensHandler(tokenMint);

    //await claimRewardHandler(tokenMint);

    // treasury.initializeTreasury();

    /* treasuryPubkey, amount, recipient */
    //const treasuryPubkey = new PublicKey("8TtqHYT8bzM5rFErpAtBNPoE3Shku5RNCriovxjdEtMQ");
    // const amount = new BN(1700);
    // treasury.createWithdrawal(treasuryPubkey, amount);

    //const withdrawalPubkey = new PublicKey("HK63YPpwCQtWcNm5B88nknwgN6VSEM2gpm6Y2CkAqN8p");
    // treasury.approveWithdrawal(treasuryPubkey, withdrawalPubkey);  
    //treasury.executeWithdrawal(treasuryPubkey, withdrawalPubkey);  




   // const { multisig, owners } = await security.initializeMultisigHandler();
   // console.log(multisig);
   // await security.callVerifyMultisig(multisig, owners);


   const emergencyStateAccount = await security.initializeEmergencyStateHandler();
   await security.emergencyHaltHandler(emergencyStateAccount);
   await security.emergencyUnhaltHandler(emergencyStateAccount);

   

    ////    DONATION      ////

    // const now = Math.floor(Date.now() / 1000);        // current unix timestamp
    // const end = now + 2;                           // voting ends in 1 hour
  
    // Step 1: Register the charity
    // const { charityPubkey } = await donation.registerCharity({
    // id: 1,
    // name: "Water for All",
    // description: "Water for underserved communities",
    // charityWallet: wallet.publicKey.toBase58(),
    // startTime: now,
    // endTime: end,
    // });
    // console.log({charityPubkey});





 // Step 2: Cast a vote
    // const voteResult = await donation.castVote({
    //     charityPubkey,
    //     voterKeypair: wallet, // Use wallet or another generated voter keypair
    //     voteWeight: 500,
    // });
    // console.log("✅ Vote Cast:", voteResult.voteRecordPDA.toBase58());




    // console.log("⏳ Waiting for 5 seconds!");
    // await new Promise((resolve)=>{
    //     setTimeout(()=>resolve(), 6000);
    // })
    // console.log("🕒 Wait finished!");


    // last step 
    // const tx = await donation.finalizeCharityVote({ charityPubkey, adminKeypair:wallet });
    // console.log(tx);


};




run();
