use ethers::prelude::*;

use super::storage_mapping_address_of_mapping_address;

pub fn uniswap_v2_factory_get_pair_storage_slot(token0: Address, token1: Address) -> H256 {
    storage_mapping_address_of_mapping_address(2, token0, token1)
}

#[cfg(test)]
mod test_util {
    use super::*;
    use ethers::types::H256;
    use meta_util::ether::address_from_str;
    use std::str::FromStr;

    #[test]
    fn test_storage() {
        let weth = address_from_str("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
        let usdc = address_from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        let slot = uniswap_v2_factory_get_pair_storage_slot(weth, usdc);
        assert_eq!(
            slot,
            H256::from_str("0x10cbec91f600e7f895b03bc61241c3cc3bb96ded9a260421b1d174032138cc01")
                .unwrap()
        );
    }
}

