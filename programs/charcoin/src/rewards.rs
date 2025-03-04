use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer};

#[error_code]
pub enum FundReleaseError {
    #[msg("Invalid distribution percentages, must sum to 100.")]
    InvalidPercentage,
}

#[derive(Accounts)]
pub struct ReleaseMonthlyFunds<'info> {
    /// CHECK: This is the treasury token account holding funds to be distributed. 
    /// Its data is validated by the SPL token program.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,
    /// Destination token account for staking rewards.
    /// CHECK: This is the staking rewards token account.
    #[account(mut)]
    pub staking_rewards:  UncheckedAccount<'info>,
    /// Destination token account for charity funds.
    /// CHECK: This is the charity token account.
    #[account(mut)]
    pub charity_fund:  UncheckedAccount<'info>,
    /// Authority for treasury withdrawals (typically a PDA or multisig signer).
    pub treasury_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn release_monthly_funds(
    ctx: Context<ReleaseMonthlyFunds>,
    total_amount: u64,
    staking_pct: u8,
    charity_pct: u8,
) -> Result<()> {
    // Ensure percentages add up to 100.
    require!(
        (staking_pct as u64).checked_add(charity_pct as u64).unwrap() == 100,
        FundReleaseError::InvalidPercentage
    );

    let staking_amount = total_amount
        .checked_mul(staking_pct as u64)
        .unwrap()
        .checked_div(100)
        .unwrap();
    let charity_amount = total_amount
        .checked_mul(charity_pct as u64)
        .unwrap()
        .checked_div(100)
        .unwrap();

    // Transfer funds to the staking rewards account.
    {
        let cpi_accounts = Transfer {
            from: ctx.accounts.treasury.to_account_info(),
            to: ctx.accounts.staking_rewards.to_account_info(),
            authority: ctx.accounts.treasury_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, staking_amount)?;
    }

    // Transfer funds to the charity fund account.
    {
        let cpi_accounts = Transfer {
            from: ctx.accounts.treasury.to_account_info(),
            to: ctx.accounts.charity_fund.to_account_info(),
            authority: ctx.accounts.treasury_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, charity_amount)?;
    }
    
    msg!("Monthly funds released: {} to staking rewards and {} to charity fund", staking_amount, charity_amount);
    Ok(())
}

#[derive(Accounts)]
pub struct ReleaseAnnualFunds<'info> {
    /// CHECK: This is the treasury token account holding annual funds.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,
    /// Destination token account for annual charity funds.
    /// CHECK: This is the annual charity token account.
    #[account(mut)]
    pub annual_charity:  UncheckedAccount<'info>,
    /// Authority for treasury withdrawals.
    pub treasury_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn release_annual_funds(ctx: Context<ReleaseAnnualFunds>, annual_amount: u64) -> Result<()> {
    let cpi_accounts = Transfer {
        from: ctx.accounts.treasury.to_account_info(),
        to: ctx.accounts.annual_charity.to_account_info(),
        authority: ctx.accounts.treasury_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, annual_amount)?;
    
    msg!("Annual funds released: {} to annual charity fund", annual_amount);
    Ok(())
}
