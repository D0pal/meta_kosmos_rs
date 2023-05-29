use meta_common::{enums::Dex};

pub fn dexs_from_str(dexs: String) -> Vec<Dex> {
    dexs.split(",").into_iter().map(|dex| dex.try_into().expect("unable to convert")).collect()
}

#[cfg(test)]
mod test_enums {
    use super::dexs_from_str;
    use meta_common::enums::Dex;

    #[test]
    fn should_dexs_from_str() {
        let ret = dexs_from_str("PANCAKE,UNISWAP_V2".to_string());
        assert_eq!(ret,vec![Dex::PANCAKE, Dex::UNISWAP_V2]);
    }
}