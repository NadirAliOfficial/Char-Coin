import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { assert } from 'chai';
import { 
  PublicKey, 
  Keypair, 
  SystemProgram, 
  SYSVAR_RENT_PUBKEY 
} from '@solana/web3.js';
import { 
  TOKEN_PROGRAM_ID, 
  createMint, 
  getAssociatedTokenAddress, 
  createAssociatedTokenAccount,
  getAccount
} from '@solana/spl-token';
import idl from '../target/idl/charcoin.json';

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = new Program(idl as anchor.Idl, provider);

describe('CHAR Coin Burn Module', () => {
  let admin: Keypair;
  let user: Keypair;
  let mint: Keypair;
  let tokenAccount: PublicKey;
  let configAccount: Keypair;

  const BURN_AMOUNT = 1000;
  const INITIAL_SUPPLY = 10000;

  before(async () => {
    // Generate accounts
    admin = Keypair.generate();
    user = Keypair.generate();
    configAccount = Keypair.generate();
    mint = Keypair.generate();

    // Fund admin
    await provider.connection.requestAirdrop(admin.publicKey, 1_000_000_000);
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(admin.publicKey, 1_000_000_000)
    );
  });

  beforeEach(async () => {
    // 1. Initialize Program Config
    const [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    await program.rpc.initialize({
      accounts: {
        config: configPDA,
        mint: mint.publicKey,
        user: admin.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [admin]
    });

    // 2. Create Mint
    await createMint(
      provider.connection,
      admin,
      admin.publicKey, // mint authority
      null, // freeze authority
      9 // decimals
    );

    // 3. Create Token Account
    tokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      admin,
      mint.publicKey,
      user.publicKey
    );

    // 4. Mint Initial Tokens
    await program.rpc.mintTokens(new anchor.BN(INITIAL_SUPPLY), {
      accounts: {
        mint: mint.publicKey,
        destination: tokenAccount,
        mintAuthority: admin.publicKey,
        admin: admin.publicKey,
        config: configPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [admin]
    });
  });

  it('Should burn tokens successfully', async () => {
    // Get initial balance
    const initialBalance = await getAccount(provider.connection, tokenAccount);

    // Burn tokens
    await program.rpc.burnTokens(new anchor.BN(BURN_AMOUNT), {
      accounts: {
        mint: mint.publicKey,
        tokenAccount: tokenAccount,
        owner: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [user]
    });

    // Verify final balance
    const finalBalance = await getAccount(provider.connection, tokenAccount);
    assert.equal(
      finalBalance.amount.toString(),
      (BigInt(initialBalance.amount) - BigInt(BURN_AMOUNT)).toString(),
      "Tokens not burned correctly"
    );
  });
});