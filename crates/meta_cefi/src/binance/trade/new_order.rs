use std::collections::BTreeMap;

use crate::binance::http::{request::Request, Credentials, Method};
use crate::binance::trade::order::{NewOrderResponseType, Side, TimeInForce};
use meta_util::time::get_current_ts;
use rust_decimal::Decimal;
use serde::Serialize;
/// `POST /api/v3/order`
///
/// Send in a new order.
///
/// * `LIMIT_MAKER` are `LIMIT` orders that will be rejected if they would immediately match and trade as a taker.
/// * `STOP_LOSS` and `TAKE_PROFIT` will execute a `MARKET` order when the `stopPrice` is reached.
/// * Any `LIMIT` or `LIMIT_MAKER` type order can be made an iceberg order by sending an `icebergQty`.
/// * Any order with an `icebergQty` MUST have `timeInForce` set to `GTC`.
/// * `MARKET` orders using `quantity` specifies how much a user wants to buy or sell based on the market price.
/// * `MARKET` orders using `quoteOrderQty` specifies the amount the user wants to spend (when buying) or receive (when selling) of the quote asset; the correct quantity will be determined based on the market liquidity and `quoteOrderQty`.
/// * `MARKET` orders using `quoteOrderQty` will not break `LOT_SIZE` filter rules; the order will execute a quantity that will have the notional value as close as possible to `quoteOrderQty`.
/// * same `newClientOrderId` can be accepted only when the previous one is filled, otherwise the order will be rejected.
///
/// Trigger order price rules against market price for both `MARKET` and `LIMIT` versions:
///
/// * Price above market price: `STOP_LOSS` `BUY`, `TAKE_PROFIT` `SELL`
/// * Price below market price: `STOP_LOSS` `SELL`, `TAKE_PROFIT` `BUY`
///
///
/// Weight(IP): 1
///
/// # Example
///
/// ```
/// use binance_spot_connector_rust::trade::{ self, order::Side };
/// use rust_decimal_macros::dec;
///
/// let request = trade::new_order("BNBUSDT", Side::Sell, "MARKET")
///     .quantity(dec!(0.1));
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct NewOrder {
    symbol: String,
    side: Side,
    #[serde(rename = "type")]
    ty: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "timeInForce")]
    time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quote_order_qty: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trailing_delta: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iceberg_qty: Option<Decimal>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "newOrderRespType")]
    new_order_resp_type: Option<NewOrderResponseType>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "recvWindow")]
    recv_window: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credentials: Option<Credentials>,
    #[serde(rename = "apiKey")]
    api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub timestamp: u128,
}

impl NewOrder {
    pub fn new(symbol: &str, side: Side, ty: &str, api_key: &str) -> Self {
        let ts = get_current_ts().as_millis();
        Self {
            symbol: symbol.to_owned(),
            side,
            ty: ty.to_owned(),
            api_key: api_key.to_owned(),
            timestamp: ts,
            signature: None,
            time_in_force: None,
            quantity: None,
            quote_order_qty: None,
            price: None,
            new_client_order_id: None,
            stop_price: None,
            trailing_delta: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            recv_window: None,
            credentials: None,
        }
    }

    pub fn time_in_force(mut self, time_in_force: TimeInForce) -> Self {
        self.time_in_force = Some(time_in_force);
        self
    }

    pub fn quantity(mut self, quantity: Decimal) -> Self {
        self.quantity = Some(quantity);
        self
    }

    pub fn quote_order_qty(mut self, quote_order_qty: Decimal) -> Self {
        self.quote_order_qty = Some(quote_order_qty);
        self
    }

    pub fn price(mut self, price: Decimal) -> Self {
        self.price = Some(price);
        self
    }

    pub fn new_client_order_id(mut self, new_client_order_id: &str) -> Self {
        self.new_client_order_id = Some(new_client_order_id.to_owned());
        self
    }

    pub fn stop_price(mut self, stop_price: Decimal) -> Self {
        self.stop_price = Some(stop_price);
        self
    }

    pub fn trailing_delta(mut self, trailing_delta: u64) -> Self {
        self.trailing_delta = Some(trailing_delta);
        self
    }

    pub fn iceberg_qty(mut self, iceberg_qty: Decimal) -> Self {
        self.iceberg_qty = Some(iceberg_qty);
        self
    }

    pub fn new_order_resp_type(mut self, new_order_resp_type: NewOrderResponseType) -> Self {
        self.new_order_resp_type = Some(new_order_resp_type);
        self
    }

    pub fn recv_window(mut self, recv_window: u64) -> Self {
        self.recv_window = Some(recv_window);
        self
    }

    pub fn credentials(mut self, credentials: &Credentials) -> Self {
        self.credentials = Some(credentials.clone());
        self
    }
}

// TODO: should use a macro
impl From<NewOrder> for Request {
    fn from(request: NewOrder) -> Request {
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        // params.insert(key, value)
        // let mut params = vec![
        params.insert("symbol".to_owned(), request.symbol.clone());
        params.insert("side".to_owned(), request.side.to_string());
        params.insert("type".to_owned(), request.ty.clone());
        params.insert("apiKey".to_owned(), request.api_key.to_string());
        params.insert("timestamp".to_owned(), request.timestamp.to_string());
        // ];

        if let Some(time_in_force) = request.time_in_force {
            params.insert("timeInForce".to_owned(), time_in_force.to_string());
        }

        if let Some(quantity) = request.quantity {
            params.insert("quantity".to_owned(), quantity.to_string());
        }

        if let Some(quote_order_qty) = request.quote_order_qty {
            params.insert("quoteOrderQty".to_owned(), quote_order_qty.to_string());
        }

        if let Some(price) = request.price {
            params.insert("price".to_owned(), price.to_string());
        }

        if let Some(ref new_client_order_id) = request.new_client_order_id {
            params.insert("newClientOrderId".to_owned(), new_client_order_id.clone());
        }

        if let Some(stop_price) = request.stop_price {
            params.insert("stopPrice".to_owned(), stop_price.to_string());
        }

        if let Some(trailing_delta) = request.trailing_delta {
            params.insert("trailingDelta".to_owned(), trailing_delta.to_string());
        }

        if let Some(iceberg_qty) = request.iceberg_qty {
            params.insert("icebergQty".to_owned(), iceberg_qty.to_string());
        }

        if let Some(new_order_resp_type) = request.new_order_resp_type {
            params.insert("newOrderRespType".to_owned(), new_order_resp_type.to_string());
        }

        if let Some(recv_window) = request.recv_window {
            params.insert("recvWindow".to_owned(), recv_window.to_string());
        }

        Request {
            body_json: Some(serde_json::to_value(request.clone()).unwrap()),
            path: "/api/v3/order".to_owned(),
            method: Method::Post,
            params,
            credentials: request.credentials,
            sign: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NewOrder, NewOrderResponseType, Side, TimeInForce};
    use crate::binance::http::{request::Request, Credentials, Method};
    use rust_decimal_macros::dec;

    static API_KEY: &str = "api-key";
    static API_SECRET: &str = "api-secret";

    #[test]
    fn trade_new_order_convert_to_request_test() {
        let credentials = Credentials::from_hmac(API_KEY.to_owned(), API_SECRET.to_owned());

        let request: Request = NewOrder::new(
            "BNBUSDT",
            Side::Sell,
            "MARKET",
            "api_key".to_string(),
            "signature".to_string(),
            1234u64,
        )
        .time_in_force(TimeInForce::Gtc)
        .quantity(dec!(10.1))
        .quote_order_qty(dec!(10.1))
        .price(dec!(10.1))
        .new_client_order_id("Test")
        .stop_price(dec!(10.1))
        .trailing_delta(100)
        .iceberg_qty(dec!(10.1))
        .new_order_resp_type(NewOrderResponseType::Ack)
        .recv_window(5000)
        .credentials(&credentials)
        .recv_window(5000)
        .credentials(&credentials)
        .into();

        // assert_eq!(
        //     request,
        //     Request {
        //         path: "/api/v3/order".to_owned(),
        //         credentials: Some(credentials),
        //         method: Method::Post,
        //         params: vec![
        //             ("symbol".to_owned(), "BNBUSDT".to_string()),
        //             ("side".to_owned(), "SELL".to_string()),
        //             ("type".to_owned(), "MARKET".to_string()),
        //             ("timeInForce".to_owned(), "GTC".to_string()),
        //             ("quantity".to_owned(), "10.1".to_string()),
        //             ("quoteOrderQty".to_owned(), "10.1".to_string()),
        //             ("price".to_owned(), "10.1".to_string()),
        //             ("newClientOrderId".to_owned(), "Test".to_string()),
        //             ("stopPrice".to_owned(), "10.1".to_string()),
        //             ("trailingDelta".to_owned(), "100".to_string()),
        //             ("icebergQty".to_owned(), "10.1".to_string()),
        //             ("newOrderRespType".to_owned(), "ACK".to_string()),
        //             ("recvWindow".to_owned(), "5000".to_string()),
        //         ],
        //         sign: true
        //     }
        // );
    }
}
