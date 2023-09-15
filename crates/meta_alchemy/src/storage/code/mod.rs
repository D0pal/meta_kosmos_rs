use ethers::prelude::*;
use meta_util::ether::address_from_str;

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref ETH_V3_POOL_CODE_MAP: HashMap<Address, (Bytes, [u8; 32])> = {
        let mut map = HashMap::new();
        let (address, tuple) = get_v3_weth_usdc_500_code();
        map.insert(address, tuple);
        map
    };
    pub static ref ARB_V3_POOL_CODE_MAP: HashMap<Address, (Bytes, [u8; 32])> = {
        let mut map = HashMap::new();
        let (address, tuple) = get_arbitrum_uni_v3_weth_usdc_500_code_and_hash();
        map.insert(address, tuple);
        map
    };
}

pub fn get_circle_proxy_code_and_hash() -> (Address, (Bytes, [u8; 32])) {
    (
        address_from_str("0xa2327a938febf5fec13bacfb16ae10ecbc4cbdcf"),
        (
            "".parse().unwrap(),
            [
                16, 214, 143, 155, 178, 186, 159, 94, 145, 99, 202, 220, 74, 52, 70, 190, 12, 107,
                42, 184, 177, 182, 80, 121, 231, 157, 205, 148, 166, 220, 50, 156,
            ],
        ),
    )
}

pub fn get_v3_weth_usdc_500_code() -> (Address, (Bytes, [u8; 32])) {
    (
        address_from_str("0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640"),
        (
            "".parse().unwrap(),
            [
                169, 129, 182, 108, 116, 122, 61, 159, 162, 157, 126, 32, 13, 95, 170, 162, 130,
                105, 96, 82, 61, 14, 90, 13, 248, 20, 142, 136, 104, 196, 128, 180,
            ],
        ),
    )
}

pub fn get_arbitrum_extended_weth_code_and_hash() -> (Address, (Bytes, [u8; 32])) {
    (
        address_from_str("0x8b194bEae1d3e0788A1a35173978001ACDFba668"),
        (
            "".parse().unwrap(),
            [
                13, 28, 32, 249, 237, 85, 30, 254, 143, 64, 43, 201, 170, 26, 155, 80, 88, 249, 37,
                236, 97, 82, 132, 197, 180, 167, 164, 98, 60, 59, 45, 205,
            ],
        ),
    )
}

pub fn get_arbitrum_fiat_token_code_and_hash() -> (Address, (Bytes, [u8; 32])) {
    (
        address_from_str("0x0f4fb9474303d10905AB86aA8d5A65FE44b6E04A"),
        (
            "".parse().unwrap(),
            [
                181, 115, 88, 162, 82, 99, 61, 115, 157, 36, 26, 152, 234, 10, 224, 255, 12, 64,
                164, 64, 79, 0, 62, 33, 220, 212, 105, 233, 245, 34, 108, 72,
            ],
        ),
    )
}

pub fn get_arbitrum_uni_v3_weth_usdc_500_code_and_hash() -> (Address, (Bytes, [u8; 32])) {
    (
        address_from_str("0xc6962004f452be9203591991d15f6b388e09e8d0"),
        (
            "".parse().unwrap(),
            [
                185, 216, 153, 218, 193, 54, 193, 245, 23, 158, 25, 135, 50, 138, 187, 76, 159, 26,
                192, 171, 240, 39, 202, 10, 216, 23, 80, 48, 9, 150, 175, 108,
            ],
        ),
    )
}
