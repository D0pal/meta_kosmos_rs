use meta_common::enums::Network;
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

const TOKEN_ADDRESS_JSON: &str = include_str!("../static/token_address.json");
const DEX_ADDRESS_JSON: &str = include_str!("../static/dex_address.json");

static TOKEN_ADDRESS_BOOK: Lazy<HashMap<String, Contract>> =
    Lazy::new(|| serde_json::from_str(TOKEN_ADDRESS_JSON).unwrap());