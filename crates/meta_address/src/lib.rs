use ethers::{abi::Hash, core::types::Address};
use meta_common::enums::{BotType, ContractType, DexExchange, Network, Token};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
/// Wrapper around a hash map that maps a [network](https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/network.rs) to the contract's deployed address on that network.
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
pub struct ContractInfo {
    pub address: Address,
    pub created_blk_num: u64,
}

// impl DexAddress {
//     /// Returns the address of the contract on the specified chain. If the contract's address is
//     /// not found in the addressbook, the getter returns None.
//     pub fn address(&self, contract_type: ContractType) -> Option<Address> {
//         self.addresses.get(&contract_type).cloned()
//     }
// }

#[derive(Clone, Debug, Deserialize)]
pub struct RpcInfo {
    pub httpUrls: Vec<String>,
    pub wsUrls: Vec<String>,
    pub chainId: u16,
    pub explorer: String,
}

const TOKEN_ADDRESS_JSON: &str = include_str!("../static/token_address.json");
const DEX_ADDRESS_JSON: &str = include_str!("../static/dex_address.json");
const BOT_ADDRESS_JSON: &str = include_str!("../static/bot_address.json");
const RPC_JSON: &str = include_str!("../static/rpc.json");

static TOKEN_ADDRESS_BOOK: Lazy<HashMap<Token, Contract>> =
    Lazy::new(|| serde_json::from_str(TOKEN_ADDRESS_JSON).unwrap());

static DEX_ADDRESS_BOOK: Lazy<
    HashMap<DexExchange, HashMap<Network, HashMap<ContractType, ContractInfo>>>,
> = Lazy::new(|| serde_json::from_str(DEX_ADDRESS_JSON).unwrap());

static BOT_ADDRESS_BOOK: Lazy<HashMap<BotType, HashMap<Network, ContractInfo>>> =
    Lazy::new(|| serde_json::from_str(BOT_ADDRESS_JSON).unwrap());

static RPC_INFO_BOOK: Lazy<HashMap<Network, RpcInfo>> =
    Lazy::new(|| serde_json::from_str(RPC_JSON).unwrap());

/// Fetch the addressbook for a contract by its name. If the contract name is not a part of
/// [ethers-addressbook](https://github.com/gakonst/ethers-rs/tree/master/ethers-addressbook) we return None.
pub fn get_token_address(name: Token) -> Option<Contract> {
    TOKEN_ADDRESS_BOOK.get(&name.into()).cloned()
}

pub fn get_bot_address(name: BotType, network: Network) -> Option<ContractInfo> {
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
    use meta_common::{
        constants::address_from_str,
        enums::{ContractType, DexExchange, Network, Token},
    };

    use super::*;

    #[test]
    fn test_token_addr() {
        println!("{:?}", get_token_address(Token::WBNB));
        assert!(get_token_address(Token::WBNB).is_some());
        println!("{:?}", get_token_address(Token::WBNB).unwrap().address(Network::BSC).unwrap());
        assert_eq!(
            get_token_address(Token::WBNB).unwrap().address(Network::BSC).unwrap(),
            address_from_str("0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c")
        );
        // assert!(get_token_address(Token::BUSD).is_some());
        assert!(get_token_address(Token::EMPTY).is_none());

        assert!(get_token_address(Token::WBNB).unwrap().address(Network::BSC).is_some());
        assert!(get_token_address(Token::WBNB).unwrap().address(Network::BSC_TEST).is_some());
    }

    #[test]
    fn test_dex_addr() {
        assert!(get_dex_address(DexExchange::PANCAKE, Network::BSC, ContractType::UNI_V2_FACTORY)
            .is_some());
        assert_eq!(
            get_dex_address(DexExchange::PANCAKE, Network::BSC, ContractType::UNI_V2_FACTORY)
                .unwrap()
                .address,
            address_from_str("0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73")
        );
        assert_eq!(
            get_dex_address(DexExchange::PANCAKE, Network::BSC, ContractType::UNI_V2_FACTORY)
                .unwrap()
                .created_blk_num,
            6809737
        );
    }

    #[test]
    fn test_get_bot_address() {
        assert!(get_bot_address(BotType::ATOMIC_SWAP_ROUTER, Network::BSC).is_some());
        let bot_addrs = get_bot_address(BotType::ATOMIC_SWAP_ROUTER, Network::ZK_SYNC_ERA).unwrap();
        assert_eq!(bot_addrs.address, address_from_str("0xea57F2ca01dAb59139b1AFC483bd29cE8B727361"));
    }

    #[test]
    fn test_get_rpc_info() {
        assert!(get_rpc_info(Network::ZK_SYNC_ERA).is_some());
        let rpc_info = get_rpc_info(Network::ZK_SYNC_ERA).unwrap();
        assert_eq!(rpc_info.chainId, 324);
        assert_eq!(rpc_info.httpUrls[0], "https://zksync2-mainnet.zksync.io");
    }
}
