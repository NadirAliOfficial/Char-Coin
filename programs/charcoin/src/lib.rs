#![allow(unexpected_cfgs)]
#[allow(ambiguous_glob_reexports)]
use anchor_lang::prelude::*;

// Modules
pub mod burn;
pub mod donation;
pub mod governance;
pub mod marketing;
pub mod rewards;
pub mod security;
pub mod staking;

use anchor_spl::token::Mint;
// Re-export public items
pub use donation::*;
pub use burn::*;
pub use governance::*;
pub use marketing::*;
pub use rewards::*;
pub use security::*;
pub use staking::*;

declare_id!("keyk1FHhXGs82vJ2qecM8Rc96PXVvArVtNfoC8xEyKN");

#[program]
pub mod charcoin {
    use super::*;

    /// Initializes the global configuration.
    pub fn initialize(ctx: Context<Initialize>, config: Config) -> Result<()> {
        let config_account = &mut ctx.accounts.config;
        config_account.config = config;

        // Validate wallets
        require!(
            config_account.config.monthly_reward_wallet != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );
        require!(
            config_account.config.annual_reward_wallet != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );

        require!(
            config_account.config.monthly_donation_wallet != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );
        require!(
            config_account.config.annual_donation_wallet != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );
        require!(
            config_account.config.chai_funds != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );

        require!(
            config_account.config.marketing_wallet_1 != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );
           require!(
            config_account.config.marketing_wallet_2 != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );

        require!(
            config_account.config.admin != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );

            require!(
            config_account.config.treasury_authority != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );

        require!(
            config_account.config.death_wallet != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );



        msg!("CHAR Coin initialized with donation wallets configured");
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
    pub fn initialize_treasury_handler(
        ctx: Context<InitializeTreasury>,
        owners: Vec<Pubkey>,
        threshold: u8,
    ) -> Result<()> {
        governance::initialize_treasury(ctx, owners, threshold)
    }

    // Staking
    /// Stake tokens with a specified lockup duration.
    pub fn stake_tokens_handler(ctx: Context<Stake>, amount: u64, lockup: u64) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        staking::stake_tokens(ctx, amount, lockup)
    }

    /// Unstake tokens after 48h delay and lockup period has expired. unstake before lockup period will result in penalty
    pub fn unstake_tokens_handler(ctx: Context<Unstake>,index:u64) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );

        staking::unstake_tokens(ctx,index)
    }

    /// request Unstake tokens.
    pub fn request_unstake_handler(ctx: Context<UnstakeRequest>,index:u64) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );

        staking::request_unstake_tokens(ctx,index)
    }

    pub fn claim_reward_handler(ctx: Context<ClaimReward>,index:u64) -> Result<()> {
        require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );

        staking::claim_reward(ctx,index)
    }

    // Burning 
    pub fn buyback_burn_handler(
    ctx: Context<ExecuteBuyback>,
    fee_amount: u64,
    conversion_rate: u64,
    ) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        burn::execute_buyback(ctx, fee_amount, conversion_rate)
    } 
    // Governance
    // Governance functions
    pub fn submit_proposal_handler(
        ctx: Context<SubmitProposal>,
        title: String,
        description: String,
        duration: i64,
    ) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        governance::submit_proposal(ctx, title, description, duration)
    }

    pub fn vote_on_proposal_handler(
        ctx: Context<VoteOnProposal>,
        proposal_id: u64,
        vote_choice: bool,
    ) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        governance::vote_on_proposal(ctx, proposal_id, vote_choice)
    }

    pub fn finalize_proposal_handler(ctx: Context<FinalizeProposal>) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        governance::finalize_proposal(ctx)
    }



    pub fn create_withdrawal_handler(
        ctx: Context<CreateWithdrawal>,
        amount: u64,
        recipient: Pubkey,
    ) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        governance::create_withdrawal(ctx, amount, recipient)
    }

    pub fn approve_withdrawal_handler(ctx: Context<ApproveWithdrawal>) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        governance::approve_withdrawal(ctx)
    }

    pub fn execute_withdrawal_handler(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        governance::execute_withdrawal(ctx)
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
        name: String,
        description: String,
        wallet: Pubkey,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        donation::register_charity(ctx, name, description, wallet, start_time, end_time)
    }

    /// Casts or updates a vote for a charity.
    pub fn cast_vote_handler(ctx: Context<CastVote>, vote_weight: u64) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        donation::cast_vote(ctx, vote_weight)
    }

    /// Finalizes charity voting after the voting period ends.
    pub fn finalize_charity_vote_handler(ctx: Context<FinalizeCharityVote>) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        donation::finalize_charity_vote(ctx)
    }

    //  Rewards
    /// Releases funds from the treasury to staking rewards and charity fund.
    pub fn release_funds_handler(
        ctx: Context<ReleaseMonthlyFunds>,
        total_amount: u64,
    ) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        rewards::release_funds(ctx, total_amount)
    }

    // Marketing
    pub fn distribute_marketing_funds_handler(
        ctx: Context<DistributeMarketingFunds>,
        total_amount: u64,
    ) -> Result<()> {
          require!(
            ctx.accounts.config_account.config.halted == false,
            ErrorCode::ProgramIsHalted
        );
        marketing::distribute_marketing_funds(ctx, total_amount)
    }
  
}

/// Stores global configuration for CHAR Coin.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Config {
    // The fields below are used in your code, so we add them to avoid errors.
    pub admin: Pubkey,

    pub monthly_reward_wallet: Pubkey,
    pub annual_reward_wallet: Pubkey,
    pub monthly_donation_wallet: Pubkey,
    pub annual_donation_wallet: Pubkey,
    pub chai_funds: Pubkey,
    pub marketing_wallet_1: Pubkey,
    pub marketing_wallet_2: Pubkey,
    pub death_wallet: Pubkey,
    pub treasury_authority: Pubkey, // fee 
    /// emergency state that indicates if the contract is halted.
    pub halted: bool,
    pub next_proposal_id:u64,
    pub next_charity_id:u64,
    pub next_staking_id:u64,
    pub total_burned: u64,

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
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Custom error definitions.
#[error_code]
pub enum ErrorCode {
    #[msg("An arithmetic error occurred.")]
    MathError,
    #[msg("Unauthorized operation.")]
    Unauthorized,
    #[msg("Invalid wallet configuration")]
    InvalidConfiguration,
    #[msg("program is halted")]
    ProgramIsHalted,
}
