use rust_decimal::Decimal;


pub fn decimal_from_str(input: &str) -> Decimal {
    Decimal::from_str_radix(input, 10).unwrap()
}

#[cfg(test)]
mod test_decimal {
    use rust_decimal::{Decimal, prelude::FromPrimitive};
    use super::*;

    #[test]
    fn test_decimal() {
        let mut input = Decimal::from_str_radix("1234.567", 10).unwrap();
        input.set_sign_negative(true);
        let output = Decimal::from_str_radix("-1234.567", 10).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_decimal_from_str() {
        let ret = decimal_from_str("1633.48000000");
        assert_eq!(ret, Decimal::from_f64(1633.48000000).unwrap());
    }
}
