use anchor_lang::prelude::*;
use anchor_spl::token_2022::{transfer_checked, Token2022 as Token, TransferChecked};
use anchor_spl::token_interface::{TokenAccount,Mint};

use crate::{ConfigAccount, StakingPool};

#[derive(Accounts)]
pub struct ReleaseRewards<'info> {
    #[account(
        mut,
            seeds=[b"config".as_ref()],
            bump
        )]
    pub config_account: Account<'info, ConfigAccount>,

    ///  Treasury token account holding funds to be distributed.
    #[account(
        mut,
        constraint = treasury_ata.mint == config_account.config.char_token_mint,
        constraint = treasury_ata.owner == config_account.config.treasury_authority

    )]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = monthly_top_tier_ata.mint == config_account.config.char_token_mint,
        constraint = monthly_top_tier_ata.owner == config_account.config.monthly_top_tier_wallet

    )]
    pub monthly_top_tier_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = monthly_charity_lottery_ata.mint == config_account.config.char_token_mint,
        constraint = monthly_charity_lottery_ata.owner == config_account.config.monthly_charity_lottery_wallet
    )]
    pub monthly_charity_lottery_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = annual_top_tier_ata.mint == config_account.config.char_token_mint,
        constraint = annual_top_tier_ata.owner == config_account.config.annual_top_tier_wallet
    )]
    pub annual_top_tier_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = annual_charity_lottery_ata.mint == config_account.config.char_token_mint,
        constraint = annual_charity_lottery_ata.owner == config_account.config.annual_charity_lottery_wallet
    )]
    pub annual_charity_lottery_ata: InterfaceAccount<'info, TokenAccount>,
  

    /// will use https://squads.xyz/ for multi sig
    /// Authority for treasury withdrawals.
    #[account(
        mut,
        constraint = config_account.config.treasury_authority == treasury_authority.key()
    )]
    pub treasury_authority: Signer<'info>,
    #[account(constraint = mint.key() == config_account.config.char_token_mint)]
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct ReleaseDonations<'info> {
    #[account(
        mut,
            seeds=[b"config".as_ref()],
            bump
        )]
    pub config_account: Account<'info, ConfigAccount>,

    ///  Treasury token account holding funds to be distributed.
    #[account(
        mut,
        constraint = treasury_ata.mint == config_account.config.char_token_mint,
        constraint = treasury_ata.owner == config_account.config.treasury_authority

    )]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,


    #[account(mut,
        constraint = monthly_one_time_causes_ata.mint == config_account.config.char_token_mint,
        constraint = monthly_one_time_causes_ata.owner == config_account.config.monthly_one_time_causes_wallet

    )]
    pub monthly_one_time_causes_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = monthly_infinite_impact_causes_ata.mint == config_account.config.char_token_mint,
        constraint = monthly_infinite_impact_causes_ata.owner == config_account.config.monthly_infinite_impact_causes_wallet

    )]
    pub monthly_infinite_impact_causes_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = annual_one_time_causes_ata.mint == config_account.config.char_token_mint,
        constraint = annual_one_time_causes_ata.owner == config_account.config.annual_one_time_causes_wallet

    )]
    pub annual_one_time_causes_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = annual_infinite_impact_causes_ata.mint == config_account.config.char_token_mint,
        constraint = annual_infinite_impact_causes_ata.owner == config_account.config.annual_infinite_impact_causes_wallet

    )]
    pub annual_infinite_impact_causes_ata: InterfaceAccount<'info, TokenAccount>,

     #[account(
        mut,
        constraint = char_funds_ata.mint == config_account.config.char_token_mint,
        constraint = char_funds_ata.owner == config_account.config.char_funds
    )]
    pub char_funds_ata: InterfaceAccount<'info, TokenAccount>,

    /// will use https://squads.xyz/ for multi sig
    /// Authority for treasury withdrawals.
    #[account(
        mut,
        constraint = config_account.config.treasury_authority == treasury_authority.key()
    )]
    pub treasury_authority: Signer<'info>,
    #[account(constraint = mint.key() == config_account.config.char_token_mint)]
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

pub fn release_rewards(ctx: Context<ReleaseRewards>, total_amount: u64) -> Result<()> {
    // Fixed distribution percentages from the CHAR Coin schema
    let donation_percent = 750; // 75% to donation ecosystem

   

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

   

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.monthly_top_tier_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                mint:ctx.accounts.mint.to_account_info()

            },
        ),
        monthly_top_tier_percentage,
                ctx.accounts.mint.decimals

    )?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.monthly_charity_lottery_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                mint:ctx.accounts.mint.to_account_info()
            },
        ),
        monthly_charity_lottery_percentage,
        ctx.accounts.mint.decimals
    )?;
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.annual_top_tier_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                mint:ctx.accounts.mint.to_account_info()

            },
        ),
        annual_top_tier_percentage,
                ctx.accounts.mint.decimals

    )?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.annual_charity_lottery_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                mint:ctx.accounts.mint.to_account_info()

            },
        ),
        annual_charity_lottery_percentage,
                ctx.accounts.mint.decimals

    )?;

    Ok(())
}




pub fn release_donations(ctx:Context<ReleaseDonations>,total_amount: u64)->Result<()>{
        let donation_percent = 750; // 75% to donation ecosystem
  // Calculate donation ecosystem total (75%)
    let donation_total = total_amount
        .checked_mul(donation_percent as u64)
        .unwrap()
        .checked_div(1000)
        .unwrap();

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
  let char_fund = donation_system
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

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.monthly_one_time_causes_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                mint:ctx.accounts.mint.to_account_info()

            },
        ),
        monthly_one_time_causes_percentage,
                ctx.accounts.mint.decimals

    )?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx
                    .accounts
                    .monthly_infinite_impact_causes_ata
                    .to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                mint:ctx.accounts.mint.to_account_info()

            },
        ),
        monthly_infinite_impact_causes_percentage,
                ctx.accounts.mint.decimals

    )?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.annual_one_time_causes_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                mint:ctx.accounts.mint.to_account_info()

            },
        ),
        annual_one_time_causes_percentage,
                ctx.accounts.mint.decimals

    )?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx
                    .accounts
                    .annual_infinite_impact_causes_ata
                    .to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                mint:ctx.accounts.mint.to_account_info()

            },
        ),
        annual_infinite_impact_causes_percentage,
                ctx.accounts.mint.decimals

    )?;
 

    // Transfer to char funds
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.char_funds_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                mint:ctx.accounts.mint.to_account_info()
            },
        ),
        char_fund,
                ctx.accounts.mint.decimals

    )?;
    Ok(())
}




#[derive(Accounts)]
pub struct ReleaseStakingFunds<'info> {
    #[account(
        mut,
            seeds=[b"config".as_ref()],
            bump
        )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        mut,
        seeds = [b"staking_pool".as_ref(), mint.key().as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,
    ///  Treasury token account holding funds to be distributed.
    #[account(
        mut,
        constraint = treasury_ata.mint == config_account.config.char_token_mint,
        constraint = treasury_ata.owner == config_account.config.treasury_authority
    )]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = staking_reward_ata.mint == config_account.config.char_token_mint,
        constraint = staking_reward_ata.owner == staking_pool.staking_reward_account
    )]
    pub staking_reward_ata: InterfaceAccount<'info, TokenAccount>,




    /// will use https://squads.xyz/ for multi sig
    /// Authority for treasury withdrawals.
    #[account(
        mut,
        constraint = config_account.config.treasury_authority == treasury_authority.key()
    )]
    pub treasury_authority: Signer<'info>,
        #[account(mut,
    constraint = mint.key() == config_account.config.char_token_mint)]
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token>,
}



pub fn release_staking_char_funds(ctx: Context<ReleaseStakingFunds>, total_amount: u64) ->Result<()>{
    let staking_percent = 150; // 15% to staking rewards

 // Calculate staking amount (15%)
    let staking_amount = total_amount
        .checked_mul(staking_percent as u64)
        .unwrap()
        .checked_div(1000)
        .unwrap();

    // Transfer to staking rewards
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.staking_reward_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
                                mint:ctx.accounts.mint.to_account_info()

            },
        ),
        staking_amount,
                ctx.accounts.mint.decimals

    )?;
  
    Ok(())
}