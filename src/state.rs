use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub struct Loan {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    pub temp_token_account_pubkey: Pubkey,  // this account holds loan processing fee
    pub initializer_token_to_receive_account_pubkey: Pubkey, // loan amount will be sent here if successful
    pub expected_amount: u64,  // the loan amount
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
    const LEN: usize = 113;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Loan::LEN];
        let (
            is_initialized,
            initializer_pubkey,
            temp_token_account_pubkey,
            initializer_token_to_receive_account_pubkey,
            expected_amount,
            interest_rate,
            duration,
        ) = array_refs![src, 1, 32, 32, 32, 8, 4, 4];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Loan {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            temp_token_account_pubkey: Pubkey::new_from_array(*temp_token_account_pubkey),
            initializer_token_to_receive_account_pubkey: Pubkey::new_from_array(*initializer_token_to_receive_account_pubkey),
            expected_amount: u64::from_le_bytes(*expected_amount),
            interest_rate: u32::from_le_bytes(*interest_rate),
            duration: u32::from_le_bytes(*duration),
        })
    }

     fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Loan::LEN];
        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            temp_token_account_pubkey_dst,
            initializer_token_to_receive_account_pubkey_dst,
            expected_amount_dst,
            interest_rate_dst,
            duration_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 8, 4, 4];

        let Loan {
            is_initialized,
            initializer_pubkey,
            temp_token_account_pubkey,
            initializer_token_to_receive_account_pubkey,
            expected_amount,
            interest_rate,
            duration,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        temp_token_account_pubkey_dst.copy_from_slice(temp_token_account_pubkey.as_ref());
        initializer_token_to_receive_account_pubkey_dst.copy_from_slice(initializer_token_to_receive_account_pubkey.as_ref());
        *expected_amount_dst = expected_amount.to_le_bytes();
        *interest_rate_dst = interest_rate.to_le_bytes();
        *duration_dst = duration.to_le_bytes();
    }
}