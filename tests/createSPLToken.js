const { createMint, getOrCreateAssociatedTokenAccount, mintTo } = require("@solana/spl-token");
const { wallet, connection } = require("./config");

async function createSPLToken() {
    console.log("");
    console.log("💸💸💸 Creating SPL token");
    console.log(wallet);
    // Create a new mint (SPL token)
    const mint = await createMint(connection, wallet, wallet.publicKey, null, 9);

    console.log("🔹 Token Mint Address:", mint.toBase58());

    // Create an associated token account for the payer
    const tokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        wallet,
        mint,
        wallet.publicKey
    );

    console.log("🔹 Token Account Address:", tokenAccount.address.toBase58());

    // Mint some tokens (e.g., 100 tokens, considering 9 decimals)
    await mintTo(
        connection,
        wallet,
        mint,
        tokenAccount.address,
        wallet.publicKey,
        1000000000_000000000 // 100 tokens (since 9 decimals = 100 * 10^9)
    );

    console.log("🔹 Minted 1000000000 tokens to:", tokenAccount.address.toBase58());
    console.log("");

    return mint.toString();
}

module.exports = createSPLToken;
