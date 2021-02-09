use solana_program_test::*;
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_sdk::{
    account::Account,
};
use spl_token::{
    instruction::approve,
    state::{Account as Token, AccountState, Mint},
};

trait AddPacked {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    );
}

impl AddPacked for ProgramTest {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    ) {
        let mut account = Account::new(amount, T::get_packed_len(), owner);
        data.pack_into_slice(&mut account.data);
        self.add_account(pubkey, account);
    }
}
