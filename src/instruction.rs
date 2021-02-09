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

    /// Start the loan request by paying a loan processing fee into a temp account'
    /// The temp account is then transferred to be owned by the program.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the loan
    /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 2. `[]` The initializer's token account for the token they will receive should the loan go through
    /// 3. `[writable]` The loan account, it will hold all necessary info about the loan.
    /// 4. `[]` The rent sysvar
    /// 5. `[]` The token program
    InitLoan {
        /// The amount party A expects to receive as a loan of token Y
        amount: u64
    }
}

impl LoanInstruction {
    /// Unpacks a byte buffer into a [LoanInstruction](enum.LoanInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitLoan {
                amount: Self::unpack_amount(rest)?,
            },
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