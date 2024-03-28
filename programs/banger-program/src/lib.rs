use anchor_lang::prelude::*;
mod state;
mod instructions;
mod errors;

use instructions::*;

declare_id!("CYu4zKWC3tNGaJkime2PagfjPCski5YAvS8DwhX25d3o");

#[program]
pub mod banger_program {
    use super::*;

    pub fn init_curve(
        ctx: Context<InitCurve>,
        pow: u64,
        frac: u64,
    ) -> Result<()> {
        ctx.accounts.init_curve(pow, frac)
    }

    pub fn init_pool(
        ctx: Context<InitPool>,
        creator_fee: u16,
        banger_fee: u16,
        token_name: String,
        token_metadata_uri: String,
        creator_id: String
    ) -> Result<()> {
        ctx.accounts.init_pool(creator_fee, banger_fee, token_name, token_metadata_uri, creator_id, &ctx.bumps)
    }

    pub fn buy(
        ctx: Context<Buy>,
        amount_in: u64,
        num_mint: u64,
    ) -> Result<()> {
        ctx.accounts.buy(amount_in, num_mint)
    }

    pub fn sell(
        ctx: Context<Sell>,
        num_burn: u64,
        amount_out: u64,
    ) -> Result<()> {
        ctx.accounts.sell(num_burn, amount_out)
    }
}
