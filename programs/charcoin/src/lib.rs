
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};

// Import our modules.
pub mod burn;
pub mod staking;
pub use staking::*;


pub mod governance;
pub use governance::*;

// pub use governance::*;
mod rewards;
mod donation;
mod security;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod char_coin {
    use super::*;

    /// Initialize the global configuration for CHAR Coin.
    pub fn initialize(ctx: Context<Initialize>, config: Config) -> Result<()> {
        let config_account = &mut ctx.accounts.config;
        config_account.config = config;
        msg!(
            "CHAR Coin contract initialized with token supply: {}",
            config_account.config.token_supply
        );
        Ok(())
    }


     /// Stake tokens with a specified lockup duration.
     pub fn stake_tokens_handler(
        ctx: Context<StakeTokens>,
        amount: u64,
        lockup: LockupPeriod,
    ) -> Result<()> {
        staking::stake_tokens(ctx, amount, lockup)
    }

    /// Unstake tokens after the lockup period has expired.
    pub fn unstake_tokens_handler(ctx: Context<UnstakeTokens>) -> Result<()> {
        staking::unstake_tokens(ctx)
    }
    
    /// Mint new tokens.
    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.destination.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // Mint the tokens.
        token::mint_to(cpi_ctx, amount)?;

        msg!("Minted {} tokens", amount);
        emit!(SupplyUpdated { new_supply: amount });
        Ok(())
    }

    /// Transfer tokens with fee deduction.
    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        // Calculate fee ( 1% fee).
        let fee = amount / 100;
        let transfer_amount = amount.checked_sub(fee).ok_or(ErrorCode::MathError)?;

        // Transfer fee to the fee collection account.
        let cpi_accounts_fee = token::Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.fee_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_fee = CpiContext::new(cpi_program.clone(), cpi_accounts_fee);
        token::transfer(cpi_ctx_fee, fee)?;

        // Transfer remaining tokens to the destination.
        let cpi_accounts_transfer = token::Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.destination.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_ctx_transfer = CpiContext::new(cpi_program, cpi_accounts_transfer);
        token::transfer(cpi_ctx_transfer, transfer_amount)?;

        msg!(
            "Transferred {} tokens with a fee of {} deducted",
            transfer_amount,
            fee
        );
        Ok(())
    }

    /// Burn tokens using the burn module.
    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        burn::process_burn(ctx, amount)
    }

    // Governance
    pub fn create_governance_proposal(
        ctx: Context<SubmitProposal>, // Use struct directly
        title: String,
        description: String,
        duration: i64
    ) -> Result<()> {
        governance::submit_proposal(ctx, title, description, duration)
    }

    pub fn cast_vote(
        ctx: Context<VoteOnProposal>, // Use struct directly
        proposal_id: u64,
        vote_choice: bool,
        amount_staked: u64
    ) -> Result<()> {
        governance::vote_on_proposal(ctx, proposal_id, vote_choice, amount_staked)
    }

    pub fn conclude_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
        governance::finalize_proposal(ctx)
    }
}
/// Event to log supply updates.
#[event]
pub struct SupplyUpdated {
    pub new_supply: u64,
}

/// Global configuration for tokenomics, staking, and governance.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Config {
    pub token_supply: u64,
    pub fee_percentage: u8,         // 1 for 1%
    pub buyback_percentage: u8,     // 10% of the fee allocation for buyback
    pub donation_percentage: u8,    // 75% for donation ecosystem
    pub staking_percentage: u8,     // 15% for staking rewards
    // Additional configuration parameters can be added here.
}

/// Account to store the global configuration.
#[account]
pub struct ConfigAccount {
    pub config: Config,
}

/// Context for the `initialize` instruction.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 256)]
    pub config: Account<'info, ConfigAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Context for minting tokens.
///
/// We use raw `AccountInfo` for SPL token accounts because they do not implement
/// Anchor’s `Discriminator`. The CHECK comments explain why these raw accounts are safe.
#[derive(Accounts)]
pub struct MintTokens<'info> {
    /// CHECK: This is the token mint account.
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    /// CHECK: This is the destination token account.
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    /// CHECK: This is the mint authority; its validity is checked off-chain or by program logic.
    #[account(signer)]
    pub mint_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

/// Context for transferring tokens.
#[derive(Accounts)]
pub struct TransferTokens<'info> {
    /// CHECK: Source token account.
    #[account(mut)]
    pub from: AccountInfo<'info>,
    /// CHECK: Destination token account.
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    /// CHECK: Fee collection account.
    #[account(mut)]
    pub fee_account: AccountInfo<'info>,
    /// CHECK: This is the token owner's account.
    #[account(signer)]
    pub owner: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

/// Context for burning tokens.
#[derive(Accounts)]
pub struct BurnTokens<'info> {
    /// CHECK: Token mint.
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    /// CHECK: Token account from which tokens will be burned.
    #[account(mut)]
    pub account: AccountInfo<'info>,
    /// CHECK: Owner of the token account.
    #[account(signer)]
    pub owner: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("An arithmetic error occurred.")]
    MathError,
}