// derive creator fund from twitter handle
// creator vault = a PDA derived from creatorId
// seeds = [b"creator_vault", creator_id]
use anchor_lang::prelude::*;
use anchor_lang::system_program::{Transfer, transfer};

#[derive(Accounts)]
#[instruction(creator_id: String)]
pub struct Claim<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    pub authority: UncheckedAccount<'info>,

    /// CHECK: ok
    #[account(
        mut,
        seeds = [b"creator_vault", creator_id.as_bytes()],
        bump
    )]
    pub creator_vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self, creator_id: String) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.creator_vault.to_account_info(),
            to: self.creator.to_account_info()
        };

        let seeds = &[
            &b"creator_vault"[..],
            &[creator_id.as_bytes()],
        ];
        let signer_seeds = [&[&seeds[..]]];

        let cpi_ctx = CpiContext::new_with_signer(self.system_program.to_account_info(), cpi_accounts, signer_seeds);

        transfer(cpi_ctx, self.creator_vault.lamports())
    }
}