use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_option::COption,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke, invoke_signed},
};
use crate::{instruction::LoanInstruction, error::LoanError, state::{Loan, LoanStatus}};
use crate::{utils::{
    get_application_fee,
    get_borrowed_amount,
    get_duration,
    get_interest_rate,
    get_guarantor_share,
    get_lender_share,
    get_processing_fee,
}};

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
            LoanInstruction::AcceptLoan => {
                msg!("Instruction: AcceptLoan");
                process_accept_loan(program_id, accounts)
            }
            LoanInstruction::RepayLoan => {
                msg!("Instruction: RepayLoan");
                process_repay_loan(program_id, accounts)
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
        LoanInstruction::AcceptLoan => {
            msg!("Instruction: AcceptLoan");
            process_accept_loan(program_id, accounts)
        }
        LoanInstruction::RepayLoan => {
            msg!("Instruction: RepayLoan");
            process_repay_loan(program_id, accounts)
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

    // get the mint account to be used for this loan
    let loan_mint_account = next_account_info(account_info_iter)?;

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
    // fail if loan account does not cover application fee
    let fee = get_application_fee(&initializer.key, amount) * amount as f64;
    if loan_account.lamports() < fee as u64  {
        return Err(ProgramError::InsufficientFunds);
    }

    // get the loan information
    let mut loan_info = Loan::unpack_unchecked(&loan_account.data.borrow())?;
    if loan_info.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // create the Loan object
    msg!("Saving loan information...");
    loan_info.is_initialized = true;
    loan_info.status = LoanStatus::Initialized as u8;
    loan_info.initializer_pubkey = *initializer.key;
    loan_info.loan_mint_pubkey = *loan_mint_account.key;
    loan_info.borrower_loan_receive_pubkey = *token_to_receive_account.key;
    loan_info.expected_amount = amount;
    loan_info.interest_rate = get_interest_rate(&initializer.key,  amount);
    loan_info.duration = get_duration(&initializer.key,  amount);
    loan_info.amount = get_borrowed_amount(&initializer.key, amount, loan_info.duration, loan_info.interest_rate);
    Loan::pack(loan_info, &mut loan_account.data.borrow_mut())?;

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
    // get the collateral_account owned by the guarantor
    let guarantor_payment_account_info = next_account_info(account_info_iter)?;

    // get the loan account and assert that it is owned by the program
    let loan_account_info = next_account_info(account_info_iter)?;
    if *loan_account_info.owner != *program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    // get the rent sysvar and check if the loan account is rent exempt
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    if !rent.is_exempt(loan_account_info.lamports(), loan_account_info.data_len()) {
        return Err(LoanError::NotRentExempt.into());
    }
    // get the loan data
    let mut loan_data = Loan::unpack(&loan_account_info.data.borrow())?;
    // fail if loan is not initialized
    if !loan_data.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    // fail if the status is not LoanStatus::Initialized
    if loan_data.status != LoanStatus::Initialized as u8 {
        return Err(LoanError::InvalidInstruction.into());
    }
    // fail if collateral is not sufficient
    if collateral_account_info.lamports() < loan_data.amount {
        return Err(ProgramError::InsufficientFunds);
    }
    // fail if guarantor_payment_account_info is not rent-exempt
    if !rent.is_exempt(guarantor_payment_account_info.lamports(), guarantor_payment_account_info.data_len()) {
        return Err(LoanError::NotRentExempt.into());
    }
    // update loan info
    msg!("Updating loan information with guarantor details...");
    loan_data.status = LoanStatus::Guaranteed as u8;
    loan_data.guarantor_pubkey = Some(*guarantor_info.key).into();
    loan_data.guarantor_repayment_pubkey = Some(*guarantor_payment_account_info.key).into();
    loan_data.collateral_account_pubkey = Some(*collateral_account_info.key).into();
    Loan::pack(loan_data, &mut loan_account_info.data.borrow_mut())?;
    // get the program derived address
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"loan"], program_id);
    // change the owner of the collateral account to be the pda
    // essentially the program now fully controls the loan collateral
    let token_program = next_account_info(account_info_iter)?;
    let collateral_acc_owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        collateral_account_info.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        guarantor_info.key,
        &[&guarantor_info.key],
    )?;
    msg!("Calling the token program to transfer collateral account ownership...");
    invoke(
        &collateral_acc_owner_change_ix,
        &[
            collateral_account_info.clone(),
            guarantor_info.clone(),
            token_program.clone(),
        ],
    )?;
    // similar change the ownership of the repayment account to be owned by
    // program.  This prevents future errors e.g. we don't want the account
    // deleted by the time we try to repay
    let guarantor_payment_acc_owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        guarantor_payment_account_info.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        guarantor_info.key,
        &[&guarantor_info.key],
    )?;
    msg!("Calling the token program to transfer guarantor payment account ownership...");
    invoke(
        &guarantor_payment_acc_owner_change_ix,
        &[
            guarantor_payment_account_info.clone(),
            guarantor_info.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}

pub fn process_accept_loan(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // get the lender and assert that they can sign
    let lender_info = next_account_info(account_info_iter)?;
    if !lender_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // get the loan transfer account owned by the lender
    let lender_loan_transfer_info = next_account_info(account_info_iter)?;

    // the account that will receive the loan when it is repaid
    let lender_repayment_account_info = next_account_info(account_info_iter)?;
    if *lender_repayment_account_info.owner != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    // the account that will receive the loan when it is repaid
    let borrower_loan_receive_account_info = next_account_info(account_info_iter)?;
    // next get the loan account.  This will be used to store state/data
    // about the loan.  We need to ensure it is owned by the program
    let loan_account_info = next_account_info(account_info_iter)?;
    if *loan_account_info.owner != *program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    // get the rent sysvar and check if the loan account is rent exempt
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    if !rent.is_exempt(loan_account_info.lamports(), loan_account_info.data_len()) {
        return Err(LoanError::NotRentExempt.into());
    }
    // confirm the loan repayment account is rent exempt
    if !rent.is_exempt(lender_repayment_account_info.lamports(), lender_repayment_account_info.data_len()) {
        return Err(LoanError::NotRentExempt.into());
    }
    // get the loan data
    let mut loan_data = Loan::unpack(&loan_account_info.data.borrow())?;
    // fail is loan is not initialized
    if !loan_data.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    // fail if the status is not LoanStatus::Guaranteed
    if loan_data.status != LoanStatus::Guaranteed as u8 {
        return Err(LoanError::InvalidInstruction.into());
    }
    // Ensure we have the right account to send borrowed funds to
    if *borrower_loan_receive_account_info.key != loan_data.borrower_loan_receive_pubkey {
        return Err(LoanError::NotAuthorized.into());
    }
    // fail if loan transfer account balance is not sufficient
    if lender_loan_transfer_info.lamports() < loan_data.expected_amount {
        return Err(ProgramError::InsufficientFunds);
    }
    let amount: u64 = loan_data.expected_amount;
    // update loan info
    msg!("Updating loan information with lender details...");
    loan_data.status = LoanStatus::Accepted as u8;
    loan_data.lender_pubkey = Some(*lender_info.key).into();
    loan_data.lender_repayment_pubkey = Some(*lender_repayment_account_info.key).into();
    Loan::pack(loan_data, &mut loan_account_info.data.borrow_mut())?;
    // change the owner of the loan repayment info account to be the pda
    // essentially the program now fully controls the loan repayment account
    // get the program derived address
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"loan"], program_id);
    let token_program = next_account_info(account_info_iter)?;
    let owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        lender_repayment_account_info.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        lender_info.key,
        &[&lender_info.key],
    )?;
    msg!("Calling the token program to transfer loan repayment account ownership...");
    invoke(
        &owner_change_ix,
        &[
            lender_repayment_account_info.clone(),
            lender_info.clone(),
            token_program.clone(),
        ],
    )?;
    // transfer the funds to the borrower
    let transfer_to_initializer_ix = spl_token::instruction::transfer(
        token_program.key,
        lender_loan_transfer_info.key,
        borrower_loan_receive_account_info.key,
        lender_info.key,
        &[&lender_info.key],
        amount,
    )?;
    msg!("Calling the token program to transfer tokens to the borrower...");
    invoke(
        &transfer_to_initializer_ix,
        &[
            lender_loan_transfer_info.clone(),
            borrower_loan_receive_account_info.clone(),
            lender_info.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}

pub fn process_repay_loan(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    // get the payer and assert that they can sign
    let payer_info = next_account_info(account_info_iter)?;
    if !payer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // get the accounts
    let payer_token_account_info = next_account_info(account_info_iter)?;
    let guarantor_account_info = next_account_info(account_info_iter)?;
    let collateral_token_account_info = next_account_info(account_info_iter)?;
    let guarantor_token_account_info = next_account_info(account_info_iter)?;
    let lender_account_info = next_account_info(account_info_iter)?;
    let lender_token_account_info = next_account_info(account_info_iter)?;
    let loan_account_info = next_account_info(account_info_iter)?;

    // get the loan data
    let mut loan_data = Loan::unpack(&loan_account_info.data.borrow())?;
    // fail is loan is not initialized
    if !loan_data.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    // fail if the status is not LoanStatus::Accepted
    if loan_data.status != LoanStatus::Accepted as u8 {
        return Err(LoanError::InvalidInstruction.into());
    }
    // fail if repayment transfer account balance is not sufficient
    if payer_token_account_info.lamports() < loan_data.amount {
        return Err(ProgramError::InsufficientFunds);
    }
    // Ensure we have the right account to send guarantor funds to
    let guarantor_account_option = Some(*guarantor_account_info.key);
    let guarantor_account_c_option: COption<Pubkey> = guarantor_account_option.into();
    if guarantor_account_c_option != loan_data.guarantor_pubkey {
        return Err(LoanError::NotAuthorized.into());
    }
    let guarantor_token_account_option = Some(*guarantor_token_account_info.key);
    let guarantor_token_account_c_option: COption<Pubkey> = guarantor_token_account_option.into();
    if guarantor_token_account_c_option != loan_data.guarantor_repayment_pubkey {
        return Err(LoanError::NotAuthorized.into());
    }
    let collateral_token_account_option = Some(*collateral_token_account_info.key);
    let collateral_token_account_c_option: COption<Pubkey> = collateral_token_account_option.into();
    if collateral_token_account_c_option != loan_data.collateral_account_pubkey {
        return Err(LoanError::NotAuthorized.into());
    }
    // Ensure we have the right account to send repaid funds to
    let lender_account_option = Some(*lender_account_info.key);
    let lender_account_c_option: COption<Pubkey> = lender_account_option.into();
    if lender_account_c_option != loan_data.lender_pubkey {
        return Err(LoanError::NotAuthorized.into());
    }
    let lender_token_option = Some(*lender_token_account_info.key);
    let lender_token_c_option: COption<Pubkey> = lender_token_option.into();
    if lender_token_c_option != loan_data.lender_repayment_pubkey {
        return Err(LoanError::NotAuthorized.into());
    }
    // calculate repayments
    let loan_interest = (loan_data.amount - loan_data.expected_amount) as f64;
    let program_share = loan_interest as f64 * get_processing_fee(
        &loan_data.initializer_pubkey,
        loan_data.expected_amount,
        loan_data.duration,
        loan_data.interest_rate
    ) as f64 / 100 as f64;
    let lender_share = (loan_interest - program_share) * (get_lender_share(lender_account_info.key, loan_data.amount) as f64 / 100 as f64);
    let total_lender_share = lender_share as u64 + loan_data.expected_amount;
    let guarantor_share = (loan_interest - program_share) * (get_guarantor_share(guarantor_account_info.key, loan_data.amount) as f64 / 100 as f64);
    // update loan info
    msg!("Updating loan information, setting status to repaid...");
    loan_data.status = LoanStatus::Repaid as u8;
    Loan::pack(loan_data, &mut loan_account_info.data.borrow_mut())?;

    // get the pda and token acconts
    let pda_account_info = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    // transfer the funds to the guarantor repayment account
    let transfer_to_guarantor_ix = spl_token::instruction::transfer(
        token_program.key,
        payer_token_account_info.key,
        guarantor_token_account_info.key,
        payer_info.key,
        &[&payer_info.key],
        guarantor_share as u64,
    )?;
    msg!("Calling the token program to transfer funds to the guarantor payment account...");
    invoke(
        &transfer_to_guarantor_ix,
        &[
            payer_token_account_info.clone(),
            guarantor_token_account_info.clone(),
            payer_info.clone(),
            token_program.clone(),
        ],
    )?;

    // transfer the funds to the lender repayment account
    let transfer_to_lender_ix = spl_token::instruction::transfer(
        token_program.key,
        payer_token_account_info.key,
        lender_token_account_info.key,
        payer_info.key,
        &[&payer_info.key],
        total_lender_share,
    )?;
    msg!("Calling the token program to transfer funds to the lender payment account...");
    invoke(
        &transfer_to_lender_ix,
        &[
            payer_token_account_info.clone(),
            lender_token_account_info.clone(),
            payer_info.clone(),
            token_program.clone(),
        ],
    )?;
    // get pda and nonce
    let (pda, nonce) = Pubkey::find_program_address(&[b"loan"], program_id);
    // change the owner of the collateral account to be the original guarantor
    let return_collateral_ix = spl_token::instruction::set_authority(
        token_program.key,
        collateral_token_account_info.key,
        Some(guarantor_account_info.key),
        spl_token::instruction::AuthorityType::AccountOwner,
        &pda,
        &[&pda],
    )?;
    msg!("Calling the token program to return collateral account to the guarantor...");
    invoke_signed(
        &return_collateral_ix,
        &[
            collateral_token_account_info.clone(),
            guarantor_account_info.clone(),
            pda_account_info.clone(),
            token_program.clone(),
        ],
        &[&[&b"loan"[..], &[nonce]]],
    )?;
    // change the owner of the guarantor payment account to be the original guarantor
    let pay_guarantor_ix = spl_token::instruction::set_authority(
        token_program.key,
        guarantor_token_account_info.key,
        Some(guarantor_account_info.key),
        spl_token::instruction::AuthorityType::AccountOwner,
        &pda,
        &[&pda],
    )?;
    msg!("Calling the token program to return guarantor payment account to the guarantor...");
    invoke_signed(
        &pay_guarantor_ix,
        &[
            guarantor_token_account_info.clone(),
            guarantor_account_info.clone(),
            pda_account_info.clone(),
            token_program.clone(),
        ],
        &[&[&b"loan"[..], &[nonce]]],
    )?;
    // change the owner of the lender payment account to be the original lender
    let pay_lender_ix = spl_token::instruction::set_authority(
        token_program.key,
        lender_token_account_info.key,
        Some(lender_account_info.key),
        spl_token::instruction::AuthorityType::AccountOwner,
        &pda,
        &[&pda],
    )?;
    msg!("Calling the token program to return lender payment account to the lender...");
    invoke_signed(
        &pay_lender_ix,
        &[
            lender_token_account_info.clone(),
            lender_account_info.clone(),
            pda_account_info.clone(),
            token_program.clone(),
        ],
        &[&[&b"loan"[..], &[nonce]]],
    )?;

    // TODO: transfer application fee + program share to program owner address

    Ok(())
}
