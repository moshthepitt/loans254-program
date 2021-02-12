// // Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
// #![cfg(feature = "test-bpf")]

// use solana_program::{
//     account_info::{AccountInfo, IntoAccountInfo},
//     clock::Epoch,
//     entrypoint::ProgramResult,
//     instruction::{AccountMeta, Instruction},
//     program_pack::Pack,
//     program::{invoke, invoke_signed},
//     pubkey::Pubkey,
//     rent::Rent,
//     sysvar,
// };
// use solana_sdk::account::{create_account, create_is_signer_account_infos, Account};
// use solana_sdk::{bpf_loader};
// use std::str::FromStr;
// use spl_token::state::{Account as TokenAccount, AccountState, Mint};

// use safe_transmute::to_bytes::{transmute_to_bytes};
// use bumpalo::{vec as bump_vec, Bump};
// use rand::prelude::*;

// use mosh_love_oov::instruction::{LoanInstruction};
// use mosh_love_oov::processor::{Processor};
// use mosh_love_oov::state::{Loan};
// use mosh_love_oov::error::{LoanError};


// fn new_spl_token_program<'bump>(bump: &'bump Bump) -> AccountInfo<'bump> {
//     AccountInfo::new(
//         &spl_token::ID,
//         true,
//         false,
//         bump.alloc(0),
//         &mut [],
//         &bpf_loader::ID,
//         false,
//         Epoch::default(),
//     )
// }

// #[tokio::test]
// async fn test_process_init_loan() {
//     let mut rng = StdRng::seed_from_u64(0);
//     let bump = Bump::new();

//     let account_key = Pubkey::new_unique();
//     let mut account = Account::new(20000000, 105, &account_key);

//     // println!("account1 >> {:#?} ", account);

//     assert_eq!(20000000, account.lamports);
//     assert_eq!(account_key, account.owner);

//     let mint_key = Pubkey::new_unique();

//     let ix = spl_token::instruction::initialize_mint(
//         &spl_token::ID,
//         &account_key,
//         &mint_key,
//         Some(&account_key),
//         2,
//     );

//     let ix = match ix {
//         Result::Ok(_ix) => _ix,
//         Result::Err(err) =>
//             panic!("called `Result::unwrap()` on an `Err` value: {:?}", err),
//     };

//     let account_info = (&account_key, true, &mut account).into_account_info();
//     // let account_info = (&account_key, true, &mut account).into_account_info();

//     let token_program = new_spl_token_program(&bump);

//     let mint = invoke_signed(
//         &ix,
//         &[
//             account_info.clone(),
//             account_info.clone(),
//             account_info.clone(),
//             token_program.clone()
//         ],
//         &[],
//     );
//     // mint.map_err(|_| LoanError::NotRentExempt.into())

//     assert_eq!(20000000, account.lamports);
//     assert_eq!(account_key, account.owner);

//     println!("account2 >> {:#?} ", mint);

//     assert_eq!(20000000, 1);
// }