use ethers::types::Address;
use std::str::FromStr;

pub fn address_to_str(addr: &Address) -> String {
    let str: String = hex::encode(addr);
    str
}

pub fn address_from_str(addr: &str) -> Address {
    Address::from_str(addr).to_owned().expect("cannot convert string to address")
}
