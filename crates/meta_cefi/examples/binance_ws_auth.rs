use futures_util::sink::SinkExt;
use meta_cefi::binance::http::Credentials;
use meta_cefi::binance::{
    http::request::Request,
    trade::{
        self,
        order::{Side, TimeInForce},
    },
    util::sign,
};
use meta_util::time::get_current_ts;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use ring::{hmac, rand};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Duration;
use tungstenite::{
    connect, handshake::client::Response, protocol::WebSocket, stream::MaybeTlsStream, Message,
};
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("BNB_AK").expect("must provide BNB_AK");
    let secret_key = std::env::var("BNB_SK").expect("must provide BNB_SK");

    let endpoint = "wss://ws-api.binance.com:443/ws-api/v3";

    // Create a WebSocket URL
    let url = Url::parse(endpoint)?;

    let credentials = Credentials::from_hmac(api_key.clone(), secret_key.clone());

    let price = Decimal::from_f64(1.2).unwrap();
    let qty = dec!(10.5);
    println!("side: {:?}", TimeInForce::Gtc.to_string());
    let mut request_order = trade::new_order("ARBUSDT", Side::Buy, "MARKET", &api_key)
        .quantity(qty);
        // .price(price)
        // .time_in_force(TimeInForce::Gtc);
    let ts = request_order.timestamp;

    let request: Request = request_order.clone().into();
    let params = request.params();
    println!("params: {:?}", params);


    let query_string = request.get_payload_to_sign();

    let signature = sign(&query_string, &credentials.signature).unwrap();
    let encoded_signature: String =
        url::form_urlencoded::byte_serialize(signature.as_bytes()).collect();
    let (mut ws_stream, _) = connect(url).expect("WebSocket connection failed");

    let id = Uuid::new_v4().to_string();


    request_order.signature= Some(encoded_signature);
    let json_value = serde_json::to_value(&request_order).expect("Serialization failed");

    let msg = json!(
    {
        "id": id,
        "method": "order.place",
        "params":  json_value
    });
    println!("msg: {:?}", msg.to_string());
    let message = Message::Text(msg.to_string());
    let a = ws_stream.write_message(message);


    loop {
        let message = ws_stream.read_message()?;
        match message {
            Message::Text(msg) => {
                println!("receive msg {}", msg);
            }
            Message::Ping(_) => {
                ws_stream.write_message(Message::Pong(vec![])).unwrap();
            }
            Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => (),
            Message::Close(e) => {
                println!("closed");
            }
        }
    }

    Ok(())
}

fn create_signature(
    secret_key: &str,
    timestamp: u128,
) -> Result<String, Box<dyn std::error::Error>> {
    let message = format!("timestamp={}", timestamp);
    let secret_key = hmac::Key::new(hmac::HMAC_SHA256, secret_key.as_bytes());
    let mut context = hmac::Context::with_key(&secret_key);
    context.update(message.as_bytes());
    let signature = context.sign();
    Ok(hex::encode(signature.as_ref()))
}
