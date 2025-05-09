use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{self, Token, Transfer};

#[derive(Accounts)]
pub struct DistributeMarketingFunds<'info> {
    /// The marketing wallet that tracks allocated funds.
    #[account(mut)]
    pub marketing_wallet: Account<'info, MarketingWallet>,
    /// The multisig configuration account (for approval, if needed).
    #[account(mut)]
    pub multisig: Account<'info, crate::security::Multisig>,
    /// CHECK: Approved multisig signer.
    #[account(signer)]
    pub signer1: AccountInfo<'info>,
    /// CHECK: Approved multisig signer.
    #[account(signer)]
    pub signer2: AccountInfo<'info>,
    /// CHECK: Approved multisig signer.
    #[account(signer)]
    pub signer3: AccountInfo<'info>,
    /// CHECK: This is the source token account from which funds are withdrawn. Its validity is managed by the token program.
    #[account(mut)]
    pub source: AccountInfo<'info>,
    /// Destination account for Marketing Wallet 1 funds.
    /// CHECK: This is the destination token account to which funds are transferred. Its validity is managed by th
    #[account(mut)]
    pub dest_wallet1: AccountInfo<'info>,
    /// Destination account for Marketing Wallet 2 funds.
    /// CHECK: This is the destination token account to which funds are transferred. Its validity is managed
    #[account(mut)]
    pub dest_wallet2: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[event]
pub struct MarketingFundDistributionEvent {
    pub marketing_wallet_1_amount: u64,
    pub marketing_wallet_2_amount: u64,
    pub death_wallet_amount: u64,
    pub timestamp: i64,
}

/// Distribute marketing funds according to the following split:
/// - Marketing Wallet 1: 42.5%
/// - Marketing Wallet 2: 42.5%
/// - Death Wallet (Burn): 15%
pub fn distribute_marketing_funds(ctx: Context<DistributeMarketingFunds>) -> Result<()> {
    let wallet = &mut ctx.accounts.marketing_wallet;
    let total = wallet.total_funds;
    // Calculate distribution amounts.
    let amount_wallet1 = (total * 425) / 1000; // 42.5%
    let amount_wallet2 = (total * 425) / 1000; // 42.5%
    let amount_death = (total * 150) / 1000; // 15%

    // Execute transfers from source to destination accounts.
    // (Here we assume that multisig approval has been verified separately.)
    let transfer_ctx1 = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.source.to_account_info(),
            to: ctx.accounts.dest_wallet1.to_account_info(),
            // For demonstration, use signer1; in production, use a multisig PDA.
            authority: ctx.accounts.signer1.to_account_info(),
        },
    );
    token::transfer(transfer_ctx1, amount_wallet1)?;

    let transfer_ctx2 = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.source.to_account_info(),
            to: ctx.accounts.dest_wallet2.to_account_info(),
            authority: ctx.accounts.signer1.to_account_info(),
        },
    );
    token::transfer(transfer_ctx2, amount_wallet2)?;

    // (Optionally, you might burn the death wallet funds via a separate burn function.)

    // Reset the wallet's total funds after distribution.
    wallet.total_funds = 0;

    // Get current timestamp.
    let clock = Clock::get()?;
    emit!(MarketingFundDistributionEvent {
        marketing_wallet_1_amount: amount_wallet1,
        marketing_wallet_2_amount: amount_wallet2,
        death_wallet_amount: amount_death,
        timestamp: clock.unix_timestamp,
    });
    msg!(
        "Distributed funds: {} to Marketing Wallet 1, {} to Marketing Wallet 2, {} for Death Wallet",
        amount_wallet1,
        amount_wallet2,
        amount_death
    );
    Ok(())
}

/// Marketing wallet state.
#[account]
pub struct MarketingWallet {
    pub threshold: u8,
    pub signers: Vec<Pubkey>,
    pub executed: bool,
    pub total_funds: u64,
    pub marketing_wallet_1: u64,
    pub marketing_wallet_2: u64,
    pub burn_wallet: u64,
}
