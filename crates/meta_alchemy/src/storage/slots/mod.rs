pub mod erc20;
pub mod uni_v2;
pub mod uni_v3;

use ethers::prelude::*;

pub fn storage_slot_by_index(index: u64) -> H256 {
    H256::from_low_u64_be(index)
}

pub fn storage_mapping_address_of_mapping_address(
    index: u32,
    first: Address,
    second: Address,
) -> H256 {
    let slot = TxHash::from(ethers::utils::keccak256(abi::encode(&[
        abi::Token::Address(first),
        abi::Token::Uint(U256::from(index)),
    ])));
    TxHash::from(ethers::utils::keccak256(abi::encode(&[
        abi::Token::Address(second),
        abi::Token::FixedBytes(slot.0.into()),
    ])))
}

