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
    pub id: u64,
    pub creator: Pubkey,
    pub title: String,
    pub description: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub status: ProposalStatus,
    pub end_time: i64,
}

#[account]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub amount_staked: u64,
    pub vote_choice: bool, // true for Yes, false for No
}

/// Submit a new proposal with a given title, description, and duration (in seconds).
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
    proposal.end_time = Clock::get()?.unix_timestamp + duration;
    msg!("Proposal '{}' submitted by {}", proposal.title, proposal.creator);
    Ok(())
}

/// Vote on a proposal using staked tokens as vote weight.
/// `amount_staked` represents the voting power of the voter.
pub fn vote_on_proposal(
    ctx: Context<VoteOnProposal>,
    proposal_id: u64,
    vote_choice: bool,
    amount_staked: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let current_time = Clock::get()?.unix_timestamp;
    require!(current_time < proposal.end_time, GovernanceError::VotingPeriodEnded);

    // Voting power is determined by the staked amount.
    if vote_choice {
        proposal.yes_votes = proposal.yes_votes
            .checked_add(amount_staked)
            .ok_or(GovernanceError::MathError)?;
    } else {
        proposal.no_votes = proposal.no_votes
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

/// Finalizes the proposal once the voting period has ended.
pub fn finalize_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let current_time = Clock::get()?.unix_timestamp;
    require!(current_time >= proposal.end_time, GovernanceError::VotingStillActive);

    if proposal.yes_votes > proposal.no_votes {
        proposal.status = ProposalStatus::Approved;
        msg!("Proposal {} approved!", proposal.id);
    } else {
        proposal.status = ProposalStatus::Rejected;
        msg!("Proposal {} rejected!", proposal.id);
    }
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
