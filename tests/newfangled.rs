// Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
#![cfg(feature = "test-bpf")]

use solana_program::{program_pack::Pack};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::{Transaction},
};
use spl_token::{
    state::{Mint},
};

use solana_program_test::{processor, ProgramTest};
use std::str::FromStr;

use mosh_love_oov::instruction::{init_loan};
use mosh_love_oov::processor::{process_instruction};
use mosh_love_oov::state::{Loan};

#[tokio::test]
async fn test_diff_init_loan() {
    let program_id = Pubkey::from_str(&"invoker111111111111111111111111111111111111").unwrap();
    let mut test = ProgramTest::new(
        "mosh_love_oov",
        program_id,
        processor!(process_instruction),
    );

    let borrower = Keypair::new();
    let borrower_pubkey = borrower.pubkey();

    let (mut banks_client, payer, _recent_blockhash) = test.start().await;

    let temp_token_keypair = Keypair::new();
    let loan_receive_keypair = Keypair::new();
    let loan_account_keypair = Keypair::new();

    let rent = banks_client.get_rent().await.unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[
            create_account(
                &payer.pubkey(),
                &temp_token_keypair.pubkey(),
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            create_account(
                &payer.pubkey(),
                &loan_receive_keypair.pubkey(),
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            create_account(
                &payer.pubkey(),
                &loan_account_keypair.pubkey(),
                rent.minimum_balance(Loan::LEN),
                Loan::LEN as u64,
                &program_id,
            ),
            init_loan(
                program_id,
                borrower_pubkey,
                temp_token_keypair.pubkey(),
                loan_receive_keypair.pubkey(),
                loan_account_keypair.pubkey(),
                13337
            ),
        ],
        Some(&payer.pubkey()),
    );
}