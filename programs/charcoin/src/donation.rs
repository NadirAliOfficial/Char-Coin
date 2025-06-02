use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

use crate::{ConfigAccount, StakingPool, UserStakeInfo};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum CharityStatus {
    Active,
    Finalized,
}

#[account]
pub struct Charity {
    pub id: u64,             // Unique charity ID
    pub name: String,        // Charity name
    pub description: String, // Charity description
    pub wallet: Pubkey,      // Charity's wallet address
    pub total_votes: u64,    // Total weighted votes received
    pub start_time: i64,     // Voting start time (unix timestamp)
    pub end_time: i64,       // Voting end time (unix timestamp)
    pub status: CharityStatus,
    pub admin:Pubkey, // Admin's public key for managing the charity
}

#[account]
pub struct VoteRecord {
    pub charity: Pubkey,  // Charity this vote is for
    pub voter: Pubkey,    // Voter's public key
    pub vote_weight: u64, // Voting weight (based on staked tokens)
}

#[error_code]
pub enum CharityError {
    #[msg("Voting period is not active.")]
    VotingNotActive,
    #[msg("Voting period has not ended.")]
    VotingNotEnded,
    #[msg("Charity voting is already finalized.")]
    AlreadyFinalized,
    #[msg("Math error occurred.")]
    MathError,
   #[msg("User must have staked for at least 15 days to vote.")]
    VotingNotEligible,
    #[msg("User has not staked any tokens.")]
    NoStakedTokens,
}

/// Registers a new charity for donation voting.
pub fn register_charity(
    ctx: Context<RegisterCharity>,
    name: String,
    description: String,
    wallet: Pubkey,
    start_time: i64,
    end_time: i64,
) -> Result<()> {
    let charity = &mut ctx.accounts.charity;
    charity.id = ctx.accounts.config_account.config.next_charity_id;
    charity.name = name;
    charity.description = description;
    charity.wallet = wallet;
    charity.total_votes = 0;
    charity.start_time = start_time;
    charity.end_time = end_time;
    charity.status = CharityStatus::Active;
    charity.admin = ctx.accounts.registrar.key();
    msg!("Charity '{}' registered.", charity.name);
    ctx.accounts.config_account.config.next_charity_id +=1;
    Ok(())
}

/// Casts or updates a vote for a charity. VoteRecord is created with PDA seeds
/// ["vote", charity.key(), voter.key()].
pub fn cast_vote(ctx: Context<CastVote>, vote_weight: u64) -> Result<()> {
    let clock = Clock::get()?.unix_timestamp;
    let user = &ctx.accounts.user;
    require!(
        user.total_amount > 0,
        CharityError::NoStakedTokens
    );
    // require!(
    //     clock - user.staked_at >= 15 * 86400,
    //     CharityError::VotingNotEligible
    // );
    require!(
        clock - user.first_staked_at >= 240, // 4 mints
        CharityError::VotingNotEligible
    );
    let charity = &mut ctx.accounts.charity;
    // Ensure voting is active.
    require!(
        clock >= charity.start_time && clock <= charity.end_time,
        CharityError::VotingNotActive
    );

    let vote_record = &mut ctx.accounts.vote_record;
    if vote_record.vote_weight == 0 {
        // New vote.
        vote_record.charity = charity.key();
        vote_record.voter = ctx.accounts.voter.key();
        vote_record.vote_weight = vote_weight;
        charity.total_votes = charity
            .total_votes
            .checked_add(vote_weight)
            .ok_or(CharityError::MathError)?;
        msg!(
            "Voter {} cast {} votes for charity {}",
            vote_record.voter,
            vote_weight,
            charity.name
        );
    } else {
        // Update existing vote.
        if vote_weight > vote_record.vote_weight {
            let diff = vote_weight
                .checked_sub(vote_record.vote_weight)
                .ok_or(CharityError::MathError)?;
            charity.total_votes = charity
                .total_votes
                .checked_add(diff)
                .ok_or(CharityError::MathError)?;
            vote_record.vote_weight = vote_weight;
            msg!(
                "Voter {} increased vote by {} for charity {}",
                vote_record.voter,
                diff,
                charity.name
            );
        } else {
            let diff = vote_record
                .vote_weight
                .checked_sub(vote_weight)
                .ok_or(CharityError::MathError)?;
            charity.total_votes = charity
                .total_votes
                .checked_sub(diff)
                .ok_or(CharityError::MathError)?;
            vote_record.vote_weight = vote_weight;
            msg!(
                "Voter {} decreased vote by {} for charity {}",
                vote_record.voter,
                diff,
                charity.name
            );
        }
    }
    Ok(())
}

/// Finalizes the charity voting after the voting period has ended.
pub fn finalize_charity_vote(ctx: Context<FinalizeCharityVote>) -> Result<()> {
    let clock = Clock::get()?.unix_timestamp;
    let charity = &mut ctx.accounts.charity;
    require!(clock > charity.end_time, CharityError::VotingNotEnded);
    charity.status = CharityStatus::Finalized;
    msg!(
        "Charity '{}' finalized with {} total votes",
        charity.name,
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
    #[account(init, payer = registrar,seeds=[b"charity".as_ref(),config_account.config.next_charity_id.to_le_bytes().as_ref()],bump, space = 8 + 8 + 4 + 64 + 4 + 256 + 32 + 8 + 8 + 1)]
    pub charity: Account<'info, Charity>,
    #[account(mut)]
    pub registrar: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// VoteRecord is created with PDA seeds: ["vote", charity.key(), voter.key()]
#[derive(Accounts)]
#[instruction()]
pub struct CastVote<'info> {
  #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
    #[account(mut)]
    pub charity: Account<'info, Charity>,
    #[account(init_if_needed,
        payer = voter,
        space = 8 + 32 + 32 + 8,
        seeds = [b"vote", charity.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    #[account(mut)]
    pub voter: Signer<'info>,
    pub system_program: Program<'info, System>,

    #[account(
        seeds = [b"user", staking_pool.key().as_ref(), voter.key().as_ref()],
        bump = user.bump
    )]
    pub user: Account<'info, UserStakeInfo>,

    pub staking_pool: Account<'info, StakingPool>
}

#[derive(Accounts)]
pub struct FinalizeCharityVote<'info> {
  #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
    #[account(mut)]
    pub charity: Account<'info, Charity>,
    #[account(mut,
    constraint = admin.key() == charity.admin 
)]
    pub admin: Signer<'info>,
}
