use ethers::abi::Address;
use meta_common::enums::{Token, Network};

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub decimals: u32,
    pub network: Network,
    pub address: Address,
}
