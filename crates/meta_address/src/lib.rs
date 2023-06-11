use ethers::{abi::Hash, core::types::Address};
use ethers::prelude::*;
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
    pub byte_code: Option<Bytes>,
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

static TOKEN_ADDRESS_BOOK: Lazy<HashMap<Token, HashMap<Network, Address>>> =
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
pub fn get_token_address(name: Token, network: Network) -> Option<Address> {
    TOKEN_ADDRESS_BOOK.get(&name.into()).map_or(None, |x| x.get(&network).cloned())
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
    use meta_common::{
        constants::address_from_str,
        enums::{ContractType, DexExchange, Network, Token},
    };

    use super::*;

    #[test]
    fn test_token_addr() {
        assert_eq!(
            get_token_address(Token::WBNB, Network::BSC).unwrap(),
            address_from_str("0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c")
        );
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
        assert!(get_bot_contract_info(BotType::ATOMIC_SWAP_ROUTER, Network::BSC).is_some());
        let bot_addrs = get_bot_contract_info(BotType::ATOMIC_SWAP_ROUTER, Network::ZK_SYNC_ERA).unwrap();
        assert_eq!(
            bot_addrs.address,
            address_from_str("0xea57F2ca01dAb59139b1AFC483bd29cE8B727361")
        );

        let bot_addrs = get_bot_contract_info(BotType::SANDWIDTH_HUFF, Network::BSC).unwrap();
        assert_eq!(
            bot_addrs.address,
            address_from_str("0xae04a2c4ecf153e4537948df46fc85c8026cd4f3")
        );
        assert_eq!(
            bot_addrs.byte_code.unwrap(),
            "6109bf80600a3d393df33d3560001a565b610624565b61077e565b6106d1565b610843565b6104d2565b610587565b610425565b610390565b6102d4565b610230565b610908565b610928565b61094e5600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005b6099357fff000000000000000000000000000000000000000000000000000000000000006000527f1f98431c8ad98523631ae4a59f267346ea31f98400000000000000000000000046526015527fe34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54603552605560002073ffffffffffffffffffffffffffffffffffffffff163314156109ba573d3d60443d3d7effffffffffffffffffffffffffffffffffffffff00000000000000000000006084351660581c60843560f81c6101fa577fa9059cbb000000000000000000000000000000000000000000000000000000003d52336004526024356024525af1156109ba57005b7fa9059cbb000000000000000000000000000000000000000000000000000000003d52336004526004356024525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60f93d3d463560601c7f128acb080000000000000000000000000000000000000000000000000000000060005230600452620186a0340260445273fffd8963efd1fc6a506488495d951d5263988d2560645260a0608452603560a45273bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c60581b60c45260153560d9525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60f93d3d463560601c7f128acb0800000000000000000000000000000000000000000000000000000000600052306004526001602452620186a034026044526401000276ad60645260a0608452603560a4527f010000000000000000000000000000000000000000000000000000000000000073bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c60581b0160c45260153560d9525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60f93d3d463560601c7f128acb08000000000000000000000000000000000000000000000000000000006000523060045260293560d01c60445273fffd8963efd1fc6a506488495d951d5263988d2560645260a0608452603560a45260153560601c60581b60c452602f3560d9525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60f93d3d463560601c7f128acb080000000000000000000000000000000000000000000000000000000060005230600452600160245260293560d01c6044526401000276ad60645260a0608452603560a4527f010000000000000000000000000000000000000000000000000000000000000060153560601c60581b0160c452602f3560d9525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60f93d3d463560601c7f128acb080000000000000000000000000000000000000000000000000000000060005230600452600160245260293560b81c6509184e72a000026044526401000276ad60645260a0608452603560a4527f010000000000000000000000000000000000000000000000000000000000000060153560601c60581b0160c45260323560d9525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60f93d3d463560601c7f128acb08000000000000000000000000000000000000000000000000000000006000523060045260293560b81c6509184e72a0000260445273fffd8963efd1fc6a506488495d951d5263988d2560645260a0608452603560a45260153560601c60581b60c45260323560d9525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60a43d3d463560601c3d3d7fa9059cbb000000000000000000000000000000000000000000000000000000003d52826004526029358060081b9060001a5260443d3d60153560601c5af1507f022c0d9f00000000000000000000000000000000000000000000000000000000600052620186a0340260045260006024523060445260806064525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60a43d3d463560601c3d3d7fa9059cbb000000000000000000000000000000000000000000000000000000003d52826004526029358060081b9060001a5260443d3d60153560601c5af1507f022c0d9f000000000000000000000000000000000000000000000000000000006000526000600452620186a034026024523060445260806064525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60a43d3d463560601c3d3d7f23b872dd000000000000000000000000000000000000000000000000000000003d523060045282602452620186a0340260445260643d3d73bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c5af1507f022c0d9f00000000000000000000000000000000000000000000000000000000600052600060045260006024526015358060081b9060001a523060445260806064525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d60a43d3d463560601c3d3d7f23b872dd000000000000000000000000000000000000000000000000000000003d523060045282602452620186a0340260445260643d3d73bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c5af1507f022c0d9f0000000000000000000000000000000000000000000000000000000060005260006004526015358060081b9060001a5260006024523060445260806064525af1156109ba57005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba5733ff005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba573d3d3d3d47335af1005b73f44f3819d60739cbcd7b051ee20be34de0b1842a3314156109ba577fa9059cbb0000000000000000000000000000000000000000000000000000000059523360045246356024523d3d60443d3d73bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c5af1156109ba57005b600380fd".parse::<Bytes>().unwrap()
        );

    }

    #[test]
    fn test_get_rpc_info() {
        assert!(get_rpc_info(Network::ZK_SYNC_ERA).is_some());
        let rpc_info = get_rpc_info(Network::ZK_SYNC_ERA).unwrap();
        assert_eq!(rpc_info.chainId, 324);
        assert_eq!(rpc_info.httpUrls[0], "https://zksync2-mainnet.zksync.io");
    }
}
