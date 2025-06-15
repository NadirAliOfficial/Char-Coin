use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::ConfigAccount;

#[derive(Accounts)]
pub struct ReleaseMonthlyFunds<'info> {
    #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]
    pub config_account: Account<'info, ConfigAccount>,

    ///  Treasury token account holding funds to be distributed.
    #[account(
        mut,
        constraint = treasury_ata.mint == config_account.config.char_token_mint
    )]
    pub treasury_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = staking_reward_ata.mint == config_account.config.char_token_mint
    )]
    pub staking_reward_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = monthly_top_tier_ata.mint == config_account.config.char_token_mint
    )]
    pub monthly_top_tier_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = monthly_charity_lottery_ata.mint == config_account.config.char_token_mint
    )]
    pub monthly_charity_lottery_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = annual_top_tier_ata.mint == config_account.config.char_token_mint
    )]
    pub annual_top_tier_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = annual_charity_lottery_ata.mint == config_account.config.char_token_mint
    )]
    pub annual_charity_lottery_ata: Account<'info, TokenAccount>,
    #[account(mut,
        constraint = monthly_one_time_causes_ata.mint == config_account.config.char_token_mint

    )]
    pub monthly_one_time_causes_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = monthly_infinite_impact_causes_ata.mint == config_account.config.char_token_mint

    )]
    pub monthly_infinite_impact_causes_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = annual_one_time_causes_ata.mint == config_account.config.char_token_mint

    )]
    pub annual_one_time_causes_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = annual_infinite_impact_causes_ata.mint == config_account.config.char_token_mint

    )]
    pub annual_infinite_impact_causes_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = char_funds_ata.mint == config_account.config.char_token_mint
    )]
    pub char_funds_ata: Account<'info, TokenAccount>,

    /// will use https://squads.xyz/ for multi sig
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

    let monthly_top_tier_percentage = monthly_reward_classification
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 50% of (monthly_reward_classification)
    let monthly_charity_lottery_percentage = monthly_reward_classification
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 50% of (monthly_reward_classification)

    let annual_reward_classification = reward_system
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 50% of (reward_system)

    let annual_top_tier_percentage = annual_reward_classification
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 50% of (annual_reward_classification)
    let annual_charity_lottery_percentage = annual_reward_classification
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap(); // 50% of (annual_reward_classification)

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

    let monthly_one_time_causes_percentage = monthly_donation_fund
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap();

    let monthly_infinite_impact_causes_percentage = monthly_donation_fund
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap();
    let annual_one_time_causes_percentage = annual_donation_fund
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap();

    let annual_infinite_impact_causes_percentage = annual_donation_fund
        .checked_mul(500)
        .unwrap()
        .checked_div(1000)
        .unwrap();

    let char_fund = donation_system
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

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.monthly_top_tier_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        monthly_top_tier_percentage,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.monthly_charity_lottery_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        monthly_charity_lottery_percentage,
    )?;
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.annual_top_tier_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        annual_top_tier_percentage,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.annual_charity_lottery_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        annual_charity_lottery_percentage,
    )?;

    // Transfer to immediate monthly donation
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.monthly_one_time_causes_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        monthly_one_time_causes_percentage,
    )?;

    // Transfer reserved portion to annual charity
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx
                    .accounts
                    .monthly_infinite_impact_causes_ata
                    .to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        monthly_infinite_impact_causes_percentage,
    )?;

    // Transfer to immediate monthly donation
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.annual_one_time_causes_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        annual_one_time_causes_percentage,
    )?;

    // Transfer reserved portion to annual charity
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx
                    .accounts
                    .annual_infinite_impact_causes_ata
                    .to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        annual_infinite_impact_causes_percentage,
    )?;

    // Transfer to char funds
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.char_funds_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
        ),
        char_fund,
    )?;

    Ok(())
}
