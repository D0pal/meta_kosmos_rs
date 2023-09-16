use meta_address::enums::Asset;
use meta_common::enums::CexExchange;

pub mod binance;
pub mod bitfinex;
pub mod cefi_service;
pub mod util;

pub fn cex_currency_to_asset(cex: CexExchange, currency: &str) -> Asset {
    match cex {
        _ => currency.parse::<Asset>().unwrap(),
    }
}

pub fn get_cex_pair(cex: CexExchange, base: Asset, quote: Asset) -> String {
    match cex {
        CexExchange::BITFINEX => format!("t{:?}{:?}", base, quote),
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use meta_address::enums::Asset;
    #[test]
    fn test_cex_currency_to_asset() {
        assert_eq!(cex_currency_to_asset(CexExchange::BITFINEX, "ARB"), Asset::ARB);
        assert_eq!(cex_currency_to_asset(CexExchange::BITFINEX, "USD"), Asset::USD);
    }
}
