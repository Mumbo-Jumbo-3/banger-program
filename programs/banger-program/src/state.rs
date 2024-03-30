use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub curve: Pubkey,
    pub treasury: Pubkey,
    pub creator_id: String,
    pub creator_fee: u16,
    pub banger_fee: u16,
    pub bump: u8,
    pub authority_bump: u8
}

impl Space for Pool {
    const INIT_SPACE: usize = 8 + 32*4 + (4+32) + 2*2 + 1*2;
}

#[account]
pub struct Curve {
    pub pow: u64,
    pub frac: u64
}

impl Space for Curve {
    const INIT_SPACE: usize = 8 + 8 + 8;
}
/*
#[account]
pub struct CreatorVault {
    pub twitter_id: u64,
    pub creator_key: Option<Pubkey>, // authority to withdraw, default = admin?
}

impl Space for CreatorVault {
    const INIT_SPACE: usize = 8 + 8 + 4 + 32;
}
*/