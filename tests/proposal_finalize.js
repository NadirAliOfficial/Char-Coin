const { Connection, Keypair, PublicKey, SystemProgram } = require("@solana/web3.js");
const anchor = require("@project-serum/anchor");
const { PROGRAM_ID, idl, wallet, connection, program, provider } = require("./config");


// // Proposal account (Replace with your proposal ID)
// const PROPOSAL_ACCOUNT = new PublicKey("E782okT4RmymT64wKGTHGpZ31F25s7fP3TmEkFiknnp5");

async function finalizeProposal(PROPOSAL_ACCOUNT) {
    // Connect to the local Solana test validator
    anchor.setProvider(provider);

    console.log("Finalizing proposal...");

    try {
        const tx = await program.methods
            .finalizeProposalHandler()
            .accounts({
                proposal: PROPOSAL_ACCOUNT,
                admin: wallet.publicKey,
            })
            .signers([wallet])
            .rpc();

        console.log("✅ Proposal finalized successfully!");
        console.log("🔹 Transaction:", tx);
    } catch (error) {
        console.error("❌ Error finalizing proposal:", error);
    }
}

module.exports = finalizeProposal;
