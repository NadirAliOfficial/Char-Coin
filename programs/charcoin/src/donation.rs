use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

use crate::{ConfigAccount, CustomError, UserStakeInfo};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum CharityStatus {
    Active,
    Finalized,
}

#[account]
pub struct Charity {
    pub id: u64,             // Unique charity ID
    pub title: String,        // Charity name
    pub wallet: Pubkey,      // Charity's wallet address
    pub total_votes: u64,    // Total weighted votes received
    pub start_time: u64,     // Voting start time (unix timestamp)
    pub end_time: u64,       // Voting end time (unix timestamp)
    pub status: CharityStatus,
    pub admin: Pubkey, // Admin's public key for managing the charity
}

#[account]
pub struct VoteRecord {
    pub charity: Pubkey,  // Charity this vote is for
    pub voter: Pubkey,    // Voter's public key
    pub vote_weight: u64, // Voting weight (based on staked tokens)
    pub voted: bool,
}



/// Registers a new charity for donation voting.
pub fn register_charity(
    ctx: Context<RegisterCharity>,
    title: String,
    wallet: Pubkey,
    start_time: u64,
    end_time: u64,
) -> Result<()> {
     if start_time == 0 {
        return Err(CustomError::InvalidArg.into());
    }
    if end_time == 0 {
        return Err(CustomError::InvalidArg.into());
    }
    if wallet == Pubkey::default() {
        return Err(CustomError::InvalidArg.into());
    }

    let charity = &mut ctx.accounts.charity;
    charity.id = ctx.accounts.config_account.config.next_charity_id;
    charity.title = title;
    charity.wallet = wallet;
    charity.total_votes = 0;
    charity.start_time = start_time;
    charity.end_time = end_time;
    charity.status = CharityStatus::Active;
    charity.admin = ctx.accounts.registrar.key();
    msg!("Charity '{}' registered. Charity id '{}", charity.title,ctx.accounts.config_account.config.next_charity_id);
    ctx.accounts.config_account.config.next_charity_id += 1;
    Ok(())
}

/// Casts vote for a charity
pub fn cast_vote(ctx: Context<CastVote>, _charity_id: u64,char_points:u64) -> Result<()> {
    require!(char_points > 0, CustomError::InvalidArg);
    let config_account = &mut ctx.accounts.config_account;
    let vote_record = &mut ctx.accounts.vote_record;
    let charity = &mut ctx.accounts.charity;

    let clock = Clock::get()?.unix_timestamp as u64;
    let user = &mut ctx.accounts.user;
    require!(char_points <= user.voting_power , CustomError::YouDontHaveEnoughVotingPower);
    
    let amount_staked = user.total_amount;
    if vote_record.voted {
        return Err(CustomError::AlreadyVoted.into());
    }


    require!(user.last_vote_time < charity.start_time,CustomError::VotingNotEligible);

    require!(
        amount_staked >= config_account.config.min_governance_stake, // Minimum stake to vote 
        CustomError::VotingNotEligible
    );
    // Ensure user has staked for at least 15 days
    require!( 
        user.eligible_at > 0 && clock - user.eligible_at >= config_account.config.min_stake_duration_voting, // 15 days
        CustomError::VotingNotEligible
    );
    // Ensure voting is active.
    require!(
        clock >= charity.start_time && clock <= charity.end_time,
        CustomError::VotingNotActive
    );
    user.last_vote_time = clock;
    let vote_weight = user.voting_power;

    user.voting_power = user.voting_power
     .checked_sub(char_points)
        .ok_or(CustomError::MathError)?;
     
    vote_record.charity = charity.key();
    vote_record.voter = ctx.accounts.voter.key();
    vote_record.vote_weight = vote_weight;
    vote_record.voted = true;
    charity.total_votes = charity
        .total_votes
        .checked_add(vote_weight)
        .ok_or(CustomError::MathError)?;
    msg!(
        "Voter {} cast {} votes for charity {}",
        vote_record.voter,
        vote_weight,
        charity.title
    );

    Ok(())
}

/// Finalizes the charity voting after the voting period has ended.
pub fn finalize_charity_vote(ctx: Context<FinalizeCharityVote>, _charity_id: u64) -> Result<()> {
    let clock = Clock::get()?.unix_timestamp as u64;
    let charity = &mut ctx.accounts.charity;
    require!(clock > charity.end_time, CustomError::VotingNotEnded);
    require!(
        charity.status == CharityStatus::Active,
        CustomError::CharityAlreadyFinalized
    );
    charity.status = CharityStatus::Finalized;
    msg!(
        "Charity '{}' finalized with {} total votes",
        charity.title,
        charity.total_votes
    );
    Ok(())
}

#[derive(Accounts)]
pub struct RegisterCharity<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        init, 
        payer = admin,
        seeds=[b"charity".as_ref(),config_account.config.next_charity_id.to_le_bytes().as_ref()],
        bump, 
        space = 
            8 +                     // discriminator
            8 +                     // id
            4 + 64 +                // title (4 bytes for length prefix + actual string 64 bytes)
            32 +                    // wallet
            8 +                     // total_votes
            8 +                     // start_time
            8 +                     // end_time
            2 +                     // status (Active,Finalized)
            32                      // admin
    )]
    pub charity: Account<'info, Charity>,
    /// CHECK: Registrar is the address of admin of charity
    pub registrar: AccountInfo<'info>, 
    #[account(
        mut,
        constraint = registrar.key() == config_account.config.admin,
    )]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(charity_id:u64)]
pub struct CastVote<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,

    #[account(
        mut,
        seeds=[b"charity".as_ref(),charity_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub charity: Account<'info, Charity>,

    #[account(
        init_if_needed,
        payer = voter,
        space = 8 + std::mem::size_of::<VoteRecord>(),
        seeds = [b"vote", charity.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [b"user",  voter.key().as_ref()],
        bump = user.bump
    )]
    pub user: Account<'info, UserStakeInfo>,


    #[account(mut)]
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(charity_id:u64)]

pub struct FinalizeCharityVote<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        mut,
        seeds=[b"charity".as_ref(),charity_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub charity: Account<'info, Charity>,
    #[account(
        mut,
        constraint = admin.key() == charity.admin 
    )]
    pub admin: Signer<'info>,
}
