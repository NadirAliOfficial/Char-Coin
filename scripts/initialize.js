const anchor = require("@project-serum/anchor");
const { Keypair, PublicKey,  SystemProgram } = require("@solana/web3.js");
const { program, PROGRAM_ID, walletKeypair, wallet } = require("./config");


// Generate config and mint accounts
const configAccount = Keypair.generate();
const mint = Keypair.generate(); // Replace with actual mint if already created

async function initialize() {
    console.log("# Initializing CHAR Coin program");
  // Derive monthly reward wallet PDA
  let [monthlyRewardWallet,] = await PublicKey.findProgramAddress(
    [Buffer.from('monthly_reward')],
    PROGRAM_ID
  );
  
  // Derive annual reward wallet PDA
  let [annualRewardWallet,] = await PublicKey.findProgramAddress(
    [Buffer.from('annual_reward')],
    PROGRAM_ID
  );
  
  // Derive monthly donation wallet PDA
  let [monthlyDonationWallet,] = await PublicKey.findProgramAddress(
    [Buffer.from('monthly_donation')],
    PROGRAM_ID
  );
  
  // Derive annual charity wallet PDA
  let [annualCharityWallet,] = await PublicKey.findProgramAddress(
    [Buffer.from('annual_charity')],
    PROGRAM_ID
  );


    // Define configuration parameters
    const config = {
        tokenSupply: new anchor.BN(1000000000), // Example: 1 billion tokens
        feePercentage: 2,
        buybackPercentage: 5,
        donationPercentage: 3,
        stakingPercentage: 10,
        admin: wallet.publicKey,
        mintAuthorityBump: 0, // Adjust as needed
        monthlyRewardWallet:monthlyRewardWallet,
        annualRewardWallet: annualRewardWallet,
        monthlyDonationWallet: monthlyDonationWallet,
        annualCharityWallet: annualCharityWallet,
    };


    const tx = await program.methods
        .initialize(config)
        .accounts({
            config: configAccount.publicKey,
            mint: mint.publicKey,
            user: wallet.publicKey,
            systemProgram: SystemProgram.programId,
        })
        .signers([walletKeypair, configAccount])
        .rpc();

    console.log("✅  Transaction successful! Tx Hash:", tx);
    console.log("🔹 Config Account Address:", configAccount.publicKey.toBase58());
    
    return configAccount.publicKey.toBase58();
}



module.exports = initialize;


