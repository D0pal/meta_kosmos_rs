#![allow(unused_assignments)]
use ethers::prelude::*;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref COMMON_SLOTS: Vec<H256> = {
        vec![
            uni_v3_slot_0_storage_slot(),
            uni_v3_fee_growth_global0_x128_slot(),
            uni_v3_fee_growth_global1_x128_slot(),
            uni_v3_liquidity_slot(),
        ]
    };

    /// for weth_usdc
    pub static ref TICK_SLOTS: HashMap<i32, [H256; 4]> = {
        let mut map = HashMap::new();
        (200000..201500).for_each(|tick| {
            map.insert(tick, uni_v3_tick_storage_slots(tick));
        });
        map
    };

     /// for weth_usdc
     pub static ref TICK_MAP_SLOTS: HashMap<(i32, i32), H256> = {
        let mut map = HashMap::new();
        (200000..201500).for_each(|tick| {
            map.insert((tick, 10), uni_v3_tick_map_slots(tick, 10));
            map.insert((tick, 60), uni_v3_tick_map_slots(tick, 60));
            map.insert((tick, 200), uni_v3_tick_map_slots(tick, 200));
        });
        map
    };

    pub static ref OBSERVATIONS_SLOTS: Vec<H256> = {
        (0..65535).map(uni_v3_observations_slot).collect()
    };


}

// pub fn get_v3_token_storage(token_address: Address) -> Vec<H256> {

// }

/// pool slot0,
/// ```solidity
/// struct Slot0 {
//     the current price
//     uint160 sqrtPriceX96;
//     // the current tick
//     int24 tick;
//     // the most-recently updated index of the observations array
//     uint16 observationIndex;
//     // the current maximum number of observations that are being stored
//     uint16 observationCardinality;
//     // the next maximum number of observations to store, triggered in observations.write
//     uint16 observationCardinalityNext;
//     // the current protocol fee as a percentage of the swap fee taken on withdrawal
//     // represented as an integer denominator (1/x)%
//     uint8 feeProtocol;
//     // whether the pool is locked
//     bool unlocked;
//  }
/// ```
/// struct field layout as  `0x00 01 00 02d2 02d2 0273 030f53 000000000000585047e34fa66b3eddd49aedae0e`
pub fn uni_v3_slot_0_storage_slot() -> H256 {
    H256::from_low_u64_be(0)
}

pub fn uni_v3_fee_growth_global0_x128_slot() -> H256 {
    H256::from_low_u64_be(1)
}

pub fn uni_v3_fee_growth_global1_x128_slot() -> H256 {
    H256::from_low_u64_be(2)
}

pub fn uni_v3_protocol_fees_slot() -> H256 {
    H256::from_low_u64_be(3)
}

pub fn uni_v3_liquidity_slot() -> H256 {
    H256::from_low_u64_be(4)
}

/// tick.info occupies 4 slots
pub fn uni_v3_tick_storage_slots(tick: i32) -> [H256; 4] {
    let mut slot_first = H256::default();
    if tick >= 0 {
        slot_first = H256::from(ethers::utils::keccak256(abi::encode(&[
            abi::Token::Int(U256::from(tick)),
            abi::Token::Uint(U256::from(5)), // 5nd storage, mapping(int24 => Tick.Info) public override ticks;
        ])));
    } else {
        let i_num =
            I256::checked_from_sign_and_abs(Sign::Negative, U256::from(tick.abs())).unwrap();
        let mut bytes: [u8; 32] = [0; 32];
        i_num.to_big_endian(&mut bytes);

        slot_first = H256::from(ethers::utils::keccak256(abi::encode(&[
            abi::Token::Int(U256::from(bytes)),
            abi::Token::Uint(U256::from(5)),
        ])));
    }

    let mut ret: [H256; 4] = [H256::default(); 4];
    for i in 0..4 {
        let u = U256::from(slot_first.0).checked_add(U256::from(i)).unwrap();
        let mut bytes: [u8; 32] = [0; 32];
        u.to_big_endian(&mut bytes);
        ret[i] = H256::from(bytes);
    }
    ret
}

pub fn uni_v3_tick_map_slots(tick: i32, tick_spacing: i32) -> H256 {
    let mut compressed = tick / tick_spacing;
    if tick < 0 && tick % tick_spacing != 0 {
        compressed -= 1; // round towards negative infinity
    }
    let (word_pos, _bit_pos) = position(compressed);
    // let (word_pos, bit_pos) = position(compressed+1);

    
    H256::from(ethers::utils::keccak256(abi::encode(&[
        abi::Token::Int(U256::from(word_pos)),
        abi::Token::Uint(U256::from(6)), // 6nd storage, mapping(int16 => uint256) public override tickBitmap;
    ])))
}

/// Oracle.Observation[65535] public override observations;
pub fn uni_v3_observations_slot(index: u64) -> H256 {
    H256::from_low_u64_be(index + 8)
}
fn position(tick: i32) -> (i16, u8) {
    let word_pos = (tick >> 8) as i16;
    let bit_pos = (tick % 256) as u8;

    (word_pos, bit_pos)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_uni_v3_tick_storage_slot() {
        let ret = uni_v3_tick_storage_slots(200546);
        assert_eq!(
            ret[0],
            H256::from_str("0x21d56931cd9f2684127ed84a91fae3decbced20bf3471d5abd2e0d1ac6098fe3")
                .unwrap()
        );
        assert_eq!(
            ret[1],
            H256::from_str("0x21d56931cd9f2684127ed84a91fae3decbced20bf3471d5abd2e0d1ac6098fe4")
                .unwrap()
        );
        assert_eq!(
            ret[2],
            H256::from_str("0x21d56931cd9f2684127ed84a91fae3decbced20bf3471d5abd2e0d1ac6098fe5")
                .unwrap()
        );
        assert_eq!(
            ret[3],
            H256::from_str("0x21d56931cd9f2684127ed84a91fae3decbced20bf3471d5abd2e0d1ac6098fe6")
                .unwrap()
        );

        let ret = uni_v3_tick_storage_slots(-200546);
        assert_eq!(
            ret[0],
            H256::from_str("0x0a492668105c971cb9e405f7bb92bcaf383dcc66137f8eb11ff7633d749ccd40")
                .unwrap()
        );
        assert_eq!(
            ret[1],
            H256::from_str("0x0a492668105c971cb9e405f7bb92bcaf383dcc66137f8eb11ff7633d749ccd41")
                .unwrap()
        );
        assert_eq!(
            ret[2],
            H256::from_str("0x0a492668105c971cb9e405f7bb92bcaf383dcc66137f8eb11ff7633d749ccd42")
                .unwrap()
        );
        assert_eq!(
            ret[3],
            H256::from_str("0x0a492668105c971cb9e405f7bb92bcaf383dcc66137f8eb11ff7633d749ccd43")
                .unwrap()
        );
    }

    #[test]
    fn test_uni_v3_tick_map_slots() {
        let slot = uni_v3_tick_map_slots(200753, 10);
        println!("slot: {:?}", slot);
    }
}
