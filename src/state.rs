use solana_program::{
    program_error::ProgramError,
    program_option::COption,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use crate::utils::{pack_coption_key, unpack_coption_key};

pub enum LoanStatus {
    Pending = 0,
    Initialized = 1,
    Guaranteed = 2,
    Accepted = 3,
    Cancelled = 4,
}

pub struct Loan {
    pub is_initialized: bool,
    pub status: u8,  // the loan status
    pub initializer_pubkey: Pubkey,  // the account that wants to borrow
    pub temp_token_account_pubkey: Pubkey,  // this account holds loan processing fee
    pub borrower_loan_receive_pubkey: Pubkey, // loan amount will be sent here if successful
    pub guarantor_pubkey: COption<Pubkey>, // the person providing collateral for the loans
    pub lender_pubkey: COption<Pubkey>, // the person providing the loans
    pub lender_loan_repayment_pubkey: COption<Pubkey>, // the person providing the loans
    pub expected_amount: u64,  // the expected loan amount
    pub amount: u64,  // the loan amount including interest
    pub interest_rate: u32,  // the loan interest rate annualized.  Note that this is an unsigned int so something like 9 would actually represent 9/100 interest rate
    pub duration: u32,  // the loan duration in seconds
}

impl Sealed for Loan {}

impl IsInitialized for Loan {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Loan {
    const LEN: usize = 230;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Loan::LEN];
        let (
            is_initialized,
            status,
            initializer_pubkey,
            temp_token_account_pubkey,
            borrower_loan_receive_pubkey,
            guarantor_pubkey,
            lender_pubkey,
            lender_loan_repayment_pubkey,
            expected_amount,
            amount,
            interest_rate,
            duration,
        ) = array_refs![src, 1, 1, 32, 32, 32, 36, 36, 36, 8, 8, 4, 4];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Loan {
            is_initialized,
            status: u8::from_le_bytes(*status),
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            temp_token_account_pubkey: Pubkey::new_from_array(*temp_token_account_pubkey),
            borrower_loan_receive_pubkey: Pubkey::new_from_array(*borrower_loan_receive_pubkey),
            guarantor_pubkey: unpack_coption_key(guarantor_pubkey)?,
            lender_pubkey: unpack_coption_key(lender_pubkey)?,
            lender_loan_repayment_pubkey: unpack_coption_key(lender_loan_repayment_pubkey)?,
            expected_amount: u64::from_le_bytes(*expected_amount),
            amount: u64::from_le_bytes(*amount),
            interest_rate: u32::from_le_bytes(*interest_rate),
            duration: u32::from_le_bytes(*duration),
        })
    }

     fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Loan::LEN];
        let (
            is_initialized_dst,
            status_dst,
            initializer_pubkey_dst,
            temp_token_account_pubkey_dst,
            borrower_loan_receive_pubkey_dst,
            guarantor_pubkey_dst,
            lender_pubkey_dst,
            lender_loan_repayment_pubkey_dst,
            expected_amount_dst,
            amount_dst,
            interest_rate_dst,
            duration_dst,
        ) = mut_array_refs![dst, 1, 1, 32, 32, 32, 36, 36, 36, 8, 8, 4, 4];

        let Loan {
            is_initialized,
            status,
            initializer_pubkey,
            temp_token_account_pubkey,
            borrower_loan_receive_pubkey,
            guarantor_pubkey,
            lender_pubkey,
            lender_loan_repayment_pubkey,
            expected_amount,
            amount,
            interest_rate,
            duration,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        *status_dst = status.to_le_bytes();
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        temp_token_account_pubkey_dst.copy_from_slice(temp_token_account_pubkey.as_ref());
        borrower_loan_receive_pubkey_dst.copy_from_slice(borrower_loan_receive_pubkey.as_ref());
        pack_coption_key(guarantor_pubkey, guarantor_pubkey_dst);
        pack_coption_key(lender_pubkey, lender_pubkey_dst);
        pack_coption_key(lender_loan_repayment_pubkey, lender_loan_repayment_pubkey_dst);
        *expected_amount_dst = expected_amount.to_le_bytes();
        *amount_dst = amount.to_le_bytes();
        *interest_rate_dst = interest_rate.to_le_bytes();
        *duration_dst = duration.to_le_bytes();
    }
}