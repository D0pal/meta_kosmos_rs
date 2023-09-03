use ethers::abi::Address;
use meta_address::{Token, TokenInfo as TokenDetail};
use meta_common::enums::Network;

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub decimals: u8,
    pub network: Network,
    pub address: Address,
}

pub fn to_token_info(token: TokenDetail, network: Network, name: Token) -> TokenInfo {
    TokenInfo { token: name, decimals: token.decimals, network: network, address: token.address }
}
