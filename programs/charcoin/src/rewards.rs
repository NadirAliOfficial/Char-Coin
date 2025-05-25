use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer};

#[error_code]
pub enum FundReleaseError {
    #[msg("Invalid distribution percentages, must sum to 100.")]
    InvalidPercentage,
}

#[derive(Accounts)]
pub struct ReleaseMonthlyFunds<'info> {
    /// CHECK: Treasury token account holding funds to be distributed.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,
    /// CHECK: Destination token account for staking rewards (15%).
    #[account(mut)]
    pub staking_rewards: UncheckedAccount<'info>,
    /// CHECK: Destination token account for monthly rewards (7.5%).
    #[account(mut)]
    pub monthly_reward: UncheckedAccount<'info>,
    /// CHECK: Destination token account for annual rewards (7.5%).
    #[account(mut)]
    pub annual_reward: UncheckedAccount<'info>,
    /// CHECK: Destination token account for immediate monthly donations (48%).
    #[account(mut)]
    pub monthly_donation: UncheckedAccount<'info>,
    /// CHECK: Destination token account for annual reserved donations (12%).
    #[account(mut)]
    pub annual_charity: UncheckedAccount<'info>,
    /// Authority for treasury withdrawals.
    pub treasury_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn release_monthly_funds(ctx: Context<ReleaseMonthlyFunds>, total_amount: u64) -> Result<()> {
    // Fixed distribution percentages from the CHAR Coin schema
    let staking_percent = 150; // 15% to staking rewards
    let donation_percent = 750; // 75% to donation ecosystem

    // Calculate staking amount (15% of total)
    let staking_amount = total_amount
        .checked_mul(staking_percent as u64)
        .unwrap()
        .checked_div(1000)
        .unwrap();

    // Calculate donation ecosystem total (75% of total)
    let donation_total = total_amount
        .checked_mul(donation_percent as u64)
        .unwrap()
        .checked_div(1000)
        .unwrap();

    // Split donation into subcategories (Donation System)
    let monthly_reward_amount = donation_total
        .checked_mul(10)
        .unwrap()
        .checked_div(100)
        .unwrap(); // 10% of donation 
    let annual_reward_amount = donation_total
        .checked_mul(10)
        .unwrap()
        .checked_div(100)
        .unwrap(); // 10% of donation 
    let monthly_donation_amount = donation_total
        .checked_mul(80)
        .unwrap()
        .checked_div(100)
        .unwrap(); // 80% of donation

    // Split monthly donation into immediate and reserved portions
    let monthly_donation_immediate = monthly_donation_amount
        .checked_mul(80)
        .unwrap()
        .checked_div(100)
        .unwrap(); // 80% of monthly donation (48% total)
    let monthly_donation_reserved = monthly_donation_amount
        .checked_mul(20)
        .unwrap()
        .checked_div(100)
        .unwrap(); // 20% of monthly donation (12% total)

    // Transfer to staking rewards
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.staking_rewards.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        staking_amount,
    )?;

    // Transfer to monthly reward fund
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.monthly_reward.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        monthly_reward_amount,
    )?;

    // Transfer to annual reward fund
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.annual_reward.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        annual_reward_amount,
    )?;

    // Transfer to immediate monthly donation
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.monthly_donation.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        monthly_donation_immediate,
    )?;

    // Transfer reserved portion to annual charity
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.annual_charity.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        monthly_donation_reserved,
    )?;

    msg!(
        "Monthly funds released: Staking={}, Monthly Reward={}, Annual Reward={}, Monthly Donation={}, Annual Reserved={}",
        staking_amount,
        monthly_reward_amount,
        annual_reward_amount,
        monthly_donation_immediate,
        monthly_donation_reserved
    );
    Ok(())
}

#[derive(Accounts)]
pub struct ReleaseAnnualFunds<'info> {
    /// CHECK: Treasury token account holding annual funds.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,
    /// CHECK: Destination token account for annual charity funds.
    #[account(mut)]
    pub annual_charity: UncheckedAccount<'info>,
    /// Authority for treasury withdrawals.
    pub treasury_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn release_annual_funds(ctx: Context<ReleaseAnnualFunds>, annual_amount: u64) -> Result<()> {
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.annual_charity.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        annual_amount,
    )?;
    msg!("Annual funds released: {}", annual_amount);
    Ok(())
}
