use solana_program::{
    pubkey::Pubkey,
};

/// get the loan interest rate
pub fn get_interest_rate(
    _borrower: &Pubkey,
    _loan_amount: u64,
) -> u32 {
    return 9;
}

/// get the loan duration
pub fn get_duration(
    _borrower: &Pubkey,
    _loan_amount: u64,
) -> u32 {
    return 86400;
}