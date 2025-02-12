// programs/char_coin/src/burn.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn};

use crate::BurnTokens;

pub fn process_burn(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
    let cpi_accounts = Burn {
        mint: ctx.accounts.mint.clone(),
        from: ctx.accounts.account.clone(),
        authority: ctx.accounts.owner.clone(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Call the SPL token burn function.
    token::burn(cpi_ctx, amount)?;

    msg!("Burned {} tokens", amount);
    Ok(())
}