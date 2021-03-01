// Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
#![cfg(feature = "test-bpf")]

use solana_program::{
    account_info::AccountInfo,
    clock::Epoch,
    entrypoint::ProgramResult,
    instruction::{Instruction},
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
};
use solana_sdk::account::{create_account, create_is_signer_account_infos, Account};
use solana_sdk::{bpf_loader};
use std::str::FromStr;
use spl_token::state::{Account as TokenAccount, AccountState, Mint};

use safe_transmute::to_bytes::{transmute_to_bytes};
use bumpalo::{vec as bump_vec, Bump};
use rand::prelude::*;

use mosh_love_oov::instruction::{init_loan};
use mosh_love_oov::processor::{Processor};
use mosh_love_oov::state::{Loan, LoanStatus};

fn do_process_instruction(
    instruction: Instruction,
    accounts: Vec<&mut Account>,
) -> ProgramResult {
    let mut meta = instruction
        .accounts
        .iter()
        .zip(accounts)
        .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
        .collect::<Vec<_>>();

    let account_infos = create_is_signer_account_infos(&mut meta);
    Processor::process(&instruction.program_id, &account_infos, &instruction.data)
}

fn rent_sysvar() -> Account {
    create_account(&Rent::default(), 42)
}

fn random_pubkey<'bump, G: rand::Rng>(_rng: &mut G, bump: &'bump Bump) -> &'bump Pubkey {
    bump.alloc(Pubkey::new(transmute_to_bytes(&rand::random::<[u64; 4]>())))
}

fn new_token_mint<'bump, Gen: Rng>(rng: &mut Gen, bump: &'bump Bump) -> AccountInfo<'bump> {
    let data = bump_vec![in bump; 0u8; Mint::LEN].into_bump_slice_mut();
    let mut mint = Mint::default();
    mint.is_initialized = true;
    Mint::pack(mint, data).unwrap();
    AccountInfo::new(
        random_pubkey(rng, bump),
        false,
        true,
        bump.alloc(0),
        data,
        &spl_token::ID,
        false,
        Epoch::default(),
    )
}

fn new_token_account<'bump, Gen: Rng>(
    rng: &mut Gen,
    mint_pubkey: &'bump Pubkey,
    owner_pubkey: &'bump Pubkey,
    bump: &'bump Bump,
) -> AccountInfo<'bump> {
    let data = bump_vec![in bump; 0u8; TokenAccount::LEN].into_bump_slice_mut();
    let mut account = TokenAccount::default();
    account.state = AccountState::Initialized;
    account.mint = *mint_pubkey;
    account.owner = *owner_pubkey;
    TokenAccount::pack(account, data).unwrap();
    AccountInfo::new(
        random_pubkey(rng, bump),
        false,
        true,
        bump.alloc(0),
        data,
        &spl_token::ID,
        false,
        Epoch::default(),
    )
}

fn new_spl_token_program<'bump>(bump: &'bump Bump) -> AccountInfo<'bump> {
    AccountInfo::new(
        &spl_token::ID,
        true,
        false,
        bump.alloc(0),
        &mut [],
        &bpf_loader::ID,
        false,
        Epoch::default(),
    )
}

#[tokio::test]
async fn test_process_init_loan() {
    let mut rng = StdRng::seed_from_u64(0);
    let bump = Bump::new();

    let program_id = Pubkey::from_str(&"mosh111111111111111111111111111111111111111").unwrap();
    let account_key = Pubkey::new_unique();
    // let temp_token_key = Pubkey::new_unique();
    let loan_acc_key = Pubkey::new_unique();

    let coin_mint = new_token_mint(&mut rng, &bump);
    let temp_token_vault = new_token_account(&mut rng, &coin_mint.key, &account_key, &bump);
    let receiving_token_vault = new_token_account(&mut rng, &coin_mint.key, &account_key, &bump);

    let mut account_account = Account::new(2000000, Loan::LEN, &account_key);
    let mut token_acc = Account::new(
        Rent::default().minimum_balance(Loan::LEN),
        Loan::LEN,
        &account_key,
    );
    let mut receiving_account = Account::new(
        Rent::default().minimum_balance(Loan::LEN),
        Loan::LEN,
        &spl_token::ID,
    );
    let mut loan_acc = Account::new(
        Rent::default().minimum_balance(Loan::LEN),
        Loan::LEN,
        &program_id,
    );
    let mut rent_sysvar = rent_sysvar();
    let spl_token_program = new_spl_token_program(&bump);
    let mut token_program = Account::new(
        Rent::default().minimum_balance(Loan::LEN),
        Loan::LEN,
        &spl_token_program.key,
    );

    println!("BEFORE >> {:?} ", token_acc.owner);

    assert_eq!(token_acc.owner, account_key);

    do_process_instruction(
        init_loan(
            program_id,
            account_key,
            *temp_token_vault.key,
            *receiving_token_vault.key,
            loan_acc_key,
            13337,
        ),
        vec![
            &mut account_account,
            &mut token_acc,
            &mut receiving_account,
            &mut loan_acc,
            &mut rent_sysvar,
            &mut token_program,
        ],
    )
    .unwrap();

    // println!("AFTER >> {:?} ", token_acc.owner);

    // assert_ne!(token_acc.owner, account_key);

    let loan_data = Loan::unpack(&loan_acc.data);
    let loan_data = match loan_data {
        Ok(data) => data,
        Err(error) => panic!("Problem: {:?}", error),
    };
    assert_eq!(true, loan_data.is_initialized);
    assert_eq!(account_key, loan_data.initializer_pubkey);
    assert_eq!(*temp_token_vault.key, loan_data.loan_mint_pubkey);
    assert_eq!(*receiving_token_vault.key, loan_data.borrower_loan_receive_pubkey);
    assert_eq!(13337, loan_data.expected_amount);
    assert_eq!(9, loan_data.interest_rate);
    assert_eq!(24 * 30, loan_data.duration);
    assert_eq!(LoanStatus::Initialized as u8, loan_data.status);
    assert_eq!(13446, loan_data.amount);

    // let option = Some(account_key);
    // let c_option: COption<Pubkey> = option.into();
    assert_eq!(false, loan_data.guarantor_pubkey.is_some());
    assert_eq!(false, loan_data.lender_pubkey.is_some());
    assert_eq!(false, loan_data.lender_repayment_pubkey.is_some());
}
