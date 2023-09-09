use meta_common::enums::DexExchange;

pub fn dexs_from_str(dexs: String) -> Vec<DexExchange> {
    dexs.split(',').map(|dex| dex.try_into().expect("unable to convert")).collect()
}

#[cfg(test)]
mod test_enums {
    use super::dexs_from_str;
    use meta_common::enums::DexExchange;

    #[test]
    fn should_dexs_from_str() {
        let ret = dexs_from_str("PANCAKE,UniswapV2".to_string());
        assert_eq!(ret, vec![DexExchange::PANCAKE, DexExchange::UniswapV2]);
    }
}
