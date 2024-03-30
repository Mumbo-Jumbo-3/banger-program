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
use crate::state::{Pool, Curve};

#[derive(Accounts)]
#[instruction(creator_id: String)]
pub struct InitPool<'info> {
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
    pub authority: UncheckedAccount<'info>,

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

    #[account(
        mut,
        seeds = [b"creator_vault", creator_id.as_bytes()],
        bump
    )]
    pub creator_vault: SystemAccount<'info>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    metadata_program: Program<'info, Metadata>,

    #[account(address = INSTRUCTIONS_ID)]
    /// CHECK: no need to check
    pub sysvar_instructions: UncheckedAccount<'info>
}

impl<'info> InitPool<'info> {
    pub fn init_pool(
        &mut self,
        creator_id: String,
        creator_fee: u16,
        banger_fee: u16,
        token_name: String,
        token_metadata_uri: String,
        bumps: &InitPoolBumps
    ) -> Result<()> {
        
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
                name: token_name,
                symbol: "BNGR".to_owned(),
                uri: token_metadata_uri,
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
            treasury: self.treasury.key(),
            creator_id,
            creator_fee,
            banger_fee,
            bump: bumps.pool,
            authority_bump: bumps.authority
        });
        
        Ok(())
    }
}