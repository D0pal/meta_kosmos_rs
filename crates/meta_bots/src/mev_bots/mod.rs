pub mod alert;
pub mod braindance;
pub mod bundle;
pub mod crypto;
pub mod oracle_runner;
pub mod relay;
pub mod sandwidth;
pub mod simulation;
pub mod types;
pub use types::*;
pub mod helpers;
pub mod testhelpder;

use meta_common::constants::address_from_str;

use ethers::{prelude::*, types::U256, utils::parse_ether};
use lazy_static::lazy_static;
use revm::primitives::Address as rAddress;
use std::str::FromStr;
lazy_static! {
    pub static ref LARK_WEBHOOK: String = "".to_string();
    pub static ref BRAINDANCE_STARTING_BALANCE: U256 = parse_ether(420).unwrap();
    pub static ref DEV_CALLER_ADDRESS: Address =
        address_from_str("0x5AbFEc25f74Cd88437631a7731906932776356f9");

    // Holds constant value representing braindance contract address
    pub static ref DEV_BRAINDANCE_ADDRESS: rAddress = rAddress::from_str("00000000000000000000000000000000F3370000").unwrap();
    // Holds constant value representing braindance caller
    pub static ref DEV_BRAINDANCE_CONTRAOLLER_ADDRESS:rAddress = rAddress::from_str("000000000000000000000000000000000420BABE").unwrap();
}
