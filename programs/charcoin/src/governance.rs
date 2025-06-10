use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

use crate::{ConfigAccount, StakingPool, UserStakeInfo};

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
    pub end_time: i64,          // Voting deadline (unix timestamp)
}

#[account]
pub struct Vote {
    pub proposal_id: u64,   // Associated proposal ID
    pub voter: Pubkey,      // Voter's public key
    pub amount_staked: u64, // Voting power (staked tokens)
    pub vote_choice: bool,  // true for Yes, false for No
}

/// Submits a new proposal.
pub fn submit_proposal(
    ctx: Context<SubmitProposal>,
    title: String,
    description: String,
    duration: i64,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
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
    vote_choice: bool

) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let proposal = &mut ctx.accounts.proposal;
    let staking_pool = &mut ctx.accounts.staking_pool;
    let config_account = &mut ctx.accounts.config_account;
    let user = &ctx.accounts.user;
    
    let amount_staked = user.total_amount;
     require!(
        amount_staked >= config_account.config.min_governance_stake, // Minimum stake to vote
        GovernanceError::VotingNotEligible
    );

     require!(
        current_time - user.first_staked_at >= config_account.config.min_stake_duration_voting, // 15 days
        GovernanceError::VotingNotEligible
    );
    require!(
        current_time < proposal.end_time,
        GovernanceError::VotingPeriodEnded
    );

    let vote_power = staking_pool.stake_lockup_reward_array
        .iter()
        .find(|x| x.lockup_days == user.largest_lockup)
        .unwrap()
        .vote_power;
    //  voting_amount = 500 * 10e6 / 1000    e.g 500 = 0.5, 1000 = 1
    let voting_amount = (vote_power as u128 * amount_staked as u128 / 1000) as u64;

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
    msg!(
        "Vote cast on proposal {}: {} votes ({} for, {} against)",
        proposal_id,
        amount_staked,
        proposal.yes_votes,
        proposal.no_votes
    );
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
pub fn finalize_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let proposal = &mut ctx.accounts.proposal;
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

/// ---------------------------------------------------------------------
/// DAO Treasury Functionality (Merged into Governance)
/// ---------------------------------------------------------------------

/// Account structure for the DAO Treasury.
#[account]
pub struct Treasury {
    /// List of multisig owner public keys.
    pub owners: Vec<Pubkey>,
    /// Approval threshold required for executing a withdrawal.
    pub threshold: u8,
    /// Counter for executed withdrawals.
    pub withdrawal_count: u64,
}

/// Account structure for a withdrawal proposal.
#[account]
pub struct WithdrawalProposal {
    /// Amount (in lamports) proposed for withdrawal.
    pub amount: u64,
    /// Recipient account to which funds will be sent.
    pub recipient: Pubkey,
    /// List of multisig approvals (owner public keys) collected.
    pub approvals: Vec<Pubkey>,
    /// Whether this proposal has been executed.
    pub executed: bool,
}

/// Initializes the DAO treasury.
pub fn initialize_treasury(
    ctx: Context<InitializeTreasury>,
    owners: Vec<Pubkey>,
    threshold: u8,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let treasury = &mut ctx.accounts.treasury;
    require!(
        owners.len() > 1 && owners.len() <= 10,
        GovernanceError::InvalidOwners
    );
    require!(
        threshold > 0 && threshold <= owners.len() as u8,
        GovernanceError::InvalidThreshold
    );
    treasury.owners = owners;
    treasury.threshold = threshold;
    treasury.withdrawal_count = 0;
    msg!(
        "DAO Treasury initialized with {} owners and threshold {}.",
        treasury.owners.len(),
        treasury.threshold
    );
    emit!(TreasuryInitializedEvent {
        owners_count: treasury.owners.len() as u64,
        threshold: treasury.threshold,
        timestamp: current_time,
    });
    Ok(())
}

/// Creates a new withdrawal proposal.
pub fn create_withdrawal(
    ctx: Context<CreateWithdrawal>,
    amount: u64,
    recipient: Pubkey,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let withdrawal = &mut ctx.accounts.withdrawal;
    withdrawal.amount = amount;
    withdrawal.recipient = recipient;
    withdrawal.approvals = Vec::new();
    withdrawal.executed = false;
    msg!(
        "Withdrawal proposal created: {} lamports to {:?}",
        amount,
        recipient
    );
    emit!(WithdrawalProposalCreatedEvent {
        amount,
        recipient,
        timestamp: current_time,
    });
    Ok(())
}

/// Approves a withdrawal proposal.
pub fn approve_withdrawal(ctx: Context<ApproveWithdrawal>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let treasury = &ctx.accounts.treasury;
    let withdrawal = &mut ctx.accounts.withdrawal;
    let signer = ctx.accounts.signer.key();
    require!(
        treasury.owners.contains(&signer),
        GovernanceError::Unauthorized
    );
    require!(!withdrawal.executed, GovernanceError::AlreadyExecuted);
    if !withdrawal.approvals.contains(&signer) {
        withdrawal.approvals.push(signer);
        msg!("Approval added by {:?}", signer);
        emit!(WithdrawalApprovedEvent {
            signer,
            current_approvals: withdrawal.approvals.len() as u64,
            timestamp: current_time,
        });
    }
    Ok(())
}

/// Executes a withdrawal proposal if sufficient approvals have been collected.
pub fn execute_withdrawal(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let treasury = &ctx.accounts.treasury;
    let withdrawal = &mut ctx.accounts.withdrawal;
    require!(!withdrawal.executed, GovernanceError::AlreadyExecuted);
    require!(
        withdrawal.approvals.len() as u8 >= treasury.threshold,
        GovernanceError::InsufficientApprovals
    );
    let lamports = withdrawal.amount;
    let treasury_account = ctx.accounts.treasury.to_account_info();
    let recipient_account = ctx.accounts.recipient.to_account_info();
    **treasury_account.try_borrow_mut_lamports()? -= lamports;
    **recipient_account.try_borrow_mut_lamports()? += lamports;
    withdrawal.executed = true;
    msg!(
        "Executed withdrawal of {} lamports to {:?}",
        lamports,
        withdrawal.recipient
    );
    emit!(WithdrawalExecutedEvent {
        amount: lamports,
        recipient: withdrawal.recipient,
        timestamp: current_time,
    });
    Ok(())
}

// Contexts for Treasury Instructions

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
  #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
    #[account(init, seeds=[b"treasury".as_ref()],
            bump, payer = signer, space = 8 + (32 * 10) + 1 + 8)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut,
        constraint = signer.key() == config_account.config.admin,
    )]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateWithdrawal<'info> {
  #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
    #[account(mut)]
    pub treasury: Account<'info, Treasury>,
    #[account(init, payer = signer, seeds=[b"withdrawal".as_ref()],bump,space = 8 + 8 + 32 + (32 * 10) + 1)]
    pub withdrawal: Account<'info, WithdrawalProposal>,
    #[account(mut,
            constraint = signer.key() == config_account.config.admin,

    )]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveWithdrawal<'info> {
   #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
    #[account(mut)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub withdrawal: Account<'info, WithdrawalProposal>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
  #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
    #[account(mut)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub withdrawal: Account<'info, WithdrawalProposal>,
        /// CHECK: Recipient account; no additional checks.

    #[account(mut)]
    pub recipient:  AccountInfo<'info>,
        #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitProposal<'info> {
  #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
    #[account(init, payer = creator, 
        seeds=[b"proposal", creator.key().as_ref(),config_account.config.next_proposal_id.to_le_bytes().as_ref()],
         bump, space = 8 + 32 + 8 + 256 + 8 + 8 + 1 + 8)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
  #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
        
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(
        seeds = [b"user",  voter.key().as_ref()],
        bump = user.bump
    )]
    pub user: Account<'info, UserStakeInfo>,
   #[account(
        mut,
        seeds = [b"staking_pool".as_ref(), staking_pool.token_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,
    pub system_program: Program<'info, System>,

}

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
  #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]    pub config_account: Account<'info, ConfigAccount>,
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
}

#[event]
pub struct ProposalFinalizedEvent {
    pub proposal_id: u64,
    pub status: ProposalStatus,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub timestamp: i64,
}

#[event]
pub struct ProposalSubmittedEvent {
    pub proposal_creator: Pubkey,
    pub proposal_title: String,
    pub timestamp: i64,
}

#[event]
pub struct VoteCastEvent {
    pub proposal_id: u64,
    pub vote_choice: bool,
    pub amount_staked: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub timestamp: i64,
}

#[event]
pub struct TreasuryInitializedEvent {
    pub owners_count: u64,
    pub threshold: u8,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalProposalCreatedEvent {
    pub amount: u64,
    pub recipient: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalApprovedEvent {
    pub signer: Pubkey,
    pub current_approvals: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalExecutedEvent {
    pub amount: u64,
    pub recipient: Pubkey,
    pub timestamp: i64,
}
