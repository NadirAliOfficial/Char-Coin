const { Connection, Keypair, PublicKey, SystemProgram } = require("@solana/web3.js");
const anchor = require("@project-serum/anchor");
const { PROGRAM_ID, idl, wallet, connection, program, provider } = require("./config");



async function voteProposal(PROPOSAL_ACCOUNT) 
{
    const voteChoice = true;
    const amountStaked = 600; 
    anchor.setProvider(provider);
    console.log("Casting vote on proposal...");

    try {
        const tx = await program.methods
            .voteOnProposalHandler(new anchor.BN(0), voteChoice, new anchor.BN(amountStaked))
            .accounts({
                proposal: PROPOSAL_ACCOUNT,
                voter: wallet.publicKey,
            })
            .signers([wallet])
            .rpc();

        console.log("✅ Vote cast successfully!");
        console.log("🔹 Transaction:", tx);
    } catch (error) {
        console.error("❌ Error casting vote:", error);
    }
    console.log("\n\n");
}



module.exports = voteProposal;

