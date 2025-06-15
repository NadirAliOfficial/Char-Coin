
# Issue 1
The `vote_on_proposal` lacks critical checks to ensure that:
1. a staker has not already submitted an unstake request



# Issue 2
The following code block is intended to verify whether the voter initially staked 15 days ago. However, it only checks the first
stake, allowing a user to stake a very small amount initially and then stake other amounts after 15 days. Consequently, the
newly staked CHAR can be used to vote immediately, bypassing the intended lock-up period.





changes:

1. donation `cast_vote` min_governance_stake check
2. donation vote_power implementation
3. governance `vote_on_proposal` min_governance_stake check
4. governance vote_power implementation
5. set_reward_percentage_handler fn added
6. update_settings fn added
7.   added these wallets  
    - monthly_top_tier_wallet: 
    - monthly_charity_lottery_wallet: 
    - annual_top_tier_wallet: 
    - annual_charity_lottery_wallet: 
    - monthly_one_time_causes_wallet: 
    - monthly_infinite_impact_causes_wallet: 
    - annual_one_time_causes_wallet: 
    - annual_infinite_impact_causes_wallet: 
8. release_funds updated logic for fund transfer
9. staking lockup dynamic
10. user largest_lockup logic implementation 
11. claim_reward added the logic for get reward_percentage based on lockup
12. added testcases for the above changes.




100