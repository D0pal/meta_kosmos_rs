use futures_util::{sink::SinkExt, StreamExt};
use meta_cefi::{
    binance::{
        constants::BINANCE_STREAM_WSS_BASE_URL,
        handler::BinanceEventHandlerImpl,
        http::{request::Request, Credentials},
        stream::{market::BookTickerStream, user_data::UserDataStream},
        trade::{
            self,
            order::{Side, TimeInForce},
        },
        util::sign,
        websockets_tokio::BinanceWebSocketClient,
    },
    cefi_service::AccessKey,
};
use meta_tracing::{init_tracing, TraceConfig};
use meta_util::time::get_current_ts;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use ring::{hmac, rand};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use serde_json::json;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{sync::RwLock, time::Duration};
use tracing::{debug, Level};
use tungstenite::{
    connect, handshake::client::Response, protocol::WebSocket, stream::MaybeTlsStream, Message,
};
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("BNB_AK").expect("must provide BNB_AK");
    let secret_key = std::env::var("BNB_SK").expect("must provide BNB_SK");

    let config = TraceConfig {
        file_name_prefix: "binance".to_string(),
        dir: "logs".to_string(),
        level: Level::DEBUG,
        flame: false,
        console: true,
    };
    let _guard = init_tracing(config);
    let handler_reader = Box::new(BinanceEventHandlerImpl::new());
    let (mut ws, mut backhend) = BinanceWebSocketClient::new(
        Some(AccessKey { api_key: api_key.clone(), api_secret: secret_key.clone() }),
        handler_reader,
    )
    .await;

    tokio::spawn(async move {
        backhend.event_loop().await;
    });

    let request_id = get_current_ts().as_millis();
    // ws.submit_order(request_id, "ETHUSDT", Decimal::from_f64(-0.005f64).unwrap()).await;
    ws.subscribe_books("ETHUSDT").await;

    loop {}

    Ok(())
}
