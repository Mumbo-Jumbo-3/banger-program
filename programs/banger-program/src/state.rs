use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub curve: Pubkey,
    pub creator_fee: u16,
    pub creator_vault: Pubkey,
    pub banger_fee: u16,
    pub treasury: Pubkey,
    pub bump: u8,
    pub authority_bump: u8
}

impl Space for Pool {
    const INIT_SPACE: usize = 8 + 32*3 + 2 + 32 + 2 + 32 + 1 + 1;
}

#[account]
pub struct Curve {
    pub pow: u8,
    pub frac: u8
}

impl Space for Curve {
    const INIT_SPACE: usize = 8 + 1 + 1;
}

#[account]
pub struct CreatorFund {
    pub twitter_id: u64,
    pub creator_key: Option<Pubkey>, // authority to withdraw, default = admin?
}

impl Space for CreatorFund {
    const INIT_SPACE: usize = 8 + 8 + 4 + 32;
}