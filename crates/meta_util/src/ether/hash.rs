use ethers::{core::types::U256, prelude::*};

use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{ops::Mul, str::FromStr};

/// Small helper function to convert [U256] into [H256].
pub fn u256_to_h256_be(u: U256) -> H256 {
    let mut h = H256::default();
    u.to_big_endian(h.as_mut());
    h
}

/// Small helper function to convert ether's [H256] into revm's [B256].
#[inline]
pub fn h256_to_b256(h: ethers::types::H256) -> revm::primitives::B256 {
    revm::primitives::B256(h.0)
}

/// Small helper function to convert ethers's [H160] into revm's [B160].
#[inline]
pub fn h160_to_b160(h: ethers::types::H160) -> revm::primitives::B160 {
    revm::primitives::B160(h.0)
}

/// Small helper function to convert [H256] into [U256].
pub fn h256_to_u256_be(storage: H256) -> U256 {
    U256::from_big_endian(storage.as_bytes())
}

/// Small helper function to convert ether's [U256] into revm's [U256].
#[inline]
pub fn u256_to_ru256(u: ethers::types::U256) -> revm::primitives::U256 {
    let mut buffer = [0u8; 32];
    u.to_little_endian(buffer.as_mut_slice());
    revm::primitives::U256::from_le_bytes(buffer)
}

#[cfg(test)]
mod test_hash {
    use std::str::FromStr;

    use super::u256_to_h256_be;
    use ethers::prelude::*;

    #[test]
    fn should_u256_to_h256_be() {
        let ret = u256_to_h256_be(
            U256::from_str_radix("0x3f349bBaFEc1551819B8be1EfEA2fC46cA749aA1", 16).unwrap(),
        );
        assert_eq!(
            ret,
            H256::from([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3F, 0x34,
                0x9b, 0xBa, 0xFE, 0xc1, 0x55, 0x18, 0x19, 0xB8, 0xbe, 0x1E, 0xfE, 0xA2, 0xfC, 0x46,
                0xcA, 0x74, 0x9a, 0xA1
            ])
        );
    }
}
