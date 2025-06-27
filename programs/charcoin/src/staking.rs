use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token_2022::{transfer_checked, Token2022 as Token, TransferChecked};
use anchor_spl::token_interface::{TokenAccount,Mint};
use crate::{ConfigAccount, CustomError};
// const FOURTY_EIGHT_HOURS_IN_SECONDS:u32 = 172800;
const FOURTY_EIGHT_HOURS_IN_SECONDS:u32 = 1;
// const ONE_DAY_IN_SECONDS:u32 = 86400;
const ONE_DAY_IN_SECONDS:u32 = 1;

pub fn stake_tokens(ctx: Context<Stake>, amount: u64, lockup: u16) -> Result<()> {
    require!(amount > 0, CustomError::NoStakedTokens);
    let staking_pool = &mut ctx.accounts.staking_pool;
    let config_account = &mut ctx.accounts.config_account;
    let user = &mut ctx.accounts.user;
    let user_stake = &mut ctx.accounts.user_stake;

    require!(
        staking_pool
            .stake_lockup_reward_array
            .iter()
            .any(|x| x.lockup_days == lockup),
        CustomError::WrongStakingPackage
    );
    require!(user_stake.amount == 0, CustomError::AlreadyStaked);
    require!(user_stake.unstaked_at == 0, CustomError::AlreadyUnStaked);
    let clock = Clock::get()?.unix_timestamp as u64;
    // Transfer tokens from user to pool
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.pool_token_account.to_account_info(),
        authority: ctx.accounts.user_authority.to_account_info(),
        mint:ctx.accounts.mint.to_account_info()
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer_checked(cpi_ctx, amount,ctx.accounts.mint.decimals)?;

    // update user stake entry
    user_stake.stake_id = user.stake_count;
    user_stake.amount = amount;
    user_stake.staked_at = clock;
    user_stake.lockup = lockup;
    // update staking pool state
    staking_pool.total_staked += amount;

    // Update user staking info
    user.authority = ctx.accounts.user_authority.key();
    user.staking_pool = staking_pool.key();
    user.bump = ctx.bumps.user;

    if user.total_amount < config_account.config.min_governance_stake &&
     user.total_amount + amount >= config_account.config.min_governance_stake{
        user.eligible_at = clock;
    }
    
    user.total_amount += amount;
    user.stake_count += 1;



    let vote_power = staking_pool
    .stake_lockup_reward_array
    .iter()
    .find(|x| x.lockup_days == lockup)
    .unwrap()
    .vote_power;
    //  voting_amount = 500 * 10e6 / 1000    e.g 500 = 0.5, 1000 = 1
    let vote_weight = (vote_power as u128 * amount as u128 / 1000) as u64;

    user.voting_power += vote_weight;


    msg!("Staked {} tokens", amount);
    Ok(())
}



pub fn request_unstake_tokens(ctx: Context<UnstakeRequest>, _stake_id: u64) -> Result<()> {
    let user_stake = &mut ctx.accounts.user_stake;

    require!(user_stake.amount > 0, CustomError::NoStakedTokens);
    require!(
        user_stake.unstake_requested_at == 0,
        CustomError::UnstakeAlreadyRequested
    );
    require!(user_stake.unstaked_at == 0, CustomError::AlreadyUnStaked);

    user_stake.unstake_requested_at = Clock::get()?.unix_timestamp as u64;
    msg!(
        "Unstake requested for {} tokens at {}",
        user_stake.amount,
        user_stake.unstake_requested_at
    );
    Ok(())
}

pub fn unstake_tokens(ctx: Context<Unstake>, _stake_id: u64) -> Result<()> {
    let user = &mut ctx.accounts.user;
    let config_account = &mut ctx.accounts.config_account;
    let user_stake = &mut ctx.accounts.user_stake;
    let staking_pool = &mut ctx.accounts.staking_pool;

    require!(user_stake.unstaked_at == 0, CustomError::AlreadyUnStaked);

    let clock = Clock::get()?.unix_timestamp as u64;

    require!(
        user_stake.unstake_requested_at != 0,
        CustomError::RequestUnstakeFirst
    );
    // unstake must be requested at least 48 hours in advance..
    require!(
        clock >= user_stake.unstake_requested_at + FOURTY_EIGHT_HOURS_IN_SECONDS as u64, 
        CustomError::WaitPeriodNotOverYet
    ); 
    

    // Check if user has staked tokens
    require!(user_stake.amount > 0, CustomError::NoStakedTokens);

    let min_staking_duration = (user_stake.lockup as u64 * ONE_DAY_IN_SECONDS as u64).try_into().unwrap();
    let staking_duration = clock.saturating_sub(user_stake.staked_at);

    user_stake.unstaked_at = clock;
    user.total_amount -= user_stake.amount;
    staking_pool.total_staked -= user_stake.amount;

    let mut fee = 0;
    if staking_duration < min_staking_duration {
        let penalty = config_account.config.early_unstake_penalty; // 100 = 10%
        fee = (user_stake.amount * penalty) / 1000;
    }

    let amount_to_return = user_stake.amount - fee;



        let vote_power = staking_pool
    .stake_lockup_reward_array
    .iter()
    .find(|x| x.lockup_days == user_stake.lockup)
    .unwrap()
    .vote_power;

    let vote_weight = (vote_power as u128 * user_stake.amount as u128 / 1000) as u64;
    if user.voting_power > vote_weight{
        user.voting_power = user.voting_power
         .checked_sub(vote_weight)
            .ok_or(CustomError::MathError)?;
    }

    // Create PDA signer seeds
    let pool_seeds = &[
        b"staking_pool".as_ref(),
        staking_pool.token_mint.as_ref(),
        &[staking_pool.bump],
    ];

    let signer = &[&pool_seeds[..]];
    // Transfer staked tokens back to user

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.pool_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: staking_pool.to_account_info(),
        mint:ctx.accounts.mint.to_account_info()
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    transfer_checked(cpi_ctx, amount_to_return,ctx.accounts.mint.decimals)?;

    if fee != 0 {
        // send penalty fee to staking reward account ata
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.pool_token_account.to_account_info(),
            to: ctx.accounts.staking_reward_ata.to_account_info(),
            authority: staking_pool.to_account_info(),
            mint:ctx.accounts.mint.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        transfer_checked(cpi_ctx, fee,ctx.accounts.mint.decimals)?;
    }

    msg!(
        "Unstaked {} tokens and penalty fee {}",
        amount_to_return,
        fee
    );
    Ok(())
}

pub fn claim_reward(ctx: Context<ClaimReward>, _stake_id: u64) -> Result<()> {
    require!(
        ctx.accounts.staking_reward_ata.amount > 0,
        CustomError::StakingRewardInsufficientBalance
    );
    let staking_pool = &mut ctx.accounts.staking_pool;

    let user = &mut ctx.accounts.user;
    let user_stake = &mut ctx.accounts.user_stake;

    require!(user_stake.amount > 0, CustomError::NoStakedTokens);
    require!(staking_pool.total_staked > 0, CustomError::NoStakedTokens);
    let clock = Clock::get()?.unix_timestamp as u64;
    let min_staking_duration = (user_stake.lockup as u64) * ONE_DAY_IN_SECONDS as u64; // days in seconds
    let staking_duration: u64;
    if user_stake.unstaked_at == 0 {
        // Case 1: Stake is still active
        staking_duration = clock
            .saturating_sub(user_stake.staked_at)
            .try_into()
            .unwrap();
    } else {
        // Case 2: Stake is unstaked - claim any unclaimed rewards
        staking_duration = user_stake
            .unstaked_at
            .saturating_sub(user_stake.staked_at)
            .try_into()
            .unwrap();
    }
    require!(staking_duration > 0, CustomError::NothingToClaim);

    let total_periods_earned = staking_duration as u64 / min_staking_duration;
    require!(
        total_periods_earned > user_stake.current_period,
        CustomError::StakingPeriodNotMet
    );
    let claimable_periods = total_periods_earned.saturating_sub(user_stake.current_period);
    require!(claimable_periods > 0, CustomError::StakingPeriodNotMet);

    user_stake.current_period = total_periods_earned;

    let reward_percentage = staking_pool
        .stake_lockup_reward_array
        .iter()
        .find(|x| x.lockup_days == user_stake.lockup)
        .ok_or(CustomError::WrongStakingPackage)?
        .reward_bps;

    let reward_amount =
        (claimable_periods * (user_stake.amount as u64) * reward_percentage as u64) / 1000;
    require!(reward_amount > 0, CustomError::NothingToClaim);

    let seeds: &[&[u8]] = &[
        b"staking_reward",
        staking_pool.token_mint.as_ref(),
        &[ctx.bumps.staking_reward],
    ];  
    let signer = &[seeds];

    // Transfer reward tokens to user
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.staking_reward_ata.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.staking_reward.to_account_info(),
        mint:ctx.accounts.mint.to_account_info()
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    transfer_checked(cpi_ctx, reward_amount,ctx.accounts.mint.decimals)?;

    ctx.accounts.staking_pool.reward_issued += reward_amount;
    user.reward_issued += reward_amount;
    msg!("Claimed reward of {} tokens", reward_amount);
    Ok(())
}

pub fn set_reward_percentage(
    ctx: Context<SetReward>,
    reward1: u16,
    lockup1: u16,
    vote_power1: u16, // reward = 50 (5%), lockup = 30 (days), vote_power = 500 (0.5x)
    reward2: u16,
    lockup2: u16,
    vote_power2: u16,
    reward3: u16,
    lockup3: u16,
    vote_power3: u16,
    reward4: u16,
    lockup4: u16,
    vote_power4: u16,
) -> Result<()> {
    let staking_pool = &mut ctx.accounts.staking_pool;
    staking_pool.stake_lockup_reward_array[0] = LockupReward {
        lockup_days: lockup1,
        reward_bps: reward1,
        vote_power: vote_power1,
    };
    staking_pool.stake_lockup_reward_array[1] = LockupReward {
        lockup_days: lockup2,
        reward_bps: reward2,
        vote_power: vote_power2,
    };
    staking_pool.stake_lockup_reward_array[2] = LockupReward {
        lockup_days: lockup3,
        reward_bps: reward3,
        vote_power: vote_power3,
    };
    staking_pool.stake_lockup_reward_array[3] = LockupReward {
        lockup_days: lockup4,
        reward_bps: reward4,
        vote_power: vote_power4,
    };

    Ok(())
}
#[derive(Accounts)]
pub struct StakeInitialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<StakingPool>(),
        seeds = [b"staking_pool".as_ref(), token_mint.key().as_ref()],
        bump
    )]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(
        init,
        payer = authority,
        space = 8,
        seeds = [b"staking_reward".as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub staking_reward: Account<'info, StakingRewards>,

   #[account(
        mut,
        constraint = authority.key() == config_account.config.admin,
    )]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(mut,
        constraint = config_account.config.char_token_mint == token_mint.key(),
    )]
    pub token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
        constraint = pool_token_account.mint == token_mint.key(),
        constraint = pool_token_account.owner == staking_pool.key()

    )]
    pub pool_token_account: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetReward<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        mut,
        seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        constraint = admin.key() == config_account.config.admin,
    )]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        mut,
        seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,
   #[account(mut,
    constraint = mint.key() == config_account.config.char_token_mint)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user_authority,
        space = 8 + std::mem::size_of::<UserStakeInfo>(),
        seeds = [b"user".as_ref(), user_authority.key().as_ref()],
        bump
    )]
    pub user: Account<'info, UserStakeInfo>,
    #[account(
        init_if_needed,
        payer = user_authority,
        space = 8 + std::mem::size_of::<UserStakesEntry>(),
        seeds = [b"user_stake".as_ref(), user_authority.key().as_ref(),user.stake_count.to_le_bytes().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStakesEntry>,

    #[account(mut)]
    pub user_authority: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.mint == staking_pool.token_mint,
        constraint = user_token_account.owner == user_authority.key()
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut,
    constraint = pool_token_account.key() == staking_pool.pool_token_account
    )]
    pub pool_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(stake_id:u64)]
pub struct Unstake<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        mut,
        seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,
 #[account(mut,
    constraint = mint.key() == config_account.config.char_token_mint)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"user".as_ref(),  user_authority.key().as_ref()],
        bump = user.bump,
        constraint = user.authority == user_authority.key(),
        constraint = user.staking_pool == staking_pool.key()
    )]
    pub user: Account<'info, UserStakeInfo>,
    #[account(
        mut,
        seeds = [b"user_stake".as_ref(), user_authority.key().as_ref(),stake_id.to_le_bytes().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStakesEntry>,
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(
        mut,
        constraint = staking_reward_ata.owner == staking_pool.staking_reward_account.key(),
        constraint = staking_reward_ata.mint == staking_pool.token_mint
    )]
    pub staking_reward_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_token_account.mint == staking_pool.token_mint,
        constraint = user_token_account.owner == user_authority.key()
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = pool_token_account.key() == staking_pool.pool_token_account
    )]
    pub pool_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(stake_id:u64)]
pub struct UnstakeRequest<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        mut,
        seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(
        mut,
        seeds = [b"user_stake".as_ref(), user_authority.key().as_ref(),stake_id.to_le_bytes().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStakesEntry>,
 

    #[account(mut)]
    pub user_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(stake_id:u64)]
pub struct ClaimReward<'info> {
    #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"user".as_ref(), user_authority.key().as_ref()],
        bump = user.bump,
        constraint = user.authority == user_authority.key(),
        constraint = user.staking_pool == staking_pool.key()
    )]
    pub user: Account<'info, UserStakeInfo>,
    #[account(
        seeds = [b"user_stake".as_ref(), user_authority.key().as_ref(),stake_id.to_le_bytes().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStakesEntry>,
    #[account(mut)]
    pub user_authority: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.mint == staking_pool.token_mint,
        constraint = user_token_account.owner == user_authority.key()
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = staking_reward_ata.owner == staking_pool.staking_reward_account.key(),
        constraint = staking_reward_ata.mint == staking_pool.token_mint

    )]
    pub staking_reward_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"staking_reward".as_ref(),staking_pool.token_mint.key().as_ref()],
        bump
    )]
    pub staking_reward: Account<'info, StakingRewards>,
 #[account(mut,
    constraint = mint.key() == config_account.config.char_token_mint)]
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Clone, Copy, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct LockupReward {
    pub lockup_days: u16, // Number of days for lockup
    pub reward_bps: u16,  // Reward percentage in basis points (500 = 5%)
    pub vote_power: u16,
}

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub pool_token_account: Pubkey,
    pub staking_reward_account: Pubkey,
    pub total_staked: u64,
    pub reward_issued: u64,
    pub bump: u8,
    pub stake_lockup_reward_array: [LockupReward; 4],
}

#[account]
pub struct StakingRewards {}

#[account]
pub struct UserStakeInfo {
    pub authority: Pubkey,
    pub staking_pool: Pubkey,

    pub eligible_at: u64, 
    pub voting_power: u64, 

    pub total_amount: u64, // total staked amount of user
    pub reward_issued: u64,// total reward amount issued claimed by user
    pub stake_count: u64,
    pub bump: u8,
    pub last_vote_time:u64
}

#[account]
pub struct UserStakesEntry {
    pub stake_id: u64,
    pub amount: u64,
    pub staked_at: u64,
    pub lockup: u16,
    pub unstake_requested_at: u64,
    pub current_period: u64,
    pub unstaked_at: u64,
}
