use crate::bitfinex::client::*;
use crate::bitfinex::errors::*;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use rust_decimal::Decimal;

#[derive(Serialize, Deserialize, Debug)]
pub struct TradingPair {
    pub price: Decimal,
    pub count: i64,
    pub amount: Decimal,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct FundingCurrency {
    pub rate: Decimal,
    pub period: Decimal,
    pub count: i64,
    pub amount: Decimal,
}

#[derive(Clone)]
pub struct Book {
    client: Client,
}

// Trading: if AMOUNT > 0 then bid else ask; Funding: if AMOUNT < 0 then bid else ask;
#[derive(Serialize, Deserialize, Debug)]
pub struct RawBook {
    pub order_id: i64,
    pub price: Decimal,
    pub amount: Decimal,
}

impl Book {
    pub fn new() -> Self {
        Book { client: Client::new(None, None) }
    }

    pub fn funding_currency<S>(&self, symbol: S, precision: S) -> Result<Vec<FundingCurrency>>
    where
        S: Into<String>,
    {
        let endpoint: String = format!("book/f{}/{}", symbol.into(), precision.into());
        let data = self.client.get(endpoint, String::new())?;

        let book: Vec<FundingCurrency> = from_str(data.as_str())?;

        Ok(book)
    }

    pub fn trading_pair<S>(&self, symbol: S, precision: S) -> Result<Vec<TradingPair>>
    where
        S: Into<String>,
    {
        let endpoint: String = format!("book/t{}/{}", symbol.into(), precision.into());
        let data = self.client.get(endpoint, String::new())?;

        let book: Vec<TradingPair> = from_str(data.as_str())?;

        Ok(book)
    }
}

#[cfg(test)]
mod test_book {
    use super::{TradingPair, TradingPairTuple};
    use serde_json::from_str;
    use serde::{Serialize,Deserialize};



    #[test]
    fn test_trading_pair_ser_deser() {
        let update: TradingPairTuple = from_str("[30367,7,1.34783522]").unwrap();
        println!("{:?}", update);
        assert_eq!(update.0, 30367f64);
        assert_eq!(update.1, 7);
        assert_eq!(update.2, 1.34783522);
    }
}
