use std::convert::TryInto;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use crate::error::LoanError::InvalidInstruction;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum LoanInstruction {

    /// Start the loan request by paying a loan processing fee into a token account
    /// The token account is then transferred to be owned by the program.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the loan
    /// 1. `[writable]` Token account that should be created prior to this instruction and owned by the initializer
    /// 2. `[]` The initializer's token account for the token they will receive should the loan go through
    /// 3. `[writable]` The loan account, it will hold all necessary info about the loan.  Owned by the program
    /// 4. `[]` The rent sysvar
    /// 5. `[]` The token program
    InitLoan {
        /// The amount party A expects to receive as a loan of token Y
        amount: u64
    },
    /// Guarantee a loan
    ///
    /// Accounts expected:
    ///
    /// Basically meant to be a mechanism through which collateral is provided for a loan
    /// This could be by a third party of by the borrower
    ///
    /// 0. `[signer]` The account of the person guaranteeing the loan
    /// 1. `[writable]` Token account that holds the collateral.  Should be owned by guarantor
    /// 2. `[writable]` Token account to which the guarantor's payment should be sent.
    /// 3. `[writable]` The loan account, has information about the loan
    /// 4. `[]` The rent sysvar
    /// 5. `[]` The token program
    GuaranteeLoan,
    /// Accept the loan
    ///
    /// Accounts expected:
    ///
    /// Basically, sends money to the borrower, from the lender
    /// 0. `[signer]` The account of the person lending the money
    /// 1. `[writable]` Token account that whose funds will be transferred to borrower
    /// 2. `[writable]` The lender's token account for the token they will receive should when loan is repaid
    /// 3. `[writable]` The borrower's token account to receive the borrowed loan amount
    /// 4. `[writable]` The loan account, has information about the loan
    /// 5. `[]` The rent sysvar
    /// 6. `[]` The token program
    AcceptLoan,
    /// Repay the loan
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person repaying the loan
    /// 1. `[writable]` The payer's token account that has the funds being repaid
    /// 2. `[writable]` The guarantor's account
    /// 3. `[writable]` The collateral account to be returned to guarantor
    /// 4. `[writable]` The guarantor's token account to be returned to guarantor
    /// 5. `[writable]` The lender's account
    /// 6. `[writable]` The lender's token account that will receive the repaid loan
    /// 7. `[writable]` The loan account, has information about the loan
    /// 8. `[]` The PDA account
    /// 9. `[]` The token program
    RepayLoan,
}

impl LoanInstruction {
    /// Unpacks a byte buffer into a [LoanInstruction](enum.LoanInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitLoan {
                amount: Self::unpack_amount(rest)?,
            },
            1 => Self::GuaranteeLoan,
            2 => Self::AcceptLoan,
            3 => Self::RepayLoan,
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }

    pub fn pack_into_vec(&self) -> Vec<u8> {
        self.try_to_vec().expect("try_to_vec")
    }
}

/// Creates an 'InitLoan' instruction.
pub fn init_loan(
    program_id: Pubkey,
    initializer_pubkey: Pubkey,
    initializer_temp_token_pubkey: Pubkey,
    initializer_loan_receive_pubkey: Pubkey,
    loan_account_pubkey: Pubkey,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(initializer_pubkey, true),
            AccountMeta::new_readonly(initializer_temp_token_pubkey, false),
            AccountMeta::new_readonly(initializer_loan_receive_pubkey, false),
            AccountMeta::new(loan_account_pubkey, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: LoanInstruction::InitLoan {
            amount,
        }
        .pack_into_vec(),
    }
}

/// Creates an 'GuaranteeLoan' instruction.
pub fn guarantee_loan(
    program_id: Pubkey,
    guarantor_pubkey: Pubkey,
    collateral_account_pubkey: Pubkey,
    guarantor_repayment_pubkey: Pubkey,
    loan_account_pubkey: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(guarantor_pubkey, true),
            AccountMeta::new(collateral_account_pubkey, false),
            AccountMeta::new(guarantor_repayment_pubkey, false),
            AccountMeta::new(loan_account_pubkey, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: LoanInstruction::GuaranteeLoan
        .pack_into_vec(),
    }
}

/// Creates an 'AcceptLoan' instruction.
pub fn accept_loan(
    program_id: Pubkey,
    lender_pubkey: Pubkey,
    lender_loan_transfer_token_pubkey: Pubkey,
    lender_repayment_token_pubkey: Pubkey,
    borrower_loan_receive_pubkey: Pubkey,
    loan_account_pubkey: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(lender_pubkey, true),
            AccountMeta::new_readonly(lender_loan_transfer_token_pubkey, false),
            AccountMeta::new_readonly(lender_repayment_token_pubkey, false),
            AccountMeta::new(loan_account_pubkey, false),
            AccountMeta::new(borrower_loan_receive_pubkey, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: LoanInstruction::AcceptLoan
        .pack_into_vec(),
    }
}
