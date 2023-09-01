use ethers::prelude::*;
use lazy_static::lazy_static;
use std::{ops::Add, str::FromStr};

pub const APEX_INIT_BLOCK_NUM: u64 = 9624480;
pub const ZERO_ADDRESS: &'static str = "000000000000000000000000000000000000000000";

pub const ARBITRUM_SENDER: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x0a, 0x4b, 0x05,
]);

pub fn address_from_str(addr: &str) -> Address {
    Address::from_str(addr).to_owned().expect("cannot convert string to address")
}

lazy_static! {
    /// This is an example for using doc comment attributes
    // pub static ref ZERO_ADDRESS: Address = address_from_str("000000000000000000000000000000000000000000");
    pub static ref ETHER: U128 = U128::from(u128::pow(10, 18));
    pub static ref ERC20_TRANSFER_EVENT_SIG: H256 = H256::from_str("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef").unwrap();

}
