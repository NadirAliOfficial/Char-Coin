
const fs = require("fs");
const anchor = require("@project-serum/anchor");
const { Keypair, PublicKey, Connection, SystemProgram } = require("@solana/web3.js");
const { publicKey } = require("@project-serum/anchor/dist/cjs/utils");

// Load wallet keypair from wallet.json
const walletPath = "./wallet.json";
const walletKeypair = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync(walletPath))));

// Connect to local Solana blockchain
const connection = new Connection("http://localhost:8899", "confirmed");

// Set up provider and wallet
const wallet = new anchor.Wallet(walletKeypair);
const provider = new anchor.AnchorProvider(connection, wallet, anchor.AnchorProvider.defaultOptions());
anchor.setProvider(provider);

// Load IDL from idl.json
const idl = JSON.parse(fs.readFileSync("./idl.json", "utf8"));
const PROGRAM_ID = new PublicKey("3Ft1CKf4SZwgRk8wR3L3Gnmu1AjKYZbDJS66KGAkKqFE");
const program = new anchor.Program(idl, PROGRAM_ID, provider);

// Helper function to find user's token account
async function getTokenAccount(owner, mint) {
    const accounts = await connection.getParsedTokenAccountsByOwner(owner, { mint });
    if (accounts.value.length === 0) throw new Error("Token account not found");
    return accounts.value[0].pubkey;
}


module.exports = {
    PROGRAM_ID, 
    program, 
    connection, 
    walletKeypair, 
    wallet:walletKeypair, 
    provider, 
    getTokenAccount, 
    idl, 
    SystemProgram
}