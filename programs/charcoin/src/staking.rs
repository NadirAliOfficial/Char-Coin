use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

const PRECISION: u128 = 1_000_000_000_000; // Fixed-point precision multiplier
const THIRTY_DAYS: i64 = 30 * 24 * 3600;
const NINETY_DAYS: i64 = 90 * 24 * 3600;
const ONE_EIGHTY_DAYS: i64 = 180 * 24 * 3600;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug, Copy)]
pub enum LockupPeriod {
    ThirtyDays,
    NinetyDays,
    OneEightyDays,
}

#[account]
pub struct StakingPool {
    pub total_staked: u64,             // Actual tokens staked.
    pub total_effective_staked: u64,     // Effective tokens staked (amount * multiplier/100).
    pub acc_reward_per_share: u128,      // Accumulated rewards per effective staked token.
    pub dynamic_interest_rate: u64,      // Dynamic interest rate in basis points.
}

#[account]
pub struct StakeInfo {
    pub staker: Pubkey,
    pub amount: u64,
    pub start_time: i64,
    pub lockup_period: LockupPeriod,
    pub multiplier: u64, // e.g. 100 for 1x, 150 for 1.5x, 200 for 2x.
}

/// Stakes tokens by creating a new stake record and updating both actual and effective staked amounts.
pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64, lockup: LockupPeriod) -> Result<()> {
    let clock = Clock::get()?;
    let stake_info = &mut ctx.accounts.stake_info;
    stake_info.staker = ctx.accounts.staker.key();
    stake_info.amount = amount;
    stake_info.start_time = clock.unix_timestamp;
    stake_info.lockup_period = lockup;
    
    // Set multiplier based on chosen lockup period.
    let multiplier = match lockup {
        LockupPeriod::ThirtyDays => 100,  // 1x
        LockupPeriod::NinetyDays => 150,    // 1.5x
        LockupPeriod::OneEightyDays => 200, // 2x
    };
    stake_info.multiplier = multiplier;
    
    let pool = &mut ctx.accounts.staking_pool;
    pool.total_staked = pool.total_staked.checked_add(amount).ok_or(ErrorCode::MathError)?;
    let effective = amount.checked_mul(multiplier).ok_or(ErrorCode::MathError)?
                          .checked_div(100).ok_or(ErrorCode::MathError)?;
    pool.total_effective_staked = pool.total_effective_staked.checked_add(effective).ok_or(ErrorCode::MathError)?;
    Ok(())
}

/// Unstakes tokens, applying a 10% penalty if unstaking occurs before the full lockup duration.
pub fn unstake_tokens(ctx: Context<UnstakeTokens>) -> Result<()> {
    let clock = Clock::get()?;
    let stake_info = &ctx.accounts.stake_info;
    let pool = &mut ctx.accounts.staking_pool;
    
    let required_lockup = match stake_info.lockup_period {
        LockupPeriod::ThirtyDays => THIRTY_DAYS,
        LockupPeriod::NinetyDays => NINETY_DAYS,
        LockupPeriod::OneEightyDays => ONE_EIGHTY_DAYS,
    };
    
    let (unstake_amount, penalty_applied) = if clock.unix_timestamp < stake_info.start_time + required_lockup {
        // Early unstaking: apply a 10% penalty.
        let penalty = stake_info.amount.checked_mul(10).ok_or(ErrorCode::MathError)?
                                  .checked_div(100).ok_or(ErrorCode::MathError)?;
        (stake_info.amount.checked_sub(penalty).ok_or(ErrorCode::MathError)?, true)
    } else {
        (stake_info.amount, false)
    };
    
    pool.total_staked = pool.total_staked.checked_sub(stake_info.amount).ok_or(ErrorCode::MathError)?;
    let effective = stake_info.amount.checked_mul(stake_info.multiplier).ok_or(ErrorCode::MathError)?
                              .checked_div(100).ok_or(ErrorCode::MathError)?;
    pool.total_effective_staked = pool.total_effective_staked.checked_sub(effective).ok_or(ErrorCode::MathError)?;
    
    if penalty_applied {
        msg!("Early unstaking penalty applied: deducted {} tokens", stake_info.amount - unstake_amount);
    }
    msg!("Unstaked {} tokens", unstake_amount);
    Ok(())
}

/// Distributes new rewards to stakers based on effective stake.
/// Additionally, updates the dynamic interest rate based on the total transaction volume.
pub fn distribute_rewards(ctx: Context<DistributeRewards>, reward_amount: u64, transaction_volume: u64) -> Result<()> {
    let pool = &mut ctx.accounts.staking_pool;
    require!(pool.total_effective_staked > 0, ErrorCode::NoStakedTokens);
    
    // Calculate dynamic interest rate as (transaction_volume * 100) / total_staked, in basis points.
    let dynamic_rate = transaction_volume.checked_mul(100).ok_or(ErrorCode::MathError)?
                                    .checked_div(pool.total_staked).ok_or(ErrorCode::MathError)?;
    pool.dynamic_interest_rate = dynamic_rate;
    
    // Calculate additional reward per share using effective staked amount.
    let additional = (reward_amount as u128)
        .checked_mul(PRECISION).ok_or(ErrorCode::MathError)?
        .checked_div(pool.total_effective_staked as u128).ok_or(ErrorCode::MathError)?;
    pool.acc_reward_per_share = pool.acc_reward_per_share.checked_add(additional).ok_or(ErrorCode::MathError)?;
    
    let clock = Clock::get()?;
    emit!(RewardDistributionEvent {
        reward_amount,
        new_acc_reward_per_share: pool.acc_reward_per_share,
        dynamic_interest_rate: pool.dynamic_interest_rate,
        timestamp: clock.unix_timestamp,
    });
    msg!("Distributed {} rewards. New acc_reward_per_share: {}, dynamic interest rate: {} bp", reward_amount, pool.acc_reward_per_share, pool.dynamic_interest_rate);
    Ok(())
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(init, payer = staker, space = 8 + 32 + 8 + 8 + 8)]
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
pub struct RewardDistributionEvent {
    pub reward_amount: u64,
    pub new_acc_reward_per_share: u128,
    pub dynamic_interest_rate: u64,
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
