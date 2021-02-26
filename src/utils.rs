use solana_program::{
    program_option::COption,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use arrayref::{array_refs, mut_array_refs};

/// get the loan interest rate
pub fn get_interest_rate(
    _borrower: &Pubkey,
    _loan_amount: u64,
) -> u32 {
    return 9;  // 9%
}

/// get the share paid out to the guarantor
pub fn get_guarantor_share(
    _guarantor: &Pubkey,
    _loan_amount: u64,
) -> u32 {
    return 50;  // 50%
}

/// get the share paid our to the lender
pub fn get_lender_share(
    _lender: &Pubkey,
    _loan_amount: u64,
) -> u32 {
    return 50;  // 50%
}

/// get the loan duration
pub fn get_duration(
    _borrower: &Pubkey,
    _loan_amount: u64,
) -> u32 {
    return 24 * 30;  // 30 days
}

/// get the loan processing fee
pub fn get_processing_fee(
    _borrower: &Pubkey,
    _expected_amount: u64,
    _loan_duration: u32,
    _loan_interest: u32,
) -> u32 {
    return 1;  // 1%
}

/// get the loan application fee
pub fn get_application_fee(
    _borrower: &Pubkey,
    _expected_amount: u64,
) -> u64 {
    return 1;  // 1%
}

/// get the loan duration
pub fn get_borrowed_amount(
    borrower: &Pubkey,
    expected_amount: u64,
    loan_duration: u32,
    loan_interest: u32,
) -> u64 {
    let processing_fee: u32 = get_processing_fee(borrower, expected_amount, loan_duration, loan_interest);
    let total_charge = loan_interest + processing_fee;
    return (u64::from(loan_duration) / (24 * 365)) * ((u64::from(total_charge) / 100) + 1) * expected_amount;
}

// Helpers
pub fn pack_coption_key(src: &COption<Pubkey>, dst: &mut [u8; 36]) {
    let (tag, body) = mut_array_refs![dst, 4, 32];
    match src {
        COption::Some(key) => {
            *tag = [1, 0, 0, 0];
            body.copy_from_slice(key.as_ref());
        }
        COption::None => {
            *tag = [0; 4];
        }
    }
}

pub fn unpack_coption_key(src: &[u8; 36]) -> Result<COption<Pubkey>, ProgramError> {
    let (tag, body) = array_refs![src, 4, 32];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
        _ => Err(ProgramError::InvalidAccountData),
    }
}
