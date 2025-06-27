#![allow(unexpected_cfgs)]
#[allow(ambiguous_glob_reexports)]
use anchor_lang::prelude::*;

// Modules
pub mod burn;
pub mod donation;
pub mod marketing;
pub mod rewards;
pub mod security;
pub mod staking;
pub mod errors;

// Re-export public items
pub use burn::*;
pub use donation::*;
pub use marketing::*;
pub use rewards::*;
pub use security::*;
pub use staking::*;
pub use errors::*;

declare_id!("aUvFTHYrF4N6vpyC5DnkWNXqahcGcDknEScKeoEuANt");

#[program]
pub mod charcoin {
    use super::*;

    /// Initializes the global configuration.
    pub fn initialize(ctx: Context<Initialize>, config: Config) -> Result<()> {
        let config_account = &mut ctx.accounts.config;
        config_account.config = config;

        Ok(())
    }

    pub fn staking_initialize(ctx: Context<StakeInitialize>) -> Result<()> {
        let staking_pool = &mut ctx.accounts.staking_pool;
        staking_pool.authority = ctx.accounts.authority.key();
        staking_pool.token_mint = ctx.accounts.token_mint.key();
        staking_pool.pool_token_account = ctx.accounts.pool_token_account.key();
        staking_pool.staking_reward_account = ctx.accounts.staking_reward.key();
        staking_pool.bump = ctx.bumps.staking_pool;
        Ok(())
    }

    // Staking
    /// Stake tokens with a specified lockup duration.
    pub fn stake_tokens_handler(ctx: Context<Stake>, amount: u64, lockup: u16) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        staking::stake_tokens(ctx, amount, lockup)
    }

    /// Unstake tokens after 48h delay and lockup period has expired. unstake before lockup period will result in penalty
    pub fn unstake_tokens_handler(ctx: Context<Unstake>, index: u64) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );

        staking::unstake_tokens(ctx, index)
    }

    /// request Unstake tokens.
    pub fn request_unstake_handler(ctx: Context<UnstakeRequest>, index: u64) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );

        staking::request_unstake_tokens(ctx, index)
    }

    pub fn claim_reward_handler(ctx: Context<ClaimReward>, index: u64) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );

        staking::claim_reward(ctx, index)
    }

    // Burning
    pub fn buyback_burn_handler(ctx: Context<ExecuteBuyback>) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        burn::execute_buyback(ctx)
    }
   
  

    // Emergency halt

    pub fn change_emergency_state_handler(
        ctx: Context<InitializeEmergencyState>,
        state: bool,
    ) -> Result<()> {
        security::change_emergency_state(ctx, state)
    }

    // Donation
    /// Registers a new charity for the donation ecosystem.
    pub fn register_charity_handler(
        ctx: Context<RegisterCharity>,
        title: String,
        wallet: Pubkey,
        start_time: u64,
        end_time: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        donation::register_charity(ctx, title, wallet, start_time, end_time)
    }

    /// Casts or updates a vote for a charity.
    pub fn cast_vote_handler(ctx: Context<CastVote>, charity_id: u64,char_points:u64) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        donation::cast_vote(ctx, charity_id,char_points)
    }

    /// Finalizes charity voting after the voting period ends.
    pub fn finalize_charity_vote_handler(
        ctx: Context<FinalizeCharityVote>,
        charity_id: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        donation::finalize_charity_vote(ctx, charity_id)
    }

    //  Rewards
    /// Releases funds from the treasury to staking rewards and charity fund.
    pub fn release_rewards_handler(
        ctx: Context<ReleaseRewards>,
        total_amount: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        rewards::release_rewards(ctx, total_amount)
    }
    pub fn release_donations_handler(
        ctx: Context<ReleaseDonations>,
        total_amount: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        rewards::release_donations(ctx, total_amount)
    }
    pub fn release_staking_funds_handler(
        ctx: Context<ReleaseStakingFunds>,
        total_amount: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        rewards::release_staking_char_funds(ctx, total_amount)
    }

    // Marketing
    pub fn distribute_marketing_funds_handler(
        ctx: Context<DistributeMarketingFunds>,
        total_amount: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        marketing::distribute_marketing_funds(ctx, total_amount)
    }

    pub fn update_settings(
        ctx: Context<Settings>,
        min_governance_stake: u64,
        min_stake_duration_voting: u64,
        early_unstake_penalty: u64,
    ) -> Result<()> {
          require!(
            ctx.accounts.config.config.halted == false,
            CustomError::ProgramIsHalted
        );
        let config = &mut ctx.accounts.config;
        config.config.min_governance_stake = min_governance_stake;
        config.config.min_stake_duration_voting = min_stake_duration_voting;
        config.config.early_unstake_penalty = early_unstake_penalty;
        Ok(())
    }
    pub fn set_reward_percentage_handler(
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
          require!(
            ctx.accounts.config_account.config.halted == false,
            CustomError::ProgramIsHalted
        );
        staking::set_reward_percentage(
            ctx,
            reward1,
            lockup1,
            vote_power1,
            reward2,
            lockup2,
            vote_power2,
            reward3,
            lockup3,
            vote_power3,
            reward4,
            lockup4,
            vote_power4,
        )
    }
}

/// Stores global configuration for CHAR Coin.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Config {
    pub admin: Pubkey,
    pub char_token_mint: Pubkey, // Char Token Mint
    // Reward System
    // Monthly Rewards Classification (50%)
    pub monthly_top_tier_wallet: Pubkey,
    pub monthly_charity_lottery_wallet: Pubkey,
    // Annual Rewards Classification (50%)
    pub annual_top_tier_wallet: Pubkey,
    pub annual_charity_lottery_wallet: Pubkey,

    // Donation System (80%)
    // Monthly Donation Classification
    pub monthly_one_time_causes_wallet: Pubkey,
    pub monthly_infinite_impact_causes_wallet: Pubkey,
    // Annual Donation Classification
    pub annual_one_time_causes_wallet: Pubkey,
    pub annual_infinite_impact_causes_wallet: Pubkey,
    // Crisis Classification (10%)
    pub char_funds: Pubkey,// Char Wallet
    pub marketing_wallet_1: Pubkey,
    pub marketing_wallet_2: Pubkey,
    pub death_wallet: Pubkey,
    pub treasury_authority: Pubkey, // char Token fee wallet
    pub halted: bool, // emergency state that indicates if the contract is halted.
    
    pub next_proposal_id: u64, // id's for governance proposals
    pub next_charity_id: u64,// id's for charity donation proposals
    pub total_burned: u64, // total amount of char tokens burn by death wallet
    pub min_governance_stake: u64, // Minimum stake required to participate in governance
    pub min_stake_duration_voting: u64, // Minimum staking period required for a user to be eligible to vote
    pub early_unstake_penalty: u64,     // 100 = 10%
}

/// Account that holds the global configuration.
#[account]
pub struct ConfigAccount {
    pub config: Config,
}

/// Accounts for initialization.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, 
        seeds=[b"config".as_ref()],
         bump, space = 8 + std::mem::size_of::<ConfigAccount>())]
    pub config: Account<'info, ConfigAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Settings<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config: Account<'info, ConfigAccount>,
    #[account(
        mut,
        constraint = admin.key() == config.config.admin,

    )]
    pub admin: Signer<'info>,
}

