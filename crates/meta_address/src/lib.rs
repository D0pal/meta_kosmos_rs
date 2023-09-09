pub mod enums;

use ethers::core::types::Address;
use meta_common::{
    enums::{BotType, ContractType, DexExchange, Network, RpcProvider},
    traits::ContractCode,
};
use meta_macro::impl_contract_code;
use once_cell::sync::Lazy;
use std::{collections::HashMap, str::FromStr};

include!(concat!(env!("OUT_DIR"), "/token_enum.rs"));

#[derive(Clone, Debug, Deserialize)]
pub struct Contract {
    addresses: HashMap<Network, Address>,
}

impl Contract {
    /// Returns the address of the contract on the specified network. If the contract's address is
    /// not found in the addressbook, the getter returns None.
    pub fn address(&self, network: Network) -> Option<Address> {
        self.addresses.get(&network).cloned()
    }
}

#[derive(Clone, Debug, Deserialize)]
struct PersistedTokenInfo {
    pub decimals: u8,
    pub address: Address,
    pub native: bool,
    pub unwrap_to: Option<Token>,
    pub byte_code: Option<String>,
    pub code_hash: Option<String>,
}

#[derive(Clone, Debug)]
#[impl_contract_code()]
pub struct TokenInfo {
    pub network: Network,
    pub token: Token,
    pub decimals: u8,
    pub address: Address,
    pub native: bool,
    pub unwrap_to: Option<Token>,
    pub byte_code: Option<String>,
    pub code_hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[impl_contract_code()]
pub struct ContractInfo {
    pub address: Address,
    pub created_blk_num: u64,
    pub byte_code: Option<String>,
    pub code_hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RpcInfo {
    pub http_urls: HashMap<RpcProvider, String>,
    pub ws_urls: HashMap<RpcProvider, String>,
    pub chain_id: u16,
    pub explorer: String,
}

const TOKEN_ADDRESS_JSON: &str = include_str!("../static/token_address.json");
const DEX_ADDRESS_JSON: &str = include_str!("../static/dex_address.json");
const BOT_ADDRESS_JSON: &str = include_str!("../static/bot_address.json");
const RPC_JSON: &str = include_str!("../static/rpc.json");

/// <token_name, <network, token_info>>
static NAMED_TOKEN_INFO_BOOK: Lazy<HashMap<String, HashMap<Network, PersistedTokenInfo>>> =
    Lazy::new(|| serde_json::from_str(TOKEN_ADDRESS_JSON).unwrap());

/// <network, <token_address, token_info>>
static ADDRESSED_TOKEN_INFO_BOOK: Lazy<
    HashMap<Network, HashMap<Address, (String, PersistedTokenInfo)>>,
> = Lazy::new(|| {
    let mut book: HashMap<Network, HashMap<ethers::types::H160, (String, PersistedTokenInfo)>, _> =
        HashMap::default();
    NAMED_TOKEN_INFO_BOOK.iter().for_each(|(name, val)| {
        val.iter().for_each(|(network, info)| {
            if !book.contains_key(network) {
                book.insert(network.to_owned(), HashMap::new());
            }
            let info_map = book.get_mut(network).unwrap();
            info_map.entry(info.address).or_insert((name.to_string(), info.to_owned()));
        })
    });
    book
});

static DEX_ADDRESS_BOOK: Lazy<
    HashMap<DexExchange, HashMap<Network, HashMap<ContractType, ContractInfo>>>,
> = Lazy::new(|| serde_json::from_str(DEX_ADDRESS_JSON).unwrap());
static BOT_ADDRESS_BOOK: Lazy<HashMap<BotType, HashMap<Network, ContractInfo>>> =
    Lazy::new(|| serde_json::from_str(BOT_ADDRESS_JSON).unwrap());

static RPC_INFO_BOOK: Lazy<HashMap<Network, RpcInfo>> =
    Lazy::new(|| serde_json::from_str(RPC_JSON).unwrap());

pub fn get_token_info<T: Into<String>>(token_name: T, network: Network) -> Option<TokenInfo> {
    let token_name: String = token_name.into();
    NAMED_TOKEN_INFO_BOOK.get(&token_name).map_or(None, |x| {
        x.get(&network).cloned().map_or(None, |e| {
            Some(TokenInfo {
                network: network,
                token: Token::from_str(&token_name).unwrap(),
                decimals: e.decimals,
                address: e.address,
                native: e.native,
                unwrap_to: e.unwrap_to,
                byte_code: e.byte_code,
                code_hash: e.code_hash,
            })
        })
    })
}

pub fn get_addressed_token_info(network: Network, address: Address) -> Option<TokenInfo> {
    ADDRESSED_TOKEN_INFO_BOOK.get(&network).map_or(None, |x| {
        x.get(&address).cloned().map_or(None, |e| {
            Some(TokenInfo {
                network: network,
                token: Token::from_str(&e.0).unwrap(),
                decimals: e.1.decimals,
                address: e.1.address,
                native: e.1.native,
                unwrap_to: e.1.unwrap_to,
                byte_code: e.1.byte_code,
                code_hash: e.1.code_hash,
            })
        })
    })
}

pub fn get_bot_contract_info(name: BotType, network: Network) -> Option<ContractInfo> {
    BOT_ADDRESS_BOOK.get(&name.into()).map_or(None, |v| v.get(&network).cloned())
}

pub fn get_dex_address(
    dex_name: DexExchange,
    chain_name: Network,
    contract_type: ContractType,
) -> Option<ContractInfo> {
    DEX_ADDRESS_BOOK
        .get(&dex_name.into())
        .map_or(None, |v| v.get(&chain_name).map_or(None, |v| v.get(&contract_type).cloned()))
}

pub fn get_rpc_info(network: Network) -> Option<RpcInfo> {
    RPC_INFO_BOOK.get(&network.into()).cloned()
}

#[cfg(test)]
mod tests {
    use meta_common::enums::{ContractType, DexExchange, Network};
    use meta_util::ether::address_from_str;

    use super::*;

    #[test]
    fn test_token_enum() {
        assert_eq!("BTC".to_string(), Token::BTC.to_string());
    }

    #[test]
    fn test_token_addr() {
        let eth_weth = get_token_info(Token::WETH, Network::ETH).unwrap();
        assert_eq!(
            eth_weth.address,
            address_from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
        );
        // assert_eq!(
        //     eth_weth.get_byte_code_and_hash().1,
        //     [
        //         208, 160, 107, 18, 172, 71, 134, 59, 92, 123, 228, 24, 92, 45, 234, 173, 28, 97,
        //         85, 112, 51, 245, 108, 125, 78, 167, 68, 41, 203, 178, 94, 35
        //     ]
        // );

        assert_eq!(
            get_token_info("WETH", Network::ETH).unwrap().address,
            address_from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
        );

        assert_eq!(
            get_token_info(Token::WETH, Network::ARBI).unwrap().address,
            address_from_str("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1")
        );
        assert_eq!(
            get_token_info(Token::USDC, Network::ARBI).unwrap().address,
            address_from_str("0xaf88d065e77c8cc2239327c5edb3a432268e5831")
        );
    }

    #[test]
    fn test_get_addressed_token_info() {
        let eth_weth = get_addressed_token_info(
            Network::ETH,
            address_from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
        )
        .unwrap();
        assert_eq!(
            eth_weth.address,
            address_from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
        );
        // assert_eq!(
        //     eth_weth.get_byte_code_and_hash().1,
        //     [
        //         208, 160, 107, 18, 172, 71, 134, 59, 92, 123, 228, 24, 92, 45, 234, 173, 28, 97,
        //         85, 112, 51, 245, 108, 125, 78, 167, 68, 41, 203, 178, 94, 35
        //     ]
        // );
    }
    #[test]
    fn test_dex_addr() {
        let quoter =
            get_dex_address(DexExchange::UniswapV3, Network::ETH, ContractType::UniV3QuoterV2);
        assert!(quoter.is_some());
        let quoter = quoter.unwrap();
        assert_eq!(quoter.address, address_from_str("0x61fFE014bA17989E743c5F6cB21bF9697530B21e"));
        // assert_eq!(
        //     quoter.get_byte_code_and_hash().1,
        //     [
        //         6, 20, 143, 71, 208, 244, 26, 104, 211, 188, 151, 0, 48, 167, 21, 14, 93, 96, 140,
        //         251, 194, 141, 55, 36, 64, 162, 228, 28, 229, 67, 217, 43
        //     ]
        // );
    }

    #[test]
    fn test_get_rpc_info() {
        assert!(get_rpc_info(Network::ETH).is_some());
        let rpc_info = get_rpc_info(Network::ETH).unwrap();
        println!("rpc {:?}", rpc_info);
        assert_eq!(rpc_info.chain_id, 1);
        assert_eq!(
            rpc_info.ws_urls.get(&RpcProvider::Quick),
            Some(
                &"wss://lively-bold-sunset.quiknode.pro/a44820da0711822c6e00da793df8695e60e027a2/"
                    .to_string()
            )
        );
    }
}
