use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata, token::{
        Mint,
        Token,
    }
};

use mpl_token_metadata::{
    instructions::{
        CreateV1Cpi,
        CreateV1CpiAccounts,
        CreateV1InstructionArgs
    }, 
    types::{
        Creator,
        TokenStandard
    }
};
pub use anchor_lang::solana_program::sysvar::instructions::ID as INSTRUCTIONS_ID;
use crate::state::{Pool, Curve, CreatorFund};

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 0,
        mint::authority = authority
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: not dangerous
    #[account(
        mut,
        seeds = [b"authority"],
        bump
    )]
    pub authority: Signer<'info>,

    /// CHECK: will be checked by metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    pub curve: Account<'info, Curve>,

    #[account(
        init,
        payer = admin,
        space = Pool::INIT_SPACE,
        seeds = [b"pool", mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    pub treasury: SystemAccount<'info>,
    // init_if_needed for now
    // must init if first tweet by creator / creatorfund not initialized yet
    pub creator_vault: Account<'info, CreatorFund>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    metadata_program: Program<'info, Metadata>,

    #[account(address = INSTRUCTIONS_ID)]
    /// CHECK: no need to check
    pub sysvar_instructions: UncheckedAccount<'info>
}

impl<'info> Init<'info> {
    pub fn init(&mut self, creator_fee: u16, banger_fee: u16, bumps: &InitBumps) -> Result<()> {
        
        let metadata = &self.metadata.to_account_info();
        let mint = &self.mint.to_account_info();
        let authority = &self.authority.to_account_info();
        let payer = &self.admin.to_account_info();
        let system_program = &self.system_program.to_account_info();
        let spl_token_program = &self.token_program.to_account_info();
        let spl_metadata_program = &self.metadata_program.to_account_info();
        let sysvar_instructions = &self.sysvar_instructions.to_account_info();

        let creator = vec![
            Creator {
                address: self.authority.key().clone(),
                verified: true,
                share: 100,
            },
        ];

        let seeds = &[
            &b"authority"[..], 
            &[bumps.authority]
        ];
        let signer_seeds = &[&seeds[..]];

        let metadata_account = CreateV1Cpi::new(
            spl_metadata_program,
            CreateV1CpiAccounts {
                metadata,
                master_edition: None,
                mint: (mint, false),
                authority,
                payer,
                update_authority: (authority, true),
                system_program,
                sysvar_instructions,
                spl_token_program: Some(spl_token_program)
            },
            CreateV1InstructionArgs {
                name: "Test Banger".to_owned(),
                symbol: "BNGR".to_owned(),
                uri: "".to_owned(),
                seller_fee_basis_points: 0,
                creators: Some(creator),
                primary_sale_happened: true,
                is_mutable: true,
                token_standard: TokenStandard::FungibleAsset,
                collection: None,
                uses: None,
                collection_details: None,
                rule_set: None,
                decimals: None,
                print_supply: None
            }
        );
        metadata_account.invoke_signed(signer_seeds)?;
        msg!("Metadata Account created!");

        // Initialize pool
        self.pool.set_inner(Pool {
            admin: self.admin.key(),
            mint: self.mint.key(),
            curve: self.curve.key(),
            creator_fee,
            creator_vault: self.creator_vault.key(),
            banger_fee,
            treasury: self.treasury.key(),
            bump: bumps.pool,
            authority_bump: bumps.authority
        });
        
        Ok(())
    }
}