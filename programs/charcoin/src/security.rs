use anchor_lang::prelude::*;


/// Context for initializing a multisig wallet.
#[derive(Accounts)]
pub struct InitializeMultisig<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 4 + (10 * 32) + 1 + 1  // discriminator + vec length + max 10 owners * 32 + threshold + wallet_type (1 byte)
    )]
    pub multisig: Account<'info, Multisig>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}


/// Parameters for multisig initialization.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeMultisigParams {
    pub owners: Vec<Pubkey>, // Maximum allowed: 10
    pub threshold: u8,
    pub wallet_type: WalletType,
}

/// Initializes a multisig wallet.
pub fn initialize_multisig(
    ctx: Context<InitializeMultisig>,
    params: InitializeMultisigParams,
) -> Result<()> {
    let multisig = &mut ctx.accounts.multisig;
    require!(params.owners.len() <= 10, MultisigError::TooManyOwners);
    multisig.owners = params.owners;
    multisig.threshold = params.threshold;
    multisig.wallet_type = params.wallet_type;
    msg!("Initialized multisig wallet for {:?} with threshold {}.", multisig.wallet_type, multisig.threshold);
    Ok(())
}

pub fn verify_multisig(ctx: &Context<ExecuteMultisig>) -> Result<()> {
    let multisig = &ctx.accounts.multisig;
    let mut valid_signers = 0;
    let signer_keys = vec![
        ctx.accounts.signer1.key(),
        ctx.accounts.signer2.key(),
        ctx.accounts.signer3.key(),
    ];
    for key in signer_keys {
        if multisig.owners.contains(&key) {
            valid_signers += 1;
        }
    }
    require!(
        valid_signers >= multisig.threshold as usize,
        MultisigError::NotEnoughSignatures
    );
    msg!("Multisig approval successful with {} valid signers.", valid_signers);
    Ok(())
}


#[account]
pub struct BurnTracker {
    /// The total number of tokens burned.
    pub total_burned: u64,
}

/// Define a wallet type for clarity.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum WalletType {
    Marketing,
    Donation,
}

#[account]
pub struct Multisig {
    /// The list of authorized owner public keys.
    pub owners: Vec<Pubkey>,
    /// The number of signatures required for approval.
    pub threshold: u8,
    /// The type of wallet (Marketing or Donation).
    pub wallet_type: WalletType,
}

#[derive(Accounts)]
pub struct ExecuteMultisig<'info> {
    /// CHECK: This is the multisig configuration account storing approved signer keys.
    #[account(mut)]
    pub multisig: Account<'info, Multisig>,
    /// CHECK: Must be one of the approved multisig signer accounts.
    #[account(signer)]
    pub signer1: AccountInfo<'info>,
    /// CHECK: Must be one of the approved multisig signer accounts.
    #[account(signer)]
    pub signer2: AccountInfo<'info>,
    /// CHECK: Must be one of the approved multisig signer accounts.
    #[account(signer)]
    pub signer3: AccountInfo<'info>,
}

#[error_code]
pub enum MultisigError {
    #[msg("Not enough valid signatures provided.")]
    NotEnoughSignatures,
    #[msg("Too many owners provided; maximum allowed is 10.")]
    TooManyOwners,
}
