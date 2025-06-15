use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

use crate::{ConfigAccount, UserStakeInfo};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum ProposalStatus {
    Active,
    Approved,
    Rejected,
}

#[account]
pub struct Proposal {
    pub id: u64,                // Unique proposal ID
    pub creator: Pubkey,        // Creator of the proposal
    pub title: String,          // Title of the proposal
    pub description: String,    // Detailed description
    pub yes_votes: u64,         // Total yes votes (weighted)
    pub no_votes: u64,          // Total no votes (weighted)
    pub status: ProposalStatus, // Current status
    pub end_time: u64,          // Voting deadline (unix timestamp)
}

#[account]
pub struct Vote {
    pub proposal_id: u64,   // Associated proposal ID
    pub voter: Pubkey,      // Voter's public key
    pub amount_staked: u64, // Voting power (staked tokens)
    pub vote_choice: bool,  // true for Yes, false for No
    pub voted: bool,        // Whether the user has voted
}

/// Submits a new proposal.
pub fn submit_proposal(
    ctx: Context<SubmitProposal>,
    title: String,
    description: String,
    duration: u64,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp as u64;
    let proposal = &mut ctx.accounts.proposal;
    proposal.creator = ctx.accounts.creator.key();
    proposal.id = ctx.accounts.config_account.config.next_proposal_id;
    proposal.title = title;
    proposal.description = description;
    proposal.yes_votes = 0;
    proposal.no_votes = 0;
    proposal.status = ProposalStatus::Active;
    proposal.end_time = current_time + duration;
    msg!(
        "Proposal '{}' submitted by {}",
        proposal.title,
        proposal.creator
    );
    ctx.accounts.config_account.config.next_proposal_id += 1;
    emit!(ProposalSubmittedEvent {
        proposal_creator: proposal.creator,
        proposal_title: proposal.title.clone(),
        timestamp: current_time,
    });
    Ok(())
}

/// Casts a vote on a proposal.
pub fn vote_on_proposal(
    ctx: Context<VoteOnProposal>,
    proposal_id: u64,
    vote_choice: bool,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp as u64;
    let proposal: &mut Account<'_, Proposal> = &mut ctx.accounts.proposal;
    let config_account = &mut ctx.accounts.config_account;
    let user = &ctx.accounts.user;
    let user_vote_account = &mut ctx.accounts.user_vote_account;
    if user_vote_account.voted {
        return Err(GovernanceError::AlreadyVoted.into());
    }

    require!(
        proposal.id == proposal_id,
        GovernanceError::InvalidProposalId
    );
    let amount_staked = user.total_amount;

    user_vote_account.voted = true;
    user_vote_account.proposal_id = proposal_id;
    user_vote_account.voter = ctx.accounts.voter.key();
    user_vote_account.amount_staked = amount_staked;
    user_vote_account.vote_choice = vote_choice;

    require!(
        amount_staked >= config_account.config.min_governance_stake, // Minimum stake to vote
        GovernanceError::VotingNotEligible
    );

   // Ensure user has staked for at least 15 days
    require!( 
        user.eligible_at > 0 && current_time - user.eligible_at >= config_account.config.min_stake_duration_voting, // 15 days
        GovernanceError::VotingNotEligible
    );
    require!(
        current_time < proposal.end_time,
        GovernanceError::VotingPeriodEnded
    );
    let voting_amount = user.voting_power;

    if vote_choice {
        proposal.yes_votes = proposal
            .yes_votes
            .checked_add(voting_amount)
            .ok_or(GovernanceError::MathError)?;
    } else {
        proposal.no_votes = proposal
            .no_votes
            .checked_add(voting_amount)
            .ok_or(GovernanceError::MathError)?;
    }

    emit!(VoteCastEvent {
        proposal_id,
        vote_choice,
        amount_staked,
        yes_votes: proposal.yes_votes,
        no_votes: proposal.no_votes,
        timestamp: current_time,
    });
    Ok(())
}

/// Finalizes a proposal after voting has ended.
pub fn finalize_proposal(ctx: Context<FinalizeProposal>, _proposal_id: u64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp as u64;
    let proposal = &mut ctx.accounts.proposal;
    require!(
        current_time >= proposal.end_time,
        GovernanceError::VotingStillActive
    );
    require!(
        proposal.status == ProposalStatus::Active,
        GovernanceError::ProposalAlreadyFinalized
    );
    if proposal.yes_votes > proposal.no_votes {
        proposal.status = ProposalStatus::Approved;
    } else {
        proposal.status = ProposalStatus::Rejected;
    }
    emit!(ProposalFinalizedEvent {
        proposal_id: proposal.id,
        status: proposal.status.clone(),
        yes_votes: proposal.yes_votes,
        no_votes: proposal.no_votes,
        timestamp: current_time,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct SubmitProposal<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(
        init, payer = creator, 
        seeds=[b"proposal", config_account.config.next_proposal_id.to_le_bytes().as_ref()],
        bump, 
        space =
            8 +         // discriminator
            8 +         // id
            32 +        // creator
            4 + 100 +   // title
            4 + 500 +   // description
            8 +         // yes_votes
            8 +         // no_votes
            1 +         // status
            8           // end_time
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id:u64)]

pub struct VoteOnProposal<'info> {
    #[account(
        mut,
        seeds=[b"config".as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,

    #[account(
        mut,
        seeds=[b"proposal", proposal_id.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(
        seeds = [b"user",  voter.key().as_ref()],
        bump = user.bump
    )]
    pub user: Account<'info, UserStakeInfo>,
    #[account(
        init_if_needed,
        space = 8 + std::mem::size_of::<Vote>(),
        payer = voter,
        seeds = [b"user_vote", proposal_id.to_le_bytes().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub user_vote_account: Account<'info, Vote>,
 
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id:u64)]
pub struct FinalizeProposal<'info> {
    #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]
    pub config_account: Account<'info, ConfigAccount>,
    #[account(  mut,
        seeds=[b"proposal", proposal_id.to_le_bytes().as_ref()],
        bump)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[error_code]
pub enum GovernanceError {
    #[msg("Voting period has ended.")]
    VotingPeriodEnded,
    #[msg("Voting period is still active.")]
    VotingStillActive,
    #[msg("Math error occurred.")]
    MathError,
    #[msg("Unauthorized operation.")]
    Unauthorized,
    #[msg("Invalid number of multisig owners.")]
    InvalidOwners,
    #[msg("Invalid approval threshold.")]
    InvalidThreshold,
    #[msg("Withdrawal proposal already executed.")]
    AlreadyExecuted,
    #[msg("Insufficient approvals for withdrawal execution.")]
    InsufficientApprovals,
    #[msg("User must have staked for at least 15 days to vote.")]
    VotingNotEligible,
    #[msg("User has not staked any tokens.")]
    NoStakedTokens,
    #[msg("User Already Voted")]
    AlreadyVoted,
    #[msg("Duplicate Owner")]
    DuplicateOwner,
    #[msg("Proposal Already Finalized")]
    ProposalAlreadyFinalized,
    #[msg("Invalid Proposal Id")]
    InvalidProposalId,
}

#[event]
pub struct ProposalFinalizedEvent {
    pub proposal_id: u64,
    pub status: ProposalStatus,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub timestamp: u64,
}

#[event]
pub struct ProposalSubmittedEvent {
    pub proposal_creator: Pubkey,
    pub proposal_title: String,
    pub timestamp: u64,
}

#[event]
pub struct VoteCastEvent {
    pub proposal_id: u64,
    pub vote_choice: bool,
    pub amount_staked: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub timestamp: u64,
}
