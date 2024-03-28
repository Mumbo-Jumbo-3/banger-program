use anchor_lang::prelude::*;
use crate::state::Curve;

#[derive(Accounts)]
pub struct InitCurve<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = Curve::INIT_SPACE,
        seeds = [b"curve"],
        bump
    )]
    pub curve: Account<'info, Curve>,

    pub system_program: Program<'info, System>
}

impl<'info> InitCurve<'info> {
    pub fn init_curve(&mut self, pow: u64, frac: u64) -> Result<()> {
        self.curve.set_inner(Curve {
            pow,
            frac
        });

        Ok(())
    }
}