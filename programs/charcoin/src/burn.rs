use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Token, Transfer};
use crate::BurnTracker; 
use crate::ErrorCode;

#[derive(Accounts)]
pub struct ExecuteBuyback<'info> {
    /// CHECK: SPL Token mint account.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    /// CHECK: Account holding tokens accumulated from fees.
    #[account(mut)]
    pub buyback_account: UncheckedAccount<'info>,
    pub buyback_authority: Signer<'info>,
    /// CHECK: Burn wallet (a locked account).
    #[account(mut)]
    pub burn_wallet: UncheckedAccount<'info>,
    /// CHECK: Authority to burn tokens from the burn wallet (typically a PDA).
    pub burn_authority: UncheckedAccount<'info>,
    /// This account tracks total tokens burned.
    #[account(mut)]
    pub burn_tracker: Account<'info, BurnTracker>,
    pub token_program: Program<'info, Token>,
}



pub fn execute_buyback(
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

    // Burn tokens from the burn_wallet.
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.burn_wallet.to_account_info(),
            authority: ctx.accounts.burn_authority.to_account_info(),
        },
    );
    token::burn(burn_ctx, tokens_to_buy)?;

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
