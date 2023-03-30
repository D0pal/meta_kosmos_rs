use ethers::prelude::*;
use lazy_static::lazy_static;
use std::{ops::Add, str::FromStr};

pub const APEX_INIT_BLOCK_NUM:u64 = 9624480;
pub const ZERO_ADDRESS: &'static str = "000000000000000000000000000000000000000000";

pub fn address_from_str(addr: &str) -> Address {
    Address::from_str(addr).to_owned().expect("cannot convert string to address")
}

lazy_static! {
    /// This is an example for using doc comment attributes
    // pub static ref ZERO_ADDRESS: Address = address_from_str("000000000000000000000000000000000000000000");
    pub static ref ETHER: U128 = U128::from(u128::pow(10, 18));
}