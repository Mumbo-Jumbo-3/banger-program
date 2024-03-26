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
        MintV1Cpi, MintV1CpiAccounts, MintV1InstructionArgs
    };
pub use anchor_lang::solana_program::sysvar::instructions::ID as INSTRUCTIONS_ID;
use crate::state::{Pool, Curve, CreatorFund};

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer
    )]
    pub buyer_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"authority"],
        pool.authority_bump
    )]
    pub authority: Signer<'info>,

    /// CHECK: will be checked by metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    pub curve: Account<'info, Curve>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    #[account(mut)]
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

impl<'info> Buy<'info> {
    pub fn buy(&mut self, amount_in: u64, num_mint: u64, bumps: &BuyBumps) -> Result<()> {

        // u64 all the way!
        let current_supply = self.mint.supply;

        let mut subtotal = 0.0;

        for i in 0..num_out {
            let supply = current_supply + i;
            let price = supply
                .checked_pow(self.curve.pow as f64)?
                .checked_div(self.curve.frac as f64);

            subtotal += price;
        }

        // add fees to subtotal, then check if <= amount_in
        
        // Transfer subtotal
        let accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.pool.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, subtotal);

        // Transfer creator fee
        let accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.creator_fund.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, subtotal.checked_mul(0.05));

        // Transfer Banger fee
        let accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.treasury.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, subtotal.checked_mul(0.05));

        let seeds = &[
            &b"authority"[..], 
            &[bumps.authority]
        ];
        let signer_seeds = &[&seeds[..]];

        // Mint token to buyer
        let mint_tokens = MintV1Cpi::new(
            &self.metadata_program.to_account_info(),
            MintV1CpiAccounts {
                token: &self.buyer_ata.to_account_info(),
                token_owner: Some(&self.buyer.to_account_info()),
                metadata: &self.metadata.to_account_info(),
                master_edition: None,
                token_record: None,
                mint: &self.mint.to_account_info(),
                authority: &self.authority.to_account_info(),
                delegate_record: None,
                payer: &self.buyer.to_account_info(),
                system_program: &self.system_program.to_account_info(),
                sysvar_instructions: &self.sysvar_instructions.to_account_info(),
                spl_token_program: &self.token_program.to_account_info(),
                spl_ata_program: &self.associated_token_program.to_account_info(),
                authorization_rules_program: None,
                authorization_rules: None
            },
            MintV1InstructionArgs {
                amount: num_mint as u64,
                authorization_data: None
            }
        );
        mint_tokens.invoke_signed(signer_seeds);
        msg!("Tokens minted!");

        Ok(())
    }
}