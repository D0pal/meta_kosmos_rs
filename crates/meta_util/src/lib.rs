pub mod enums;
pub mod ether;
pub mod time;
pub mod defi;
pub mod decimal;

use std::time::{SystemTime, UNIX_EPOCH};
use rust_decimal::{Decimal, prelude::{Signed, FromPrimitive}};


pub fn int_from_hex_str(input: &str) -> u64 {
    let parsed = input.replace("0x", "");
    u64::from_str_radix(&parsed, 16).unwrap()
}

pub fn get_price_delta_in_bp(bid: Decimal, ask: Decimal) -> Decimal {
    let change = bid.checked_sub(ask).unwrap().checked_div(ask).unwrap().checked_mul(Decimal::from_u32(10000).unwrap()).unwrap();
    change.abs()
}
// todo: toEther, fromEther

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_int_from_hex_str() {
        assert_eq!(int_from_hex_str("0xdf8475800"), 60_000_000_000);
        assert_eq!(int_from_hex_str("df8475800"), 60_000_000_000);
    }

    #[test]
    fn test_get_price_delta_in_bp() {
        assert_eq!(get_price_delta_in_bp(Decimal::from_f64(1010f64).unwrap(), Decimal::from_f64(1000f64).unwrap()), Decimal::from_f64(100f64).unwrap());
        assert_eq!(get_price_delta_in_bp(Decimal::from_f64(1000f64).unwrap(), Decimal::from_f64(1010f64).unwrap()), Decimal::from_str_radix("99.00990099009900990099009900", 10).unwrap());
    }
}
