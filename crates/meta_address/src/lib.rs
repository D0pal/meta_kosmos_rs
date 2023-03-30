use meta_common::enums::{Network, ContractType, Bot};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use ethers::core::types::{Address};
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
pub struct DexAddress {
    addresses: HashMap<ContractType, Address>,
}

impl DexAddress {
    /// Returns the address of the contract on the specified chain. If the contract's address is
    /// not found in the addressbook, the getter returns None.
    pub fn address(&self, contract_type: ContractType) -> Option<Address> {
        self.addresses.get(&contract_type).cloned()
    }
}

const TOKEN_ADDRESS_JSON: &str = include_str!("../static/token_address.json");
const DEX_ADDRESS_JSON: &str = include_str!("../static/dex_address.json");
const BOT_ADDRESS_JSON: &str = include_str!("../static/bot_address.json");

static TOKEN_ADDRESS_BOOK: Lazy<HashMap<String, Contract>> =
    Lazy::new(|| serde_json::from_str(TOKEN_ADDRESS_JSON).unwrap());

static DEX_ADDRESS_BOOK: Lazy<HashMap<String, HashMap<Network, DexAddress>>> =
    Lazy::new(|| serde_json::from_str(DEX_ADDRESS_JSON).unwrap());

static BOT_ADDRESS_BOK: Lazy<HashMap<Bot, Contract>> =
Lazy::new(|| serde_json::from_str(BOT_ADDRESS_JSON).unwrap());

/// Fetch the addressbook for a contract by its name. If the contract name is not a part of
/// [ethers-addressbook](https://github.com/gakonst/ethers-rs/tree/master/ethers-addressbook) we return None.
pub fn get_token_address<S: Into<String>>(name: S) -> Option<Contract> {
    TOKEN_ADDRESS_BOOK.get(&name.into()).cloned()
}

pub fn get_dex_address<S: Into<String>>(dex_name: S, chain_name: Network) -> Option<DexAddress> {
    DEX_ADDRESS_BOOK
        .get(&dex_name.into())
        .map_or(None, |v| v.get(&chain_name).cloned())
}

#[cfg(test)]
mod tests {
    use meta_common::enums::{Token, Network,Dex,ContractType};

    use super::*;

    #[test]
    fn test_token_addr() {
        assert!(get_token_address(Token::WBNB).is_some());
        assert!(get_token_address(Token::BUSD).is_some());
        assert!(get_token_address("rand").is_none());

        assert!(get_token_address(Token::WBNB)
            .unwrap()
            .address(Network::BSC)
            .is_some());
        assert!(get_token_address(Token::WBNB)
            .unwrap()
            .address(Network::BSC_TEST)
            .is_some());
    }

    #[test]
    fn test_dex_addr() {
        assert!(get_dex_address(Dex::PANCAKE, Network::BSC).is_some());
        assert!(get_dex_address(Dex::PANCAKE, Network::BSC).unwrap().address(ContractType::UNI_V2_FACTORY).is_some());
    }
}
