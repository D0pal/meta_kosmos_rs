#[cfg(test)]
mod test_decimal {
    use rust_decimal::Decimal;

    #[test]
    fn test_decimal() {
        let mut input = Decimal::from_str_radix("1234.567", 10).unwrap();
        input.set_sign_negative(true);
        let output = Decimal::from_str_radix("-1234.567", 10).unwrap();
        assert_eq!(input, output);
    }
}