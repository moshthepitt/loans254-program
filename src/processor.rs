use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke, invoke_signed},
    system_instruction,
};
use crate::{instruction::LoanInstruction, error::LoanError, state::Loan};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        let instruction = LoanInstruction::unpack(instruction_data)?;

        match instruction {
            LoanInstruction::InitLoan { amount: _ } => {
                msg!("Instruction: InitLoan");
                process_init_loan(program_id, accounts, instruction_data)
            }
        }
    }
}

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = LoanInstruction::unpack(instruction_data)?;

    match instruction {
        LoanInstruction::InitLoan { amount: _ } => {
            msg!("Instruction: InitLoan");
            process_init_loan(program_id, accounts, instruction_data)
        }
    }
}

pub fn process_init_loan(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // get the initializer and assert that they can sign
    let initializer = next_account_info(account_info_iter)?;
    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // get the temp token account owned by the initializer
    let temp_token_account = next_account_info(account_info_iter)?;

    // the account that will receive the loan if it goes through
    // ensure that it is owned by the program
    let token_to_receive_account = next_account_info(account_info_iter)?;
    if *token_to_receive_account.owner != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // next get the loan account.  This will be used to store state/data
    // about the loan.  We need to ensure it is rent-exempt
    let loan_account = next_account_info(account_info_iter)?;

    // get the rent sysvar and check if the loan account is rent exempt
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    if !rent.is_exempt(loan_account.lamports(), loan_account.data_len()) {
        return Err(LoanError::NotRentExempt.into());
    }

    // get the loan information
    let mut loan_info = Loan::unpack_unchecked(&loan_account.data.borrow())?;
    if loan_info.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // create the Loan object
    msg!("Saving loan information...");
    loan_info.is_initialized = true;
    loan_info.initializer_pubkey = *initializer.key;
    loan_info.temp_token_account_pubkey = *temp_token_account.key;
    loan_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
    loan_info.expected_amount = 13337;
    Loan::pack(loan_info, &mut loan_account.data.borrow_mut())?;

    // get the program derived address
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"loan"], program_id);
    // change the owner of the temp_token_account to be the pda
    // essentially the program now fully controls the loan application fees
    let token_program = next_account_info(account_info_iter)?;
    let owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        temp_token_account.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        initializer.key,
        &[&initializer.key],
    )?;

    msg!("Calling the token program to transfer token account ownership...");
    invoke(
        &owner_change_ix,
        &[
            temp_token_account.clone(),
            initializer.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}

/// Instruction processor
pub fn process_example(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Create in iterator to safety reference accounts in the slice
    let account_info_iter = &mut accounts.iter();

    // Account info for the program being invoked
    let system_program_info = next_account_info(account_info_iter)?;
    // Account info to allocate
    let allocated_info = next_account_info(account_info_iter)?;

    let expected_allocated_key =
        Pubkey::create_program_address(&[b"You pass butter", &[instruction_data[0]]], program_id)?;
    if *allocated_info.key != expected_allocated_key {
        // allocated key does not match the derived address
        return Err(ProgramError::InvalidArgument);
    }

    // Invoke the system program to allocate account data
    invoke_signed(
        &system_instruction::allocate(allocated_info.key, 42 as u64),
        // Order doesn't matter and this slice could include all the accounts and be:
        // `&accounts`
        &[
            system_program_info.clone(), // program being invoked also needs to be included
            allocated_info.clone(),
        ],
        &[&[b"You pass butter", &[instruction_data[0]]]],
    )?;

    Ok(())
}