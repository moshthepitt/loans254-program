use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum LoanError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
}

impl From<LoanError> for ProgramError {
    fn from(e: LoanError) -> Self {
        ProgramError::Custom(e as u32)
    }
}