use ethers::abi::Address;
use meta_common::enums::{Network};
use meta_address::Token;

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub decimals: u8,
    pub network: Network,
    pub address: Address,
}


