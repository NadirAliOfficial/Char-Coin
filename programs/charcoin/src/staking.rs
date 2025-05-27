use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::ConfigAccount;

pub fn stake_tokens(ctx: Context<Stake>, amount: u64, lockup: u64) -> Result<()> {
    let staking_pool = &mut ctx.accounts.staking_pool;
    let user = &mut ctx.accounts.user;

    require!(
        lockup == 30 || lockup == 90 || lockup == 180,
        StakingError::WrongStakingPackage
    );
    //require!(user.amount != 0, StakingError::AlreadyStaked);
    let clock = Clock::get()?;
    // Transfer tokens from user to pool
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.pool_token_account.to_account_info(),
        authority: ctx.accounts.user_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Update user staking info
    user.authority = ctx.accounts.user_authority.key();
    user.staking_pool = staking_pool.key();
    user.amount += amount;
    user.staked_at = clock.unix_timestamp;
    user.lockup = lockup;
    user.bump = ctx.bumps.user;
    //staking_pool.total_staked += amount;
    msg!("Staked {} tokens", amount);
    Ok(())
}

pub fn request_unstake_tokens(ctx: Context<UnstakeRequest>) -> Result<()> {
    let user = &mut ctx.accounts.user;

    require!(user.amount > 0, StakingError::NoStakedTokens);
    require!(user.unstake_requested_at == 0, StakingError::UnstakeAlreadyRequested);

    user.unstake_requested_at = Clock::get()?.unix_timestamp;
    msg!(
        "Unstake requested for {} tokens at {}",
        user.amount,
        user.unstake_requested_at
    );
    Ok(())
}

pub fn unstake_tokens(ctx: Context<Unstake>) -> Result<()> {
    let user = &mut ctx.accounts.user;
    let staking_pool = &ctx.accounts.staking_pool;
    let clock = Clock::get()?;
    
    require!(user.unstake_requested_at != 0, StakingError::RequestUnstakeFirst);
   
    require!(clock.unix_timestamp >= user.unstake_requested_at + 172800,StakingError::WaitFor48Hours); // 48 hours in seconds
  

    // Check if user has staked tokens
    require!(user.amount > 0, StakingError::NoStakedTokens);

    let min_staking_duration: i64 = (user.lockup * 24 * 60 * 60).try_into().unwrap();
    let staking_duration = clock.unix_timestamp.saturating_sub(user.staked_at);

    let mut fee = 0;
    if staking_duration < min_staking_duration {
        fee = (user.amount * 10) / 100;
    }

    let amount_to_return = user.amount - fee;

    // Create PDA signer seeds
    let pool_seeds = &[
        b"staking_pool".as_ref(),
        staking_pool.token_mint.as_ref(),
        &[staking_pool.bump],
    ];

    let signer = &[&pool_seeds[..]];
    // Transfer staked tokens back to user

    let cpi_accounts = Transfer {
        from: ctx.accounts.pool_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: staking_pool.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, amount_to_return)?;
    // let amount = user.amount;
    user.amount = 0;
    user.staked_at = 0;
    user.unstake_requested_at = 0;
    //let staking_pool = &mut ctx.accounts.staking_pool;
    // staking_pool.total_staked -= amount;

    msg!("Unstaked {} tokens", amount_to_return);
    Ok(())
}

fn get_reward_percentage(lockup: u64) -> u64 {
    if lockup == 30 {
        return 100;
    }

    if lockup == 90 {
        return 150;
    }

    if lockup == 180 {
        return 200;
    }
    return 0;
}

pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    let user = &mut ctx.accounts.user;
    require!(user.amount > 0, StakingError::NoStakedTokens);
    let clock = Clock::get()?;
    let min_staking_duration = user.lockup * 24 * 60 * 60; // days in seconds

    // Calculate staking duration
    let staking_duration: i64 = clock
        .unix_timestamp
        .saturating_sub(user.staked_at)
        .try_into()
        .unwrap();
    let periods = staking_duration as u64 / min_staking_duration;
    require!(periods > 0, StakingError::StakingPeriodNotMet);
    let reward_percentage = get_reward_percentage(user.lockup);

    let reward_amount = (periods * (user.amount as u64) * reward_percentage) / 100;

    // Create PDA signer seeds
    let pool_seeds = &[
        b"staking_pool".as_ref(),
        ctx.accounts.staking_pool.token_mint.as_ref(),
        &[ctx.accounts.staking_pool.bump],
    ];
    let signer = &[&pool_seeds[..]];
    // Transfer reward tokens to user
    let cpi_accounts = Transfer {
        from: ctx.accounts.reward_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.staking_pool.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, reward_amount)?;
    // user.staked_at += (min_staking_duration * periods) as i64;

    // staking_pool.reward_issued += reward_amount as i64;
    msg!("Claimed reward of {} tokens", reward_amount);
    Ok(())
}



#[derive(Accounts)]
pub struct StakeInitialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + StakingPool::LEN,
        seeds = [b"staking_pool".as_ref(), token_mint.key().as_ref()],
        bump
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(mut)]
    pub authority: Signer<'info>,

    // Modified to remove the constraint for the specific mint address
    pub token_mint: Account<'info, Mint>,
    pub pool_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
        #[account(mut)]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        init_if_needed,
        payer = user_authority,
        space = 8 + std::mem::size_of::<UserStakeInfo>(),
        seeds = [b"user".as_ref(), staking_pool.key().as_ref(), user_authority.key().as_ref()],
        bump
    )]
    pub user: Account<'info, UserStakeInfo>,

    #[account(mut)]
    pub user_authority: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.mint == staking_pool.token_mint,
        constraint = user_token_account.owner == user_authority.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut,
    constraint = pool_token_account.key() == staking_pool.pool_token_account
    )]
    pub pool_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
            #[account(mut)]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"user".as_ref(), staking_pool.key().as_ref(), user_authority.key().as_ref()],
        bump = user.bump,
        constraint = user.authority == user_authority.key(),
        constraint = user.staking_pool == staking_pool.key()
    )]
    pub user: Account<'info, UserStakeInfo>,

    #[account(mut)]
    pub user_authority: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.mint == staking_pool.token_mint,
        constraint = user_token_account.owner == user_authority.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = pool_token_account.key() == staking_pool.pool_token_account
    )]
    pub pool_token_account: Account<'info, TokenAccount>,

 

    pub token_program: Program<'info, Token>,
}



#[derive(Accounts)]
pub struct UnstakeRequest<'info> {
            #[account(mut)]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"user".as_ref(), staking_pool.key().as_ref(), user_authority.key().as_ref()],
        bump = user.bump,
        constraint = user.authority == user_authority.key(),
        constraint = user.staking_pool == staking_pool.key()
    )]
    pub user: Account<'info, UserStakeInfo>,

    #[account(mut)]
    pub user_authority: Signer<'info>,
}


#[derive(Accounts)]
pub struct ClaimReward<'info> {
            #[account(mut)]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"user".as_ref(), staking_pool.key().as_ref(), user_authority.key().as_ref()],
        bump = user.bump,
        constraint = user.authority == user_authority.key(),
        constraint = user.staking_pool == staking_pool.key()
    )]
    pub user: Account<'info, UserStakeInfo>,

    #[account(mut)]
    pub user_authority: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.mint == staking_pool.token_mint,
        constraint = user_token_account.owner == user_authority.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = reward_token_account.mint == staking_pool.token_mint
    )]
    pub reward_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub pool_token_account: Pubkey,
    pub total_staked: u64,
    pub reward_issued: i64,
    pub bump: u8,
}

impl StakingPool {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 1;
}

#[account]
pub struct UserStakeInfo {
    pub authority: Pubkey,
    pub staking_pool: Pubkey,
    pub amount: u64,
    pub staked_at: i64,
    pub lockup: u64,
    pub unstake_requested_at: i64,
    pub bump: u8,
}



#[error_code]
pub enum StakingError {
    #[msg("User has no staked tokens")]
    NoStakedTokens,
    #[msg("Staking period has not been met yet")]
    StakingPeriodNotMet,
    #[msg("Wrong Staking Package")]
    WrongStakingPackage,
    #[msg("Reward has already been claimed")]
    RewardAlreadyClaimed,
    #[msg("Already Staked")]
    AlreadyStaked,
    #[msg("Wait For 48 Hours")]
    WaitFor48Hours,
    #[msg("Request Unstake First")]
    RequestUnstakeFirst,
    #[msg("Unstake Already Requested")]
    UnstakeAlreadyRequested,
}

