use ethers::core::types::U256;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::ops::Mul;

pub fn decimal_to_wei(input: Decimal, decimals: u32) -> U256 {
    if input.is_sign_negative() {
        panic!("should not convert negative to u256");
    }
    let rounded = input.round_dp(6);

    let rounded_f64 = rounded.to_string().parse::<f64>().unwrap();
    let rounded_u128 = unsafe { rounded_f64.mul(1e6).to_int_unchecked::<u128>() };

    U256::from(rounded_u128.mul(u128::pow(10, decimals - 6)))
}

pub fn decimal_from_wei(input: U256, decimals: u32) -> Decimal {
    let reduced = input.checked_div(U256::from(u128::pow(10, decimals - 6))).unwrap();

    let reduced_f64 = reduced.to_string().parse::<f64>().unwrap();
    let out = Decimal::from_f64(reduced_f64).unwrap();
    out.checked_div(Decimal::from_f64(1e6f64).unwrap()).unwrap()
}

#[cfg(test)]
mod test_wei {
    use ethers::core::types::U256;
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::{decimal_from_wei, decimal_to_wei};

    #[test]
    fn test_decimal_to_wei() {
        let input = Decimal::from_f64(1912.12f64).unwrap();
        assert_eq!(
            decimal_to_wei(input, 18),
            U256::from_str_radix("1912120000000000000000", 10).unwrap()
        );

        let input = Decimal::from_f64(1912.1234567f64).unwrap();
        assert_eq!(
            decimal_to_wei(input, 18),
            U256::from_str_radix("1912123457000000000000", 10).unwrap()
        );

        let input = Decimal::from_f64(0.012f64).unwrap();
        assert_eq!(
            decimal_to_wei(input, 18),
            U256::from_str_radix("12000000000000000", 10).unwrap()
        );

        let input = Decimal::from_f64(0.1234567f64).unwrap();
        assert_eq!(
            decimal_to_wei(input, 18),
            U256::from_str_radix("123457000000000000", 10).unwrap()
        );
    }

    #[test]
    fn test_decimal_from_wei() {
        let out = decimal_from_wei(U256::from_str_radix("1912120000000000000000", 10).unwrap(), 18);
        assert_eq!(out.to_string(), "1912.12");

        let out = decimal_from_wei(U256::from_str_radix("1912123457000000000000", 10).unwrap(), 18);
        assert_eq!(out.to_string(), "1912.123457");

        let out = decimal_from_wei(U256::from_str_radix("12000000000000000", 10).unwrap(), 18);
        assert_eq!(out.to_string(), "0.012");

        let out = decimal_from_wei(U256::from_str_radix("123457000000000000", 10).unwrap(), 18);
        assert_eq!(out.to_string(), "0.123457");
    }
}
