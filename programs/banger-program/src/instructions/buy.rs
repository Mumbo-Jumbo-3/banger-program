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
use crate::state::{Pool, Curve};
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

    /// CHECK: used for signing
    #[account(
        mut,
        seeds = [b"authority"],
        bump = pool.authority_bump
    )]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: will be checked by metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    pub curve: Account<'info, Curve>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,

    // CHECK: Checked by seeds
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
        has_one = treasury
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
            let supply_str = supply.to_string();
            let supply_str_ref = supply_str.as_str();
            msg!(supply_str_ref);
            let price = supply
                .checked_pow(self.curve.pow as u32).ok_or(CurveError::Overflow)?
                .checked_div(self.curve.frac).ok_or(CurveError::Overflow)?;

            subtotal = subtotal.checked_add(price).ok_or(CurveError::Overflow)?;
        }
        let subtotal1_str = subtotal.to_string();
        let subtotal1_str_ref = subtotal1_str.as_ref();
        msg!(subtotal1_str_ref);

        subtotal = subtotal.checked_mul(1_000_000_000).ok_or(CurveError::Overflow)?;

        let banger_fee = subtotal
            .checked_mul(self.pool.banger_fee as u64).ok_or(CurveError::Overflow)?
            .checked_div(10000).ok_or(CurveError::Overflow)?;

        let creator_fee = subtotal
            .checked_mul(self.pool.creator_fee as u64).ok_or(CurveError::Overflow)?
            .checked_div(10000).ok_or(CurveError::Overflow)?;

        let total = subtotal
            .checked_add(banger_fee).ok_or(CurveError::Overflow)?
            .checked_add(creator_fee).ok_or(CurveError::Overflow)?;

        let current_supply_str = current_supply.to_string();
        let current_supply_str_ref = current_supply_str.as_str();

        let amount_in_str = amount_in.to_string();
        let amount_in_str_ref = amount_in_str.as_str();

        let num_mint_str = num_mint.to_string();
        let num_mint_str_ref = num_mint_str.as_str();

        let pow_str = self.curve.pow.to_string();
        let pow_str_ref = pow_str.as_ref();

        let frac_str = self.curve.frac.to_string();
        let frac_str_ref = frac_str.as_ref();

        let subtotal_str = subtotal.to_string();
        let subtotal_str_ref = subtotal_str.as_ref();

        let creator_fee_str = creator_fee.to_string();
        let creator_fee_str_ref = creator_fee_str.as_ref();                
        
        let total_str = total.to_string();
        let total_str_ref = total_str.as_ref();

        msg!(current_supply_str_ref);
        msg!(amount_in_str_ref);
        msg!(num_mint_str_ref);
        msg!(pow_str_ref);
        msg!(frac_str_ref);
        msg!(subtotal_str_ref);
        msg!(creator_fee_str_ref);
        msg!(total_str_ref);
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