#![allow(unexpected_cfgs)]
#[allow(ambiguous_glob_reexports)]
use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, Burn, MintTo, Token, Transfer,
};

// Modules
pub mod security;
pub mod burn;
pub mod staking;
pub mod governance;
pub mod marketing;
pub mod private_sale;
pub mod donation;
mod rewards;

// Re-export public items
pub use security::*;
pub use burn::*;
pub use staking::*;
pub use governance::*;
pub use marketing::*;
pub use private_sale::*;
pub use donation::*;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");


#[program]
pub mod charcoin {
    use super::*;

     /// Initializes the global configuration.
    pub fn initialize(ctx: Context<Initialize>, config: Config) -> Result<()> {
        let config_account = &mut ctx.accounts.config;
        config_account.config = config;
        msg!("CHAR Coin initialized with token supply: {}", config_account.config.token_supply);
        Ok(())
    }

    /// Initializes the multisig module.
    pub fn initialize_multisig_handler(
        ctx: Context<InitializeMultisig>,  // Add module prefix here
        params: security::InitializeMultisigParams
    ) -> Result<()> {
        security::initialize_multisig(ctx, params)
    }
        

     /// Stake tokens with a specified lockup duration.
     pub fn stake_tokens_handler(ctx: Context<StakeTokens>, amount: u64, lockup: staking::LockupPeriod) -> Result<()> {
        staking::stake_tokens(ctx, amount, lockup)
    }

    /// Unstake tokens after the lockup period has expired.
    pub fn unstake_tokens_handler(ctx: Context<UnstakeTokens>) -> Result<()> {
        staking::unstake_tokens(ctx)
    }

    /// Distribute staking rewards.
    pub fn distribute_staking_rewards(ctx: Context<DistributeRewards>, reward_amount: u64, transaction_volume: u64) -> Result<()> {
        staking::distribute_rewards(ctx, reward_amount, transaction_volume)
    }
    
    /// Mints tokens (admin only).
    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        // Ensure admin calling this matches the config.
        require!(
            ctx.accounts.admin.key() == ctx.accounts.config.config.admin,
            ErrorCode::Unauthorized
        );

        // Prepare PDA signer seeds.
        let seeds: &[&[u8]] = &[b"mint_authority", &[ctx.accounts.config.config.mint_authority_bump]];
        let _signer_seeds = &[seeds];
        // Use CPI to call the SPL token program’s `mint_to`.
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;

        msg!("Minted {} tokens", amount);
        emit!(SupplyUpdated { new_supply: amount });
        Ok(())
    }

    /// Burns tokens (token owner only).
    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        // Use CPI to call the SPL token program’s `burn`.
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            amount,
        )?;
        msg!("Burned {} tokens.", amount);
        Ok(())
    }

    /// Transfers tokens with fee deduction.
    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        // For demonstration, we do a 1% fee.
        let fee = amount / 100;
        let transfer_amount = amount
            .checked_sub(fee)
            .ok_or(ErrorCode::MathError)?;

        // Transfer fee to the fee_account.
        let fee_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.fee_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(fee_ctx, fee)?;

        // Transfer remaining tokens to the destination.
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, transfer_amount)?;

        msg!(
            "Transferred {} tokens (fee {} deducted)",
            transfer_amount,
            fee
        );
        Ok(())
    }

    /// Executes a buyback & burn via the `burn` module.
    pub fn execute_buyback_handler(
        ctx: Context<ExecuteBuyback>,
        fee_amount: u64,
        conversion_rate: u64,
    ) -> Result<()> {
        // tokens_to_buy = fee_amount * conversion_rate
        let tokens_to_buy = fee_amount
            .checked_mul(conversion_rate)
            .ok_or(ErrorCode::MathError)?;
    
        // Transfer tokens from buyback_account to burn_wallet.
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyback_account.to_account_info(),
                to: ctx.accounts.burn_wallet.to_account_info(),
                authority: ctx.accounts.buyback_authority.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, tokens_to_buy)?;
    
        // Burn tokens from burn_wallet.
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.burn_wallet.to_account_info(),
                authority: ctx.accounts.burn_authority.to_account_info(),
            },
        );
        token::burn(burn_ctx, tokens_to_buy)?;
    
        // Update burn tracker.
        ctx.accounts.burn_tracker.total_burned = ctx.accounts
            .burn_tracker
            .total_burned
            .checked_add(tokens_to_buy)
            .ok_or(ErrorCode::MathError)?;
    
        // Get current timestamp.
        let clock = Clock::get()?;
    
        // Emit event with tracking details.
        emit!(BuybackBurnEvent {
            fee_amount,
            tokens_bought: tokens_to_buy,
            new_total_burned: ctx.accounts.burn_tracker.total_burned,
            timestamp: clock.unix_timestamp,
        });
    
        msg!(
            "Executed buyback: fee_amount {} resulted in burning {} tokens. Total burned: {}",
            fee_amount,
            tokens_to_buy,
            ctx.accounts.burn_tracker.total_burned
        );
        Ok(())
    }
    
    

    // Governance
    // Governance functions
    pub fn submit_proposal_handler(
        ctx: Context<SubmitProposal>,
        title: String,
        description: String,
        duration: i64,
    ) -> Result<()> {
        governance::submit_proposal(ctx, title, description, duration)
    }
    
    pub fn vote_on_proposal_handler(
        ctx: Context<VoteOnProposal>,
        proposal_id: u64,
        vote_choice: bool,
        amount_staked: u64,
    ) -> Result<()> {
        governance::vote_on_proposal(ctx, proposal_id, vote_choice, amount_staked)
    }
    

    pub fn finalize_proposal_handler(ctx: Context<FinalizeProposal>) -> Result<()> {
        governance::finalize_proposal(ctx)
    }
    // Marketing 
    pub fn distribute_marketing_funds_handler(ctx: Context<DistributeMarketingFunds>) -> Result<()> {
        marketing::distribute_marketing_funds(ctx)
    }
    
    // Private Sale 
      /// Initializes a vesting account for a private sale investor.
    pub fn initialize_vesting_handler(ctx: Context<InitializeVesting>, locked_amount: u64) -> Result<()> {
        private_sale::initialize_vesting(ctx, locked_amount)
    }
    
    /// Deposits funds into the private sale vault.
    pub fn deposit_funds_handler(ctx: Context<DepositFunds>, deposit_amount: u64) -> Result<()> {
        private_sale::deposit_funds(ctx, deposit_amount)
    }
    
    /// Allows an investor to claim vested tokens after the vesting period.
    pub fn claim_tokens_handler(ctx: Context<ClaimTokens>, sale_token_amount: u64) -> Result<()> {
        private_sale::claim_tokens(ctx, sale_token_amount)
    }

    // Donation 
     /// Registers a new charity for the donation ecosystem.
     pub fn register_charity_handler(
        ctx: Context<RegisterCharity>,
        id: u64,
        name: String,
        description: String,
        wallet: Pubkey,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
        donation::register_charity(ctx, id, name, description, wallet, start_time, end_time)
    }

    /// Casts or updates a vote for a charity.
    pub fn cast_vote_handler(ctx: Context<CastVote>, vote_weight: u64) -> Result<()> {
        donation::cast_vote(ctx, vote_weight)
    }

    /// Finalizes charity voting after the voting period ends.
    pub fn finalize_charity_vote_handler(ctx: Context<FinalizeCharityVote>) -> Result<()> {
        donation::finalize_charity_vote(ctx)
    }

    // Emergency halt
    pub fn emergency_halt_handler(ctx: Context<EmergencyHalt>) -> Result<()> {
        security::emergency_halt(ctx)
    }

    pub fn emergency_unhalt_handler(ctx: Context<EmergencyUnhalt>) -> Result<()> {
        security::emergency_unhalt(ctx)
    }

}

/// Stores global configuration for CHAR Coin.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Config {
    pub token_supply: u64,
    pub fee_percentage: u8,
    pub buyback_percentage: u8,
    pub donation_percentage: u8,
    pub staking_percentage: u8,
    // The fields below are used in your code, so we add them to avoid errors.
    pub admin: Pubkey,
    pub mint_authority_bump: u8,
}

/// Account that holds the global configuration.
#[account]
pub struct ConfigAccount {
    pub config: Config,
}

/// Accounts for initialization.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 256)]
    pub config: Account<'info, ConfigAccount>,
    /// CHECK: SPL Token mint account; its data is managed by the token program.
    pub mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Accounts for minting tokens.
#[derive(Accounts)]
pub struct MintTokens<'info> {
    /// CHECK: SPL Token mint account.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    /// CHECK: Destination token account.
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    /// CHECK: PDA mint authority.
    pub mint_authority: UncheckedAccount<'info>,
    #[account(address = config.config.admin)]
    pub admin: Signer<'info>,
    pub config: Account<'info, ConfigAccount>,
    pub token_program: Program<'info, Token>,
}

/// Accounts for burning tokens.
#[derive(Accounts)]
pub struct BurnTokens<'info> {
    /// CHECK: SPL Token mint account.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    /// CHECK: Token account from which tokens will be burned.
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: Owner of the token account.
    pub owner: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

/// Accounts for transferring tokens.
#[derive(Accounts)]
pub struct TransferTokens<'info> {
    /// CHECK: Source token account.
    #[account(mut)]
    pub from: UncheckedAccount<'info>,
    /// CHECK: Fee collection account.
    #[account(mut)]
    pub fee_account: UncheckedAccount<'info>,
    /// CHECK: Destination token account.
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    /// CHECK: Owner of the token account.
    pub owner: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}


/// Event to log supply updates.
#[event]
pub struct SupplyUpdated {
    pub new_supply: u64,
}

/// Custom error definitions.
#[error_code]
pub enum ErrorCode {
    #[msg("An arithmetic error occurred.")]
    MathError,
    #[msg("Unauthorized operation.")]
    Unauthorized,
}
