use ethers::prelude::*;

/// Small helper function to convert [U256] into [H256].
pub fn u256_to_h256_be(u: U256) -> H256 {
    let mut h = H256::default();
    u.to_big_endian(h.as_mut());
    h
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
