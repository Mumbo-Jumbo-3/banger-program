use anchor_lang::prelude::*;
mod state;
mod instructions;
use instructions::*;

declare_id!("CYu4zKWC3tNGaJkime2PagfjPCski5YAvS8DwhX25d3o");

#[program]
pub mod banger_program {
    use super::*;

    pub fn init(ctx: Context<Init>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
