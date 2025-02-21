use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
// use anchor_spl::token::{self, Token, TokenAccount, Transfer};

const NINETY_DAYS: i64 = 90 * 24 * 3600; // 90 days in seconds

#[account]
pub struct VestingAccount {
    pub investor: Pubkey,       // Investor's public key
    pub locked_amount: u64,     // Amount of tokens locked in vesting
    pub start_time: i64,        // Unix timestamp when tokens are locked
    pub claimed: bool,          // Whether tokens have been claimed
}

#[error_code]
pub enum VestingError {
    #[msg("Vesting period has not yet elapsed.")]
    VestingPeriodNotCompleted,
    #[msg("Tokens have already been claimed.")]
    AlreadyClaimed,
    #[msg("Insufficient funds in vesting account.")]
    InsufficientFunds,
}

/// Initializes a vesting account for a private sale investor.
/// Tokens are locked at the current time and remain locked for 90 days.
pub fn initialize_vesting(ctx: Context<InitializeVesting>, locked_amount: u64) -> Result<()> {
    let vesting = &mut ctx.accounts.vesting;
    let clock = Clock::get()?;
    vesting.investor = ctx.accounts.investor.key();
    vesting.locked_amount = locked_amount;
    vesting.start_time = clock.unix_timestamp;
    vesting.claimed = false;
    msg!("Vesting account initialized for investor {} with locked amount {}", vesting.investor, locked_amount);
    Ok(())
}

/// Allows an investor to claim their vested tokens after 90 days.
/// In a real implementation, this would trigger a CPI call to transfer tokens.
/// For simplicity, we mark the tokens as claimed.
pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
    let vesting = &mut ctx.accounts.vesting;
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp >= vesting.start_time + NINETY_DAYS,
        VestingError::VestingPeriodNotCompleted
    );
    require!(!vesting.claimed, VestingError::AlreadyClaimed);
    require!(vesting.locked_amount > 0, VestingError::InsufficientFunds);

    // In a production contract, perform a CPI call to transfer tokens:
    // let cpi_accounts = Transfer { ... };
    // let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    // token::transfer(cpi_ctx, vesting.locked_amount)?;

    vesting.claimed = true;
    msg!("Investor {} claimed {} tokens.", vesting.investor, vesting.locked_amount);
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeVesting<'info> {
    #[account(init, payer = investor, space = 8 + 32 + 8 + 1)]
    pub vesting: Account<'info, VestingAccount>,
    #[account(mut)]
    pub investor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(mut, has_one = investor)]
    pub vesting: Account<'info, VestingAccount>,
    pub investor: Signer<'info>,
    // Uncomment these lines if you plan to perform a token transfer:
    // #[account(mut)]
    // pub investor_token_account: Account<'info, TokenAccount>,
    // pub token_program: Program<'info, Token>,
}
