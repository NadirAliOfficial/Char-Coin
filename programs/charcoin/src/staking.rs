use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

const PRECISION: u128 = 1_000_000_000_000; // Fixed-point precision multiplier
const THIRTY_DAYS: i64 = 30 * 24 * 3600;
const NINETY_DAYS: i64 = 90 * 24 * 3600;
const ONE_EIGHTY_DAYS: i64 = 180 * 24 * 3600;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum LockupPeriod {
    ThirtyDays,
    NinetyDays,
    OneEightyDays,
}

#[account]
pub struct StakingPool {
    pub total_staked: u64,
    pub acc_reward_per_share: u128, // Accumulated rewards per share (scaled by PRECISION)
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
    pool.total_staked = pool
        .total_staked
        .checked_add(amount)
        .ok_or(ErrorCode::MathError)?;

    // Emit event for staking activity
    emit!(StakingEvent {
        staker: stake_info.staker,
        amount: stake_info.amount,
        lockup_period: stake_info.lockup_period.clone(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Unstakes tokens after the required lockup duration has passed.
pub fn unstake_tokens(ctx: Context<UnstakeTokens>) -> Result<()> {
    let clock = Clock::get()?;
    let stake_info = &ctx.accounts.stake_info;
    let required_lockup = match stake_info.lockup_period {
        LockupPeriod::ThirtyDays => THIRTY_DAYS,
        LockupPeriod::NinetyDays => NINETY_DAYS,
        LockupPeriod::OneEightyDays => ONE_EIGHTY_DAYS,
    };
    require!(
        clock.unix_timestamp >= stake_info.start_time + required_lockup,
        ErrorCode::LockPeriodNotExpired
    );

    let pool = &mut ctx.accounts.staking_pool;
    pool.total_staked = pool
        .total_staked
        .checked_sub(stake_info.amount)
        .ok_or(ErrorCode::MathError)?;

    // Emit event for unstaking activity
    emit!(UnstakingEvent {
        staker: stake_info.staker,
        amount: stake_info.amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Distributes new rewards to stakers by updating the pool's reward accumulator.
/// Reward per share is computed as reward_amount * PRECISION / total_staked.
pub fn distribute_rewards(ctx: Context<DistributeRewards>, reward_amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.staking_pool;
    require!(pool.total_staked > 0, ErrorCode::NoStakedTokens);

    let additional = (reward_amount as u128)
        .checked_mul(PRECISION)
        .ok_or(ErrorCode::MathError)?
        .checked_div(pool.total_staked as u128)
        .ok_or(ErrorCode::MathError)?;
    pool.acc_reward_per_share = pool
        .acc_reward_per_share
        .checked_add(additional)
        .ok_or(ErrorCode::MathError)?;

    let clock = Clock::get()?;
    emit!(RewardDistributionEvent {
        reward_amount,
        new_acc_reward_per_share: pool.acc_reward_per_share,
        timestamp: clock.unix_timestamp,
    });

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

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
}

#[event]
pub struct StakingEvent {
    pub staker: Pubkey,
    pub amount: u64,
    pub lockup_period: LockupPeriod,
    pub timestamp: i64,
}

#[event]
pub struct UnstakingEvent {
    pub staker: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardDistributionEvent {
    pub reward_amount: u64,
    pub new_acc_reward_per_share: u128,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Math error occurred.")]
    MathError,
    #[msg("Lock period not expired yet.")]
    LockPeriodNotExpired,
    #[msg("No tokens are staked in the pool.")]
    NoStakedTokens,
}
