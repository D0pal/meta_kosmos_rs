use crate::bitfinex::{client::*, errors::*};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradingOrderBookLevel {
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

    pub fn trading_pair<S>(&self, symbol: S, precision: S) -> Result<Vec<TradingOrderBookLevel>>
    where
        S: Into<String>,
    {
        let endpoint: String = format!("book/t{}/{}", symbol.into(), precision.into());
        let data = self.client.get(endpoint, String::new())?;

        let book: Vec<TradingOrderBookLevel> = from_str(data.as_str())?;

        Ok(book)
    }
}
