#![no_std]
#![feature(proc_macro_hygiene)]

extern crate pwasm_ethereum;
extern crate pwasm_std;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;
extern crate lazy_static;

use lazy_static::lazy_static;
use pwasm_abi_derive::eth_abi;
use pwasm_abi::eth::EndpointInterface;
use pwasm_std::types::{Address, H256, U256};

#[eth_abi(WalletEndpoint, WalletClient)]
pub trait WalletInterface {
    fn constructor(&mut self);

    #[constant]
    fn owner(&mut self) -> Address;
    #[constant]
    fn balance(&mut self) -> U256;
    fn addfund(&mut self) -> bool;
    fn withdraw(&mut self) -> bool;
}

lazy_static! {
    static ref OWNER_KEY: H256 =
        H256::from([2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static ref BALANCE_KEY: H256 =
        H256::from([3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
}

#[derive(Clone)]
pub struct WalletContract;

impl WalletInterface for WalletContract {
    fn constructor(&mut self) {
        let sender: [u8; 32] = H256::from(pwasm_ethereum::sender()).into();
        pwasm_ethereum::write(&OWNER_KEY, &sender)
    }

    fn owner(&mut self) -> Address {
        H256::from(pwasm_ethereum::read(&OWNER_KEY)).into()
    }

    fn balance(&mut self) -> U256 {
        pwasm_ethereum::read(&BALANCE_KEY).into()
    }

    fn addfund(&mut self) -> bool {
        let sender = pwasm_ethereum::sender();
        if sender != self.owner() {
            false
        }
        else {
            let new_balance: [u8; 32] = (self.balance() + pwasm_ethereum::value()).into();
            pwasm_ethereum::write(&BALANCE_KEY, &new_balance);
            true
        }
    }

    fn withdraw(&mut self) -> bool {
        let sender = pwasm_ethereum::sender();
        if sender != self.owner() {
            false
        }
        else {
            pwasm_ethereum::suicide(&sender)
        }
    }
}

#[no_mangle]
pub fn call() {
    let mut endpoint = WalletEndpoint::new(WalletContract{});
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = WalletEndpoint::new(WalletContract{});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}

#[cfg(test)]
mod tests {
    extern crate pwasm_test;
    extern crate std;

    use std::panic;
    use core::str::FromStr;
    use core::clone::Clone;
    use pwasm_abi::types::*;
    use pwasm_test::*;
    use super::*;

    #[test]
    fn test() {
        let mut contract = WalletContract{};
        let owner_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        ext_reset(|e| e.sender(owner_address.clone()));
        contract.constructor();

        assert_eq!(contract.owner(), owner_address);
        assert_eq!(contract.balance(), U256::from(0));

        ext_update(|e| e.sender(owner_address.clone()).value(U256::from(42)));

        assert_eq!(contract.addfund(), true);
        assert_eq!(contract.balance(), U256::from(42));

        let other_address = Address::from_str("0ecF65368318D64744a390235Abc5a1842F7b4b7").unwrap();
        ext_update(|e| e.sender(other_address.clone()));

        assert_eq!(contract.addfund(), false);
        assert_eq!(contract.withdraw(), false);

        ext_update(|e| e.sender(owner_address.clone()));

        assert!(panic::catch_unwind(|| {
            contract.clone().withdraw()
        }).is_err());
    }
}
