use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer};



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

    /// CHECK: Destination token account for annual reserved donations (12%).
    #[account(mut)]
    pub chai_funds: UncheckedAccount<'info>,
    /// Authority for treasury withdrawals.
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
        monthly_reward_classification,
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
        annual_reward_classification,
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
        monthly_donation_fund,
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
        annual_donation_fund,
    )?;


     // Transfer to chai funds
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.chai_funds.to_account_info(),
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

