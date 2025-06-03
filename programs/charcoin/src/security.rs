use anchor_lang::prelude::*;

use crate::ConfigAccount;

#[derive(Accounts)]
pub struct InitializeEmergencyState<'info> {
      #[account(
            mut,
            seeds=[b"config".as_ref()],
            bump
        )]
    pub config_account: Account<'info, ConfigAccount>,

    #[account(
        mut,
        constraint = config_account.config.admin == payer.key() // Ensure the signer is the admin
    )]    
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn change_emergency_state(
    ctx: Context<InitializeEmergencyState>,
    state: bool,
) -> Result<()> {
    let emergency_state = &mut ctx.accounts.config_account.config;
    emergency_state.halted = state;
    Ok(())
}


