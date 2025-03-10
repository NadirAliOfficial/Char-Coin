use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{self, Token, Transfer};

const NINETY_DAYS: i64 = 90 * 24 * 3600; // 90 days in seconds

#[account]
pub struct VestingAccount {
    pub investor: Pubkey,
    pub locked_amount: u64,
    pub start_time: i64,
    pub claimed: bool,
}

#[error_code]
pub enum VestingError {
    #[msg("Vesting period is not yet complete.")]
    VestingPeriodNotCompleted,
    #[msg("Tokens have already been claimed.")]
    AlreadyClaimed,
    #[msg("Insufficient funds in vesting account.")]
    InsufficientFunds,
}

#[event]
pub struct VestingInitializedEvent {
    pub investor: Pubkey,
    pub locked_amount: u64,
    pub start_time: i64,
}

#[event]
pub struct FundsDepositedEvent {
    pub investor: Pubkey,
    pub deposit_amount: u64,
}

#[event]
pub struct TokensClaimedEvent {
    pub investor: Pubkey,
    pub claimed_amount: u64,
    pub claim_time: i64,
}

/// Initializes a vesting account for a private sale investor.
/// Tokens are locked at the current time and remain locked for 90 days.
pub fn initialize_vesting(ctx: Context<InitializeVesting>, locked_amount: u64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let vesting = &mut ctx.accounts.vesting;
    vesting.investor = ctx.accounts.investor.key();
    vesting.locked_amount = locked_amount;
    vesting.start_time = current_time;
    vesting.claimed = false;
    msg!("Vesting account initialized for investor {} with locked amount {}", vesting.investor, locked_amount);
    emit!(VestingInitializedEvent {
        investor: vesting.investor,
        locked_amount,
        start_time: current_time,
    });
    Ok(())
}

/// Deposits funds from the investor's token account (e.g. USDC) into the private sale vault.
pub fn deposit_funds(ctx: Context<DepositFunds>, deposit_amount: u64) -> Result<()> {
    let cpi_accounts = Transfer {
        from: ctx.accounts.source.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.investor.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, deposit_amount)?;
    msg!("Investor {} deposited {} funds into the private sale vault", ctx.accounts.investor.key(), deposit_amount);
    emit!(FundsDepositedEvent {
        investor: ctx.accounts.investor.key(),
        deposit_amount,
    });
    Ok(())
}

/// Allows an investor to claim vested tokens after 90 days.
/// Sale tokens are transferred from the vault to the investor's token account.
pub fn claim_tokens(ctx: Context<ClaimTokens>, sale_token_amount: u64) -> Result<()> {
    let vesting = &mut ctx.accounts.vesting;
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        current_time >= vesting.start_time + NINETY_DAYS,
        VestingError::VestingPeriodNotCompleted
    );
    require!(!vesting.claimed, VestingError::AlreadyClaimed);
    require!(vesting.locked_amount > 0, VestingError::InsufficientFunds);

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.investor_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, sale_token_amount)?;
    
    vesting.claimed = true;
    msg!("Investor {} claimed {} sale tokens", vesting.investor, sale_token_amount);
    emit!(TokensClaimedEvent {
        investor: vesting.investor,
        claimed_amount: sale_token_amount,
        claim_time: current_time,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeVesting<'info> {
    // Increased allocated space: 8 (discriminator) + 32 (investor) + 8 (locked_amount) + 8 (start_time) + 1 (claimed) = 57 bytes.
    #[account(init, payer = investor, space = 8 + 32 + 8 + 8 + 1)]
    pub vesting: Account<'info, VestingAccount>,
    #[account(mut)]
    pub investor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// For SPL token accounts (e.g. USDC or sale token accounts) that don't implement the Discriminator,
/// we use UncheckedAccount with a CHECK comment.
#[derive(Accounts)]
pub struct DepositFunds<'info> {
    /// CHECK: Investor's token account (e.g. USDC) validated by the token program.
    #[account(mut)]
    pub source: UncheckedAccount<'info>,
    /// CHECK: Vault token account to hold deposited funds.
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub investor: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(mut, has_one = investor)]
    pub vesting: Account<'info, VestingAccount>,
    pub investor: Signer<'info>,
    /// CHECK: Investor's token account to receive sale tokens.
    #[account(mut)]
    pub investor_token_account: UncheckedAccount<'info>,
    /// CHECK: Vault token account holding sale tokens.
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
    /// CHECK: Vault authority (typically a PDA) that signs for the vault transfers.
    pub vault_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}
