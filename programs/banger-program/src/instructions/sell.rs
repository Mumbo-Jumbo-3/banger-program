use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata,
    associated_token::AssociatedToken,
    token::{
        Mint,
        Token,
        TokenAccount
    }};
use mpl_token_metadata::instructions::{
        BurnV1Cpi, BurnV1CpiAccounts, BurnV1InstructionArgs, DelegateStandardV1Cpi, DelegateStandardV1CpiAccounts, DelegateStandardV1InstructionArgs
    };
pub use anchor_lang::solana_program::sysvar::instructions::ID as INSTRUCTIONS_ID;
use crate::state::{Pool, Curve};
use crate::errors::CurveError;

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

    #[account(mut)]
    pub treasury: SystemAccount<'info>,

    // CHECK: checked by seeds
    #[account(
        mut,
        seeds = [b"creator_vault", pool.creator_id.as_bytes()],
        bump
    )]
    pub creator_vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"pool", mint.key().as_ref()],
        bump = pool.bump,
        has_one = curve,
        has_one = treasury,
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
    pub fn sell(&mut self, num_burn: u64, amount_out: u64) -> Result<()> {

        let current_supply = self.mint.supply;

        let mut total: u64 = 0;

        for i in 0..num_burn {
            let supply = current_supply - (i + 1);
            let price = supply
                .checked_pow(self.curve.pow as u32).ok_or(CurveError::Overflow)?
                .checked_div(self.curve.frac).ok_or(CurveError::Overflow)?;

            total = total.checked_add(price).ok_or(CurveError::Overflow)?;
        }

        total = total.checked_mul(1_000_000_000).ok_or(CurveError::Overflow)?;

        let banger_fee = total
            .checked_mul(self.pool.banger_fee as u64).ok_or(CurveError::Overflow)?
            .checked_div(10000).ok_or(CurveError::Overflow)?;

        let creator_fee = total
            .checked_mul(self.pool.creator_fee as u64).ok_or(CurveError::Overflow)?
            .checked_div(10000).ok_or(CurveError::Overflow)?;

        let subtotal = total
            .checked_sub(banger_fee).ok_or(CurveError::Overflow)?
            .checked_sub(banger_fee).ok_or(CurveError::Overflow)?;
        
        require!(amount_out >= total, CurveError::Slippage);
        
        /*
        let seeds = &[
            &b"pool"[..],
            &self.mint.to_account_info().key.as_ref(),
            &[self.pool.bump],
        ];

        let signer_seeds = &[&seeds[..]];
        
        // Transfer subtotal to seller
        let accounts = Transfer {
            from: self.pool.to_account_info(),
            to: self.seller.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer(self.system_program.to_account_info(), accounts, signer_seeds);
        transfer(cpi_ctx, subtotal)?;
        */
        msg!("CPI1");

        **self.pool.to_account_info().try_borrow_mut_lamports()? -= subtotal;
        **self.seller.to_account_info().try_borrow_mut_lamports()? += subtotal;
        
        /*
        let accounts = Transfer {
            from: self.pool.to_account_info(),
            to: self.creator_vault.to_account_info()
        };
        let cpi_ctx = CpiContext::new_with_signer(self.system_program.to_account_info(), accounts, signer_seeds);
        transfer(cpi_ctx, creator_fee)?;
        */
        msg!("CPI2");
        **self.pool.to_account_info().try_borrow_mut_lamports()? -= creator_fee;
        **self.creator_vault.to_account_info().try_borrow_mut_lamports()? += creator_fee;
        
        /*
        // Transfer Banger fee
        let accounts = Transfer {
            from: self.pool.to_account_info(),
            to: self.treasury.to_account_info()
        };
        let cpi_ctx = CpiContext::new_with_signer(self.system_program.to_account_info(), accounts, signer_seeds);
        transfer(cpi_ctx, banger_fee)?;
        */
        msg!("CPI3");
        **self.pool.to_account_info().try_borrow_mut_lamports()? -= banger_fee;
        **self.treasury.to_account_info().try_borrow_mut_lamports()? += banger_fee;

        // Burn tokens from seller
        let seeds = &[
            &b"authority"[..], 
            &[self.pool.authority_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let metadata_program = &self.metadata_program.to_account_info();
        let authority = &self.authority.to_account_info();
        let metadata = &self.metadata.to_account_info();
        let mint = &self.mint.to_account_info();
        let seller = &self.seller.to_account_info();
        let token = &self.seller_ata.to_account_info();
        let system_program = &self.system_program.to_account_info();
        let sysvar_instructions = &self.sysvar_instructions.to_account_info();
        let spl_token_program = &self.token_program.to_account_info();

        let delegate_authority = DelegateStandardV1Cpi::new(
            metadata_program,
            DelegateStandardV1CpiAccounts {
                delegate_record: None,
                delegate: authority,
                metadata,
                master_edition: None,
                token_record: None,
                mint,
                token,
                authority: seller,
                payer: seller,
                system_program,
                sysvar_instructions,
                spl_token_program: Some(spl_token_program),
                authorization_rules_program: None,
                authorization_rules: None,
            },
            DelegateStandardV1InstructionArgs {
                amount: self.seller_ata.amount
            }
        );
        delegate_authority.invoke_signed(signer_seeds)?;
        msg!("Delegated authority!");
        
        let burn_tokens = BurnV1Cpi::new(
            metadata_program,
            BurnV1CpiAccounts {
                authority,
                collection_metadata: None,
                metadata,
                edition: None,
                mint,
                token,
                master_edition: None,
                master_edition_mint: None,
                master_edition_token: None,
                edition_marker: None,
                token_record: None,
                system_program,
                sysvar_instructions,
                spl_token_program,
            },
            BurnV1InstructionArgs {
                amount: num_burn as u64
            }
        );
        burn_tokens.invoke_signed(signer_seeds)?;
        msg!("Tokens burned!");

        Ok(())
    }
}