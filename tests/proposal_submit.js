const { Connection, Keypair, PublicKey, SystemProgram } = require("@solana/web3.js");
const anchor = require("@project-serum/anchor");
const { PROGRAM_ID, idl, wallet, connection, program, provider } = require("./config");



// Proposal details
const proposalTitle = "Second Proposal";
const proposalDescription = "Second Proposla : Testing proposal submission on localhost Solana.";
const proposalDuration = 9; // 1 day in seconds

async function submitProposal() {

    
    anchor.setProvider(provider);
    // Generate a new Proposal account
    const proposalAccount = Keypair.generate();

    console.log("Submitting proposal to localhost...");

    try {
        const tx = await program.methods
            .submitProposalHandler(proposalTitle, proposalDescription, new anchor.BN(proposalDuration))
            .accounts({
                proposal: proposalAccount.publicKey,
                creator: wallet.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .signers([wallet, proposalAccount])
            .rpc();

        console.log("✅ Proposal submitted successfully!");
        console.log("🔹 Transaction:", tx);
        console.log("🔹 Proposal Account:", proposalAccount.publicKey.toString());
    } catch (error) {
        console.error("❌ Error submitting proposal:", error);
    }
    console.log("\n\n");
    return proposalAccount.publicKey;
}

module.exports = submitProposal;
