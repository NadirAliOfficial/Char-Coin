use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum LockupPeriod {
    ThirtyDays,
    NinetyDays,
    OneEightyDays,
}

#[account]
pub struct StakingPool {
    pub total_staked: u64,
}

#[account]
pub struct StakeInfo {
    pub staker: Pubkey,
    pub amount: u64,
    pub start_time: i64,
    pub lockup_period: LockupPeriod,
}

/// Stakes tokens by creating a new stake record and updating the pool.
pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64, lockup: LockupPeriod) -> Result<()> {
    let clock = Clock::get()?;
    let stake_info = &mut ctx.accounts.stake_info;
    stake_info.staker = ctx.accounts.staker.key();
    stake_info.amount = amount;
    stake_info.start_time = clock.unix_timestamp;
    stake_info.lockup_period = lockup;

    let pool = &mut ctx.accounts.staking_pool;
    pool.total_staked = pool.total_staked.checked_add(amount).ok_or(ErrorCode::MathError)?;
    Ok(())
}

/// Unstakes tokens after the required lockup duration has passed.
pub fn unstake_tokens(ctx: Context<UnstakeTokens>) -> Result<()> {
    let clock = Clock::get()?;
    let stake_info = &ctx.accounts.stake_info;
    let required_lockup = match stake_info.lockup_period {
        LockupPeriod::ThirtyDays => 30 * 24 * 3600,
        LockupPeriod::NinetyDays => 90 * 24 * 3600,
        LockupPeriod::OneEightyDays => 180 * 24 * 3600,
    };
    require!(
        clock.unix_timestamp >= stake_info.start_time + required_lockup,
        ErrorCode::LockPeriodNotExpired
    );
    let pool = &mut ctx.accounts.staking_pool;
    pool.total_staked = pool.total_staked.checked_sub(stake_info.amount).ok_or(ErrorCode::MathError)?;
    Ok(())
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(init, payer = staker, space = 8 + 32 + 8 + 8 + 1)]
    pub stake_info: Account<'info, StakeInfo>,
    #[account(mut)]
    pub staker: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut, close = staker)]
    pub stake_info: Account<'info, StakeInfo>,
    #[account(mut)]
    pub staker: Signer<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("An arithmetic error occurred.")]
    MathError,
    #[msg("Lock period not expired yet.")]
    LockPeriodNotExpired,
}
