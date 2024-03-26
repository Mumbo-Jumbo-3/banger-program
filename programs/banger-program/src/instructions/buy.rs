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
use crate::errors::CurveError;

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
        bump = pool.authority_bump
    )]
    pub authority: Signer<'info>,

    /// CHECK: will be checked by metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    pub curve: Account<'info, Curve>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    #[account(mut)]
    pub creator_vault: Account<'info, CreatorFund>,

    #[account(
        mut,
        seeds = [b"pool", mint.key().as_ref()],
        bump = pool.bump,
        has_one = curve,
        has_one = treasury,
        has_one = creator_vault
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
    pub fn buy(&mut self, amount_in: u64, num_mint: u64) -> Result<()> {

        let current_supply = self.mint.supply;

        let mut subtotal: u64 = 0;

        for i in 0..num_mint {
            let supply = current_supply + i;
            let price = supply
                .checked_pow(self.curve.pow as u32).ok_or(CurveError::Overflow)?
                .checked_div(self.curve.frac as u64).ok_or(CurveError::Overflow)?;

            subtotal = subtotal.checked_add(price).ok_or(CurveError::Overflow)?;
        }

        subtotal = subtotal.checked_mul(1_000_000_000).ok_or(CurveError::Overflow)?;

        let banger_fee = subtotal
            .checked_mul(self.pool.banger_fee as u64).ok_or(CurveError::Overflow)?
            .checked_div(10000).ok_or(CurveError::Overflow)?;

        let creator_fee = subtotal
            .checked_mul(self.pool.creator_fee as u64).ok_or(CurveError::Overflow)?
            .checked_div(10000).ok_or(CurveError::Overflow)?;

        let total = subtotal
            .checked_add(banger_fee).ok_or(CurveError::Overflow)?
            .checked_add(banger_fee).ok_or(CurveError::Overflow)?;
        
        require!(amount_in <= total, CurveError::Slippage);

        // Transfer subtotal to pool
        let accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.pool.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, subtotal)?;

        // Transfer creator fee
        let accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.creator_vault.to_account_info()
        };
        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(cpi_ctx, creator_fee)?;

        // Transfer Banger fee
        let accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.treasury.to_account_info()
        };
        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(cpi_ctx, banger_fee)?;

        let seeds = &[
            &b"authority"[..], 
            &[self.pool.authority_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let metadata_program = &self.metadata_program.to_account_info();
        let token = &self.buyer_ata.to_account_info();
        let token_owner = &self.buyer.to_account_info();
        let metadata = &self.metadata.to_account_info();
        let mint = &self.mint.to_account_info();
        let authority = &self.authority.to_account_info();
        let payer = &self.buyer.to_account_info();
        let system_program = &self.system_program.to_account_info();
        let sysvar_instructions = &self.sysvar_instructions.to_account_info();
        let spl_token_program = &self.token_program.to_account_info();
        let spl_ata_program = &self.associated_token_program.to_account_info();

        // Mint token to buyer
        let mint_tokens = MintV1Cpi::new(
            metadata_program,
            MintV1CpiAccounts {
                token,
                token_owner: Some(token_owner),
                metadata,
                master_edition: None,
                token_record: None,
                mint,
                authority,
                delegate_record: None,
                payer,
                system_program,
                sysvar_instructions,
                spl_token_program,
                spl_ata_program,
                authorization_rules_program: None,
                authorization_rules: None
            },
            MintV1InstructionArgs {
                amount: num_mint as u64,
                authorization_data: None
            }
        );
        mint_tokens.invoke_signed(signer_seeds)?;
        msg!("Tokens minted!");

        Ok(())
    }
}