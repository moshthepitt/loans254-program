use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke},
};
use crate::{instruction::LoanInstruction, error::LoanError, state::Loan};
use crate::{utils::{get_borrowed_amount, get_duration, get_interest_rate}};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        let instruction = LoanInstruction::unpack(instruction_data)?;

        match instruction {
            LoanInstruction::InitLoan { amount } => {
                msg!("Instruction: InitLoan");
                process_init_loan(program_id, accounts, amount)
            }
            LoanInstruction::GuaranteeLoan => {
                msg!("Instruction: GuaranteeLoan");
                process_guarantee_loan(program_id, accounts)
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
        LoanInstruction::InitLoan { amount } => {
            msg!("Instruction: InitLoan");
            process_init_loan(program_id, accounts, amount)
        }
        LoanInstruction::GuaranteeLoan => {
            msg!("Instruction: GuaranteeLoan");
            process_guarantee_loan(program_id, accounts)
        }
    }
}

pub fn process_init_loan(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // get the initializer and assert that they can sign
    let initializer = next_account_info(account_info_iter)?;
    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // get the temp token account owned by the initializer
    let temp_token_account = next_account_info(account_info_iter)?;
    if *temp_token_account.owner != *initializer.key {
        return Err(LoanError::NotAuthorized.into());
    }

    // the account that will receive the loan if it goes through
    // ensure that it is owned by the program
    let token_to_receive_account = next_account_info(account_info_iter)?;
    if *token_to_receive_account.owner != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // next get the loan account.  This will be used to store state/data
    // about the loan.  We need to ensure it is owned by the program
    let loan_account = next_account_info(account_info_iter)?;
    if *loan_account.owner != *program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

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
    loan_info.expected_amount = amount;
    loan_info.interest_rate = get_interest_rate(&initializer.key,  amount);
    loan_info.duration = get_duration(&initializer.key,  amount);
    loan_info.amount = get_borrowed_amount(&initializer.key, amount, loan_info.duration, loan_info.interest_rate);
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

pub fn process_guarantee_loan(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    // get the guarantor and assert that they can sign
    let guarantor_info = next_account_info(account_info_iter)?;
    if !guarantor_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // get the collateral_account owned by the guarantor
    let collateral_account_info = next_account_info(account_info_iter)?;
    if *collateral_account_info.owner != *guarantor_info.key {
        return Err(LoanError::NotAuthorized.into());
    }
    // get the rent sysvar and check if the loan account is rent exempt
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    if !rent.is_exempt(collateral_account_info.lamports(), collateral_account_info.data_len()) {
        return Err(LoanError::NotRentExempt.into());
    }
    // get the loan account and assert that it is owned by the program
    let loan_account_info = next_account_info(account_info_iter)?;
    if *loan_account_info.owner != *program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    // get the loan data
    let mut loan_data = Loan::unpack(&loan_account_info.data.borrow())?;
    // fail is loan is not initialized
    if !loan_data.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    // fail if collateral is not sufficient
    if collateral_account_info.lamports() < loan_data.amount {
        return Err(ProgramError::InsufficientFunds);
    }
    // update loan info
    msg!("Updating loan information...");
    loan_data.is_guaranteed = true;
    loan_data.guarantor_pubkey = Some(*guarantor_info.key).into();
    Loan::pack(loan_data, &mut loan_account_info.data.borrow_mut())?;

    // get the program derived address
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"loan"], program_id);
    // change the owner of the collateral account to be the pda
    // essentially the program now fully controls the loan collateral
    let token_program = next_account_info(account_info_iter)?;
    let owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        collateral_account_info.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        guarantor_info.key,
        &[&guarantor_info.key],
    )?;

    msg!("Calling the token program to transfer collateral account ownership...");
    invoke(
        &owner_change_ix,
        &[
            collateral_account_info.clone(),
            guarantor_info.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}