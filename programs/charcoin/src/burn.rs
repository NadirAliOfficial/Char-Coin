use crate::{ConfigAccount, CustomError};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token_interface::{Mint, TokenAccount};
use anchor_spl::token_2022::{Burn, burn, Token2022 as Token};

#[derive(Accounts)]
pub struct ExecuteBuyback<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(mut,
    constraint = mint.key() == config_account.config.char_token_mint)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
            constraint = burn_wallet_ata.owner.key() ==  config_account.config.death_wallet,
            constraint = burn_wallet_ata.mint.key() ==  config_account.config.char_token_mint
    )]
    pub burn_wallet_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = config_account.config.death_wallet == burn_authority.key() 
    )]
    pub burn_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}


// this function will be run in the backend inside a cron job
pub fn execute_buyback(ctx: Context<ExecuteBuyback>) -> Result<()> {
    let tokens_to_buy = ctx.accounts.burn_wallet_ata.amount;
    require!(tokens_to_buy > 0, CustomError::NoTokensToBuyback);
    // Burn tokens from the burn_wallet.
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.burn_wallet_ata.to_account_info(),
            authority: ctx.accounts.burn_authority.to_account_info(),
        },
    );
    burn(burn_ctx, tokens_to_buy)?;

    // Update the burn tracker.
    let tracker = &mut ctx.accounts.config_account.config;
    tracker.total_burned += tokens_to_buy;

    // Cache the current timestamp.
    let current_time = Clock::get()?.unix_timestamp as u64;

    // Emit an event logging the buyback and burn details.
    emit!(BuybackBurnEvent {
        tokens_bought: tokens_to_buy,
        new_total_burned: tracker.total_burned,
        timestamp: current_time,
    });

    Ok(())
}

#[event]
pub struct BuybackBurnEvent {
    pub tokens_bought: u64,
    pub new_total_burned: u64,
    pub timestamp: u64,
}
