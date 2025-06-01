use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{ConfigAccount, StakingPool};



#[derive(Accounts)]
pub struct ReleaseMonthlyFunds<'info> {
     #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]
    pub config_account: Account<'info, ConfigAccount>,
 #[account(
        seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,
    /// CHECK: Treasury token account holding funds to be distributed.
    #[account(mut,
    constraint = treasury_ata.owner == config_account.config.treasury_authority,
    )]
    pub treasury_ata: Account<'info, TokenAccount>,
    /// CHECK: Destination token account for staking rewards (15%).
    #[account(mut,
            constraint = staking_reward_ata.owner == staking_pool.staking_reward_account
    )]
    pub staking_reward_ata: Account<'info, TokenAccount>,
    /// CHECK: Destination token account for monthly rewards (7.5%).
    #[account(mut,
    constraint = monthly_reward_ata.owner == config_account.config.monthly_reward_wallet,
    )]
    pub monthly_reward_ata: Account<'info, TokenAccount>,
    /// CHECK: Destination token account for annual rewards (7.5%).
    #[account(mut,
    constraint = annual_reward_ata.owner == config_account.config.annual_reward_wallet,
    )]
    pub annual_reward_ata: Account<'info, TokenAccount>,
    /// CHECK: Destination token account for immediate monthly donations (48%).
    #[account(mut,
    constraint = monthly_donation_ata.owner == config_account.config.monthly_donation_wallet,
    
    )]
    pub monthly_donation_ata: Account<'info, TokenAccount>,
    /// CHECK: Destination token account for annual reserved donations (12%).
    #[account(mut,
    constraint = annual_donation_ata.owner == config_account.config.annual_donation_wallet,
    )]
    pub annual_donation_ata: Account<'info, TokenAccount>,

    /// CHECK: Destination token account for annual reserved donations (12%).
    #[account(mut,
    constraint = chai_funds_ata.owner == config_account.config.chai_funds,
    )]
    pub chai_funds_ata: Account<'info, TokenAccount>,
    /// Authority for treasury withdrawals.
    #[account(
        mut,
        constraint = config_account.config.treasury_authority == treasury_authority.key()
    )]
    pub treasury_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn release_funds(ctx: Context<ReleaseMonthlyFunds>, total_amount: u64) -> Result<()> {
    // Fixed distribution percentages from the CHAR Coin schema
    let staking_percent = 150; // 15% to staking rewards
    let donation_percent = 750; // 75% to donation ecosystem

    // Calculate staking amount (15%)
    let staking_amount = total_amount
        .checked_mul(staking_percent as u64)
        .unwrap()
        .checked_div(1000)
        .unwrap();

    // Calculate donation ecosystem total (75%)
    let donation_total = total_amount
        .checked_mul(donation_percent as u64)
        .unwrap()
        .checked_div(1000)
        .unwrap();

    // Split donation into subcategories 
    let reward_system = donation_total 
        .checked_mul(200)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 20% of (donation_total)



    let monthly_reward_classification = reward_system 
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 50% of (reward_system)


  let annual_reward_classification = reward_system 
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 50% of (reward_system)




    let donation_system = donation_total 
        .checked_mul(800)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 80% of (donation_total)



    // Split monthly donation into immediate and reserved portions
    let monthly_donation_fund = donation_system
        .checked_mul(800)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 80% of (monthly_donation_amount)
    let annual_donation_fund = donation_system
        .checked_mul(100)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 10% of (monthly_donation_amount)
     let chai_fund = donation_system
        .checked_mul(100)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 10% of (monthly_donation_amount)

    // Transfer to staking rewards
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.staking_reward_ata.to_account_info(),
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
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.monthly_reward_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        monthly_reward_classification,
    )?;

    // Transfer to annual reward fund
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.annual_reward_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        annual_reward_classification,
    )?;

    // Transfer to immediate monthly donation
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.monthly_donation_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        monthly_donation_fund,
    )?;

    // Transfer reserved portion to annual charity
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.annual_donation_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        annual_donation_fund,
    )?;


     // Transfer to chai funds
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.chai_funds_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        chai_fund,
    )?;

    msg!(
        "Monthly funds released: Staking={}, Monthly Reward={}, Annual Reward={}, Monthly Donation={}, Annual Reserved={} chai funds={}" ,
        staking_amount,
        monthly_reward_classification,
        annual_reward_classification,
        monthly_donation_fund,
        annual_donation_fund,
        chai_fund
    );
    Ok(())
}

