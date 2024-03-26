use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::{
    metadata::Metadata,
    associated_token::AssociatedToken,
    token::{
        Mint,
        Token,
        TokenAccount
    }};
use mpl_token_metadata::instructions::{
        BurnV1Cpi,
        BurnV1CpiAccounts,
        BurnV1InstructionArgs,
    };
pub use anchor_lang::solana_program::sysvar::instructions::ID as INSTRUCTIONS_ID;
use crate::state::{Pool, Curve, CreatorFund};

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = seller
    )]
    pub seller_ata: Account<'info, TokenAccount>,

    /// CHECK: not dangerous
    #[account(
        mut,
        seeds = [b"authority"],
        bump
    )]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: will be checked by metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    pub curve: Account<'info, Curve>,
    pub treasury: SystemAccount<'info>,
    pub creator_fund: Account<'info, CreatorFund>,

    #[account(
        mut,
        seeds = [b"pool", mint.key().as_ref()],
        bump = pool.bump,
        has_one = curve,
        has_one = treasury,
        has_one = creator_fund
    )]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = INSTRUCTIONS_ID)]
    /// CHECK: no need to check
    pub sysvar_instructions: UncheckedAccount<'info>
}

impl<'info> Sell<'info> {
    pub fn sell(&mut self, amount_out: u64, num_burn: i16, bumps: &SellBumps) -> Result<()> {

        // u64 all the way, multiply by basis points, NO DECIMALS
        let current_supply = self.mint.supply;

        let mut total = 0.0;

        for i in 0..num_burn {
            let supply = current_supply - (i + 1);
            let price = supply
                .checked_pow(self.curve.pow as f64)?
                .checked_div(self.curve.frac as f64);

            total += price;
        }
        
        // Transfer subtotal
        let accounts = Transfer {
            from: self.pool.to_account_info(),
            to: self.seller.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, total.checked_mult(0.9));

        // Transfer creator fee
        let accounts = Transfer {
            from: self.pool.to_account_info(),
            to: self.creator_fund.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, total.checked_mul(0.05));

        // Transfer Banger fee
        let accounts = Transfer {
            from: self.pool.to_account_info(),
            to: self.treasury.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, total.checked_mul(0.05));

        // Burn tokens from seller
        let seeds = &[
            &b"authority"[..], 
            &[bumps.authority]
        ];
        let signer_seeds = &[&seeds[..]];
        
        let burn_tokens = BurnV1Cpi::new(
            &self.metadata_program.to_account_info(),
            BurnV1CpiAccounts {
                authority: &self.authority.to_account_info(),
                collection_metadata: None,
                metadata: &self.metadata.to_account_info(),
                edition: None,
                mint: &self.mint.to_account_info(),
                token: &self.seller_ata.to_account_info(),
                master_edition: None,
                master_edition_mint: None,
                master_edition_token: None,
                edition_marker: None,
                token_record: None,
                system_program: &self.system_program.to_account_info(),
                sysvar_instructions: &self.sysvar_instructions.to_account_info(),
                spl_token_program: &self.token_program.to_account_info(),
            },
            BurnV1InstructionArgs {
                amount: num_burn as u64
            }
        );
        burn_tokens.invoke_signed(signer_seeds);
        msg!("Tokens burned!");

        Ok(())
    }
}