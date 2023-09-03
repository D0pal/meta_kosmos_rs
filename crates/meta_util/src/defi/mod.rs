use ethers::prelude::*;

use lazy_static::lazy_static;

lazy_static! {
    // @dev The maximum value that can be returned from #getSqrtRatioAtTick. Equivalent to getSqrtRatioAtTick(MAX_TICK)
    // uint160 internal constant MAX_SQRT_RATIO = 1461446703485210103287273052203988822378723970342;
    pub static ref V3_POOL_MAX_SQRT_RATIO: U256 =
        U256::from_str_radix("1461446703485210103287273052203988822378723970341", 10).unwrap();
}

pub fn get_token0_and_token1(token_a: Address, token_b: Address) -> (Address, Address) {
    if token_a.lt(&token_b) {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    }
}

pub fn get_swap_price_limit(token_a: Address, token_b: Address, token_in: Address) -> U256 {
    let (token_0, _token_1) = get_token0_and_token1(token_a, token_b);
    if token_in.eq(&token_0) {
        U256::zero()
    } else {
        *V3_POOL_MAX_SQRT_RATIO
    }
}

pub fn get_tick_from_slot_value(hash: H256) -> i32 {
    let bytes = hash.0;
    let u8_array = &bytes[9..12];
    let mut result: i32 = 0;

    for &byte in u8_array {
        result = (result << 8) | (byte as i32);
    }
    if result & (1 << (u8_array.len() * 8 - 1)) != 0 {
        // If the sign bit is set, it's a negative value
        result -= 1 << (u8_array.len() * 8);
    }

    result
}

#[cfg(test)]
mod test {

    use std::str::FromStr;

    use crate::ether::address_from_str;

    use super::*;
    #[test]
    fn test_get_token0_and_token1() {
        let token_a = address_from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
        let token_b = address_from_str("0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d");
        let token_c = address_from_str("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1");
        assert_eq!(get_token0_and_token1(token_a, token_b), (token_b, token_a));
        assert_eq!(get_token0_and_token1(token_a, token_c), (token_c, token_a));
        assert_eq!(get_token0_and_token1(token_b, token_c), (token_c, token_b));
    }

    #[test]
    fn test_get_tick_from_slot_value() {
        let hash =
            H256::from_str("0x00010002d202d2017703156e0000000000005f7da3d41ecb8dbab7e862243c3e")
                .unwrap();
        let tick = get_tick_from_slot_value(hash);
        assert_eq!(tick, 202094);

        let hash =
            H256::from_str("0x00010000960096003c00d957000000000000001025fe8d8c5f1b397588f88c82")
                .unwrap();
        let tick = get_tick_from_slot_value(hash);
        assert_eq!(tick, 55639);

        let hash =
            H256::from_str("0x00010000c800c8005ffcea5600000000000000000002ac47071bd364b755830e")
                .unwrap();
        let tick = get_tick_from_slot_value(hash);
        assert_eq!(tick, -202154);
    }
}
