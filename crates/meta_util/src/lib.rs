pub mod enums;
pub mod ether;

use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use ethers::types::Address;
use hex;


pub fn address_to_str(addr: &Address) -> String {
    let str: String = hex::encode(addr);
    str
}

pub fn address_from_str(addr: &str) -> Address {
    Address::from_str(addr)
        .to_owned()
        .expect("cannot convert string to address")
}

pub fn int_from_hex_str(input: &str) -> u64 {
    let parsed = input.replace("0x", "");
    u64::from_str_radix(&parsed, 16).unwrap()
}

pub fn get_current_ts_in_second() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}
// todo: toEther, fromEther

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_current_ts_in_second() {
        let ts = get_current_ts_in_second();
        assert_eq!(ts.to_string().len(), 10);
    }

    #[test]
    fn test_int_from_hex_str() {
        assert_eq!(int_from_hex_str("0xdf8475800"), 60_000_000_000);
        assert_eq!(int_from_hex_str("df8475800"), 60_000_000_000);
    }
}
