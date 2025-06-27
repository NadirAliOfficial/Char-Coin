use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("No tokens available for buyback.")]
    NoTokensToBuyback,
    #[msg("Voting period is not active.")]
    VotingNotActive,
    #[msg("Voting period has not ended.")]
    VotingNotEnded,
    #[msg("User not Eligible for voting")]
    VotingNotEligible,
    #[msg("User has not staked any tokens.")]
    NoStakedTokens,
    #[msg("User Already Voted")]
    AlreadyVoted,
    #[msg("Charity Already Finalized")]
    CharityAlreadyFinalized,
    #[msg("Invalid Argument Provided")]
    InvalidArg,
    #[msg("program is halted")]
    ProgramIsHalted,
    #[msg("Staking period has not been met yet")]
    StakingPeriodNotMet,
    #[msg("Wrong Staking Package")]
    WrongStakingPackage,
    #[msg("Reward has already been claimed")]
    RewardAlreadyClaimed,
    #[msg("Already Staked")]
    AlreadyStaked,
    #[msg("Already un Staked")]
    AlreadyUnStaked,
    #[msg("Wait period not over yet")]
    WaitPeriodNotOverYet,
    #[msg("Request Unstake First")]
    RequestUnstakeFirst,
    #[msg("Unstake Already Requested")]
    UnstakeAlreadyRequested,
    #[msg("Invalid Stake Id")]
    InvalidStakeId,
    #[msg("Nothing To Claim")]
    NothingToClaim,
    #[msg("Staking Reward Insufficient Balance")]
    StakingRewardInsufficientBalance,
    #[msg("Math error occurred.")]
    MathError,
    #[msg("You Dont Have Enough Voting Power")]
    YouDontHaveEnoughVotingPower,
}
