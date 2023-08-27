use crate::bitfinex::book::*;
use crate::bitfinex::orders::*;

#[derive(Clone)]
pub struct Bitfinex {
    pub book: Book,
    pub orders: Orders,
    // pub ticker: Ticker,
    // pub trades: Trades,
    // pub candles: Candles,
    // pub account: Account,
    // pub ledger: Ledger
}

impl Bitfinex {
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Bitfinex {
            book: Book::new(),
            // ticker: Ticker::new(),
            // trades: Trades::new(),
            // candles: Candles::new(),
            orders: Orders::new(api_key.clone(), secret_key.clone()),
            // account: Account::new(api_key.clone(), secret_key.clone()),
            // ledger: Ledger::new(api_key.clone(), secret_key.clone()),
        }
    }
}
