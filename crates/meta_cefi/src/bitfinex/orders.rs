#![allow(non_camel_case_types)]

use crate::bitfinex::client::*;
use crate::bitfinex::errors::*;
use crate::bitfinex::model::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::collections::BTreeMap;
use strum::{AsRefStr, Display, EnumCount, EnumIter, EnumString, EnumVariantNames};

#[derive(Clone)]
pub struct Orders {
    client: Client,
}

struct OrderRequest {
    pub order_type: String,
    pub symbol: String,
    pub amount: f64,
    pub price: f64,
}

// struct MarketOrderRequest {
//     pub order_type: String,
//     pub symbol: String,
//     pub amount: f64,
// }

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsRefStr,         // AsRef<str>, fmt::Display and serde::Serialize
    EnumVariantNames, // Chain::VARIANTS
    EnumString,       // FromStr, TryFrom<&str>
    EnumIter,         // Chain::iter
    EnumCount,        // Chain::COUNT
    Deserialize,
    Serialize,
    Display,
)]
pub enum OrderType {
    LIMIT,
    #[strum(serialize = "EXCHANGE LIMIT")]
    EXCHANGE_LIMIT,
    MARKET,
    #[strum(serialize = "EXCHANGE MARKET")]
    EXCHANGE_MARKET,
    STOP,
    #[strum(serialize = "EXCHANGE STOP")]
    EXCHANGE_STOP,
    #[strum(serialize = "STOP LIMIT")]
    STOP_LIMIT,
    #[strum(serialize = "EXCHANGE STOP LIMIT")]
    EXCHANGE_STOP_LIMIT,
    #[strum(serialize = "TRAILING STOP")]
    TRAILING_STOP,
    #[strum(serialize = "EXCHANGE TRAILING STOP")]
    EXCHANGE_TRAILING_STOP,
    FOK,
    #[strum(serialize = "EXCHANGE FOK")]
    EXCHANGE_FOK,
    IOC,
    #[strum(serialize = "EXCHANGE IOC")]
    EXCHANGE_IOC,
}

impl Orders {
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Orders { client: Client::new(api_key, secret_key) }
    }

    pub fn active_orders(&self) -> Result<Vec<Order>> {
        let payload: String = format!("{}", "{}");

        self.orders("orders".to_owned(), payload)
    }

    pub fn history<T>(&self, symbol: T) -> Result<Vec<Order>>
    where
        T: Into<Option<String>>,
    {
        let value = symbol.into().unwrap_or("".into());
        let payload: String = format!("{}", "{}");

        if value.is_empty() {
            return self.orders("orders/hist".into(), payload);
        } else {
            let request: String = format!("orders/t{}/hist", value);
            return self.orders(request, payload);
        }
    }

    pub fn orders<S>(&self, request: S, payload: S) -> Result<Vec<Order>>
    where
        S: Into<String>,
    {
        let data = self.client.post_signed(request.into(), payload.into())?;
        let orders: Vec<Order> = from_str(data.as_str())?;

        Ok(orders)
    }
    pub fn submit_market_order<S, F>(&self, symbol: S, qty: F) -> Result<TradeResponse>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            order_type: OrderType::EXCHANGE_MARKET.to_string(),
            symbol: symbol.into(),
            amount: qty.into(),
            price: 0.0,
        };

        let order = self.build_order(order);
        let payload = to_string(&order)?;

        let data = self.client.post_signed_order("order/submit".into(), payload)?;
        println!("DATA: {:?}", data.as_str());
        let transaction: TradeResponse = from_str(data.as_str())?;

        println!("Trans: {:?}", transaction);
        Ok(transaction)
    }

    pub fn submit_limit_order<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<TradeResponse>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy: OrderRequest = OrderRequest {
            order_type: OrderType::EXCHANGE_LIMIT.to_string(),
            symbol: symbol.into(),
            amount: qty.into(),
            price,
        };

        let order = self.build_order(buy);
        let payload = to_string(&order)?;

        let data = self.client.post_signed_order("order/submit".into(), payload)?;
        println!("DATA: {:?}", data.as_str());
        let transaction: TradeResponse = from_str(data.as_str())?;

        println!("Trans: {:?}", transaction);
        Ok(transaction)
    }

    pub fn cancel_order(&self, order_id: i64) -> Result<TradeCancelResponse> {
        let mut parameters: BTreeMap<String, i64> = BTreeMap::new();
        parameters.insert("id".into(), order_id);
        let payload = to_string(&parameters)?;
        let data = self.client.post_signed_order("order/cancel".into(), payload)?;
        let order_canceled: TradeCancelResponse = from_str(data.as_str())?;

        Ok(order_canceled)
    }

    fn build_order(&self, order: OrderRequest) -> BTreeMap<String, String> {
        let mut order_parameters: BTreeMap<String, String> = BTreeMap::new();

        order_parameters.insert("symbol".into(), order.symbol);
        order_parameters.insert("type".into(), order.order_type);
        order_parameters.insert("amount".into(), order.amount.to_string());

        if order.price != 0.0 {
            order_parameters.insert("price".into(), order.price.to_string());
        }

        order_parameters
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_order_type() {
        let ot = OrderType::EXCHANGE_MARKET.to_string();
        assert_eq!(ot, "EXCHANGE MARKET");
        let ot = "EXCHANGE MARKET".parse::<OrderType>().unwrap();
        assert_eq!(ot, OrderType::EXCHANGE_MARKET);
    }

    #[test]
    fn test_to_trade_response() {
        // "[1693068623,\"on-req\",null,null,[[125309226058,null,1693068623334,\"tARBUSD\",1693068623334,1693068623334,-10,-10,\"EXCHANGE MARKET\",null,null,null,0,\"ACTIVE\",null,null,0.94244,0,0,0,null,null,null,0,0,null,null,null,\"API>BFX\",null,null,{}]],null,\"SUCCESS\",\"Submitting 1 orders.\"]"
    }
}
