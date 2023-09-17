use crate::bitfinex::api;

use self::{order::Side, new_order::NewOrder};

pub mod new_order;
pub mod order;

pub fn new_order(symbol: &str, side: Side, r#type: &str,  api_key: &str) -> NewOrder {
    NewOrder::new(symbol, side, r#type, api_key)
}