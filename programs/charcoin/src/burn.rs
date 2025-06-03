use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{Mint, TokenAccount};
use anchor_spl::token::{self, Burn, Token};
use crate::ConfigAccount;
use crate::ErrorCode;

#[derive(Accounts)]
pub struct ExecuteBuyback<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
      )]    
    pub config_account: Account<'info, ConfigAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub burn_wallet_ata: Account<'info,TokenAccount>,
    #[account(mut)]
    pub burn_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn execute_buyback(
    ctx: Context<ExecuteBuyback>,
    fee_amount: u64,
    conversion_rate: u64,
) -> Result<()> {
    // Calculate the number of tokens to buy back.
    let tokens_to_buy = fee_amount
        .checked_mul(conversion_rate)
        .ok_or(ErrorCode::MathError)?;



    // Burn tokens from the burn_wallet.
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.burn_wallet_ata.to_account_info(),
            authority: ctx.accounts.burn_authority.to_account_info(),
        },
    );
    token::burn(burn_ctx, tokens_to_buy)?;

    // Update the burn tracker.
    let tracker = &mut ctx.accounts.config_account.config;
    tracker.total_burned += tokens_to_buy;

    // Cache the current timestamp.
    let current_time = Clock::get()?.unix_timestamp;

    // Emit an event logging the buyback and burn details.
    emit!(BuybackBurnEvent {
        fee_amount,
        tokens_bought: tokens_to_buy,
        new_total_burned: tracker.total_burned,
        timestamp: current_time,
    });
    msg!(
        "Executed buyback: fee_amount {} resulted in burning {} tokens.",
        fee_amount,
        tokens_to_buy
    );
    Ok(())
}

#[event]
pub struct BuybackBurnEvent {
    pub fee_amount: u64,
    pub tokens_bought: u64,
    pub new_total_burned: u64,
    pub timestamp: i64,
}
