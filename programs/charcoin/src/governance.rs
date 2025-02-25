use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum ProposalStatus {
    Active,
    Approved,
    Rejected,
}

#[account]
pub struct Proposal {
    pub id: u64,                  // Unique proposal ID
    pub creator: Pubkey,          // Creator of the proposal
    pub title: String,            // Title of the proposal
    pub description: String,      // Detailed description
    pub yes_votes: u64,           // Total yes votes (weighted)
    pub no_votes: u64,            // Total no votes (weighted)
    pub status: ProposalStatus,   // Current status
    pub end_time: i64,            // Voting deadline (unix timestamp)
}

#[account]
pub struct Vote {
    pub proposal_id: u64,         // Associated proposal ID
    pub voter: Pubkey,            // Voter's public key
    pub amount_staked: u64,       // Voting power (staked tokens)
    pub vote_choice: bool,        // true for Yes, false for No
}

/// Submits a new proposal with a title, description, and duration (in seconds).
pub fn submit_proposal(
    ctx: Context<SubmitProposal>,
    title: String,
    description: String,
    duration: i64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    proposal.creator = ctx.accounts.creator.key();
    proposal.title = title;
    proposal.description = description;
    proposal.yes_votes = 0;
    proposal.no_votes = 0;
    proposal.status = ProposalStatus::Active;
    // Set the end time as current time plus duration.
    proposal.end_time = Clock::get()?.unix_timestamp + duration;
    msg!("Proposal '{}' submitted by {}", proposal.title, proposal.creator);
    Ok(())
}

/// Casts a vote on a proposal using staked tokens as voting power.
/// `vote_choice`: true for Yes, false for No.
pub fn vote_on_proposal(
    ctx: Context<VoteOnProposal>,
    proposal_id: u64,
    vote_choice: bool,
    amount_staked: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        current_time < proposal.end_time,
        GovernanceError::VotingPeriodEnded
    );

    // Add the vote weight to the appropriate counter.
    if vote_choice {
        proposal.yes_votes = proposal
            .yes_votes
            .checked_add(amount_staked)
            .ok_or(GovernanceError::MathError)?;
    } else {
        proposal.no_votes = proposal
            .no_votes
            .checked_add(amount_staked)
            .ok_or(GovernanceError::MathError)?;
    }
    msg!(
        "Vote cast on proposal {}: {} votes ({} for, {} against)",
        proposal_id,
        amount_staked,
        proposal.yes_votes,
        proposal.no_votes
    );
    Ok(())
}

/// Finalizes a proposal after the voting period has ended.
/// Updates the proposal's status to Approved if yes_votes > no_votes, else Rejected,
/// and emits a ProposalFinalizedEvent with the vote counts and status.
pub fn finalize_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        current_time >= proposal.end_time,
        GovernanceError::VotingStillActive
    );

    if proposal.yes_votes > proposal.no_votes {
        proposal.status = ProposalStatus::Approved;
        msg!("Proposal {} approved!", proposal.id);
    } else {
        proposal.status = ProposalStatus::Rejected;
        msg!("Proposal {} rejected!", proposal.id);
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
    #[account(init, payer = creator, space = 8 + 32 + 8 + 256 + 8 + 8 + 1 + 8)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
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
    #[msg("Math error.")]
    MathError,
}

#[event]
pub struct ProposalFinalizedEvent {
    pub proposal_id: u64,
    pub status: ProposalStatus,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub timestamp: i64,
}
