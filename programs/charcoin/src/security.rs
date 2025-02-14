use anchor_lang::prelude::*;

#[account]
pub struct Multisig {
    /// The list of authorized owner public keys.
    pub owners: Vec<Pubkey>,
    /// The number of signatures required for approval.
    pub threshold: u8,
}

#[error_code]
pub enum MultisigError {
    #[msg("Not enough valid signatures provided.")]
    NotEnoughSignatures,
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
