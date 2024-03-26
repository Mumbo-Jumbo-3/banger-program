use anchor_lang::error_code;

#[error_code]
pub enum CurveError {
    #[msg("failed to do math")]
    Overflow,
    #[msg("slippage limit exceeded")]
    Slippage
}