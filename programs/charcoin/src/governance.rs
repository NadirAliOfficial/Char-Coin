use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
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

// Governance parameters
pub fn submit_proposal(ctx: Context<SubmitProposal>, title: String, description: String, duration: i64) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    proposal.creator = ctx.accounts.creator.key();
    proposal.title = title.clone();
    proposal.description = description;
    proposal.status = ProposalStatus::Active;
    proposal.end_time = Clock::get()?.unix_timestamp + duration;
    
    msg!("New proposal submitted: {}", title);
    Ok(())
}

// Voting process
pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, proposal_id: u64, vote_choice: bool, amount_staked: u64) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let clock = Clock::get()?.unix_timestamp;

    require!(clock < proposal.end_time, GovernanceError::VotingPeriodEnded);

    let vote_weight = amount_staked; // Voting power based on staked tokens

    if vote_choice {
        proposal.yes_votes += vote_weight;
    } else {
        proposal.no_votes += vote_weight;
    }

    msg!("Vote casted on proposal {}: {} votes", proposal_id, vote_weight);
    Ok(())
}
 
// Finalize proposal.
pub fn finalize_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let clock = Clock::get()?.unix_timestamp;

    require!(clock >= proposal.end_time, GovernanceError::VotingStillActive);

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
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(init, payer = creator, space = 8 + 32 + 256)]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

#[error_code]
pub enum GovernanceError {
    #[msg("Voting period has ended.")]
    VotingPeriodEnded,
    #[msg("Voting period is still active.")]
    VotingStillActive,
}
