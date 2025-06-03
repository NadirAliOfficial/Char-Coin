use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::ConfigAccount;
#[event]
pub struct MarketingFundDistributionEvent {
    pub marketing_wallet_1_amount: u64,
    pub marketing_wallet_2_amount: u64,
    pub death_wallet_amount: u64,
    pub timestamp: i64,
}
#[derive(Accounts)]
pub struct DistributeMarketingFunds<'info> {
    /// The marketing wallet that tracks allocated funds.
  #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
    /// CHECK: signer.
    #[account(
        mut,
        constraint = config_account.config.treasury_authority == signer1.key() // Ensure the signer is the admin
    )]
    pub signer1: Signer<'info>,

    /// CHECK: This is the source token account from which funds are withdrawn. Its validity is managed by the token program.
    #[account(mut,
        constraint = source_ata.owner == config_account.config.treasury_authority// Ensure the owner matches the marketing wallet
    )]
    pub source_ata: Account<'info, TokenAccount>,
    /// Destination token account for Marketing Wallet 1 funds.
    #[account(
        mut,
        constraint = dest_wallet1_ata.owner == config_account.config.marketing_wallet_1 ,// Ensure the owner matches the marketing wallet

    )]
    pub dest_wallet1_ata: Account<'info, TokenAccount>,
    /// Destination token  account for Marketing Wallet 2 funds.
    #[account(
        mut,
        constraint = dest_wallet2_ata.owner == config_account.config.marketing_wallet_2,// Ensure the owner matches the marketing wallet
    )]
    pub dest_wallet2_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = death_wallet_ata.owner == config_account.config.death_wallet,// Ensure the owner matches the marketing wallet
    )]
    pub death_wallet_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}



/// Distribute marketing funds according to the following split:
/// - Marketing Wallet 1: 42.5%
/// - Marketing Wallet 2: 42.5%
/// - Death Wallet (Burn): 15%
pub fn distribute_marketing_funds(
    ctx: Context<DistributeMarketingFunds>,
    total_amount: u64,
) -> Result<()> {
    // let wallet = &mut ctx.accounts.marketing_wallet;
    let total = total_amount;
    // Calculate distribution amounts.
    let amount_wallet1 = (total * 425) / 1000; // 42.5%
    let amount_wallet2 = (total * 425) / 1000; // 42.5%
    let amount_death = (total * 150) / 1000; // 15%

    // Execute transfers from source to destination accounts.
    let transfer_ctx1 = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.source_ata.to_account_info(),
            to: ctx.accounts.dest_wallet1_ata.to_account_info(),
            // use signer1; in production.
            authority: ctx.accounts.signer1.to_account_info(),
        },
    );
    token::transfer(transfer_ctx1, amount_wallet1)?;

    let transfer_ctx2 = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.source_ata.to_account_info(),
            to: ctx.accounts.dest_wallet2_ata.to_account_info(),
            authority: ctx.accounts.signer1.to_account_info(),
        },
    );
    token::transfer(transfer_ctx2, amount_wallet2)?;

    // (Optionally, you might burn the death wallet funds via a separate burn function.)
    let transfer_death_wallet = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.source_ata.to_account_info(),
            to: ctx.accounts.death_wallet_ata.to_account_info(),
            authority: ctx.accounts.signer1.to_account_info(),
        },
    );
    token::transfer(transfer_death_wallet, amount_death)?;

    // Reset the wallet's total funds after distribution.
    // wallet.total_funds = 0;

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


