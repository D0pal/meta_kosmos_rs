use std::str::FromStr;

use super::storage_mapping_address_of_mapping_address;
use ethers::prelude::*;
use lazy_static::lazy_static;

lazy_static! {

    pub static ref ETH_ERC20_USDC_PREFETCH_SLOTS: Vec<H256> = {
        vec![
            H256::from_str("0x7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3").unwrap(), // IMPLEMENTATION_SLOT
            H256::from_str("0x10d6a54a4754c8869d6886b5f5d7fbfa5b4522237ea5c60d11bc4e7a1ff9390b").unwrap(), // ADMIN_SLOT
            H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(), // OWNER_SLOT
            H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), // PAWSER_SLOT
        ]
    };
}

pub fn erc20_total_supply_slot() -> H256 {
    H256::from_low_u64_be(0)
}

pub fn erc20_balance_of_storage_slot(holder: Address) -> H256 {
    TxHash::from(ethers::utils::keccak256(abi::encode(&[
        abi::Token::Address(holder),
        abi::Token::Uint(U256::from(3)),
    ])))
}

/// usual erc20 token index is 4
/// usdc is 10
pub fn erc20_allowance_storage_slot(index: u32, owner: Address, spender: Address) -> H256 {
    storage_mapping_address_of_mapping_address(index, owner, spender)
}

#[cfg(test)]
mod tests {
    use meta_util::ether::address_from_str;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_erc20_balance_of_storage_slot() {
        let v3_weth_usdc_500_address =
            address_from_str("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
        let slot = erc20_balance_of_storage_slot(v3_weth_usdc_500_address);
        assert_eq!(
            slot,
            H256::from_str("0x390f6178407c9b8e95802b8659e6df8e34c1e3d4f8d6a49e6132bbcdd937b63a")
                .unwrap()
        );
    }

    #[test]
    fn test_erc20_allowance_slot() {
        let owner = address_from_str("0x9a6dcf5e566fa65c67a5f82aa98d03207e065726");
        let spender: H160 = address_from_str("0xE592427A0AEce92De3Edee1F18E0157C05861564");
        let slot = erc20_allowance_storage_slot(10, owner, spender);
        println!("slot: {:?}", slot);
        // assert_eq!(slot, H256::from_str("0x2f8888605407b1ca4b015f90553c45ccb64080c15a980253b89e2ce1724cd034").unwrap());
    }
}

// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0xe62d414f3a567cbfb91d56365331f3a3f45c3551729a8edcce7cad198e1e74c7, val: 0x0000000000000000000000000000000000000000000000000000000000000000
// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0x1f21a62c4538bacf2aabeca410f0fe63151869f172e03c0e00357ba26a341eff, val: 0x00000000000000000000000000000000000000000000000000007289738edbdb
// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0x3c53a91714fd8d27deef117c3827ce1fa74e6ad389952cc14a012bab8632e4ed, val: 0x0000000000000000000000000000000000000000000000000000000000000000

// fetch storage address: 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2, slot: 0x390f6178407c9b8e95802b8659e6df8e34c1e3d4f8d6a49e6132bbcdd937b63a, val: 0x000000000000000000000000000000000000000000000ef2dfa7d53dbf05ffa0
