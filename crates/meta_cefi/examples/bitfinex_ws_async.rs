use futures_util::sink::SinkExt;
use futures_util::StreamExt;

use meta_address::enums::Asset;
use meta_cefi::bitfinex::common::P0;
use meta_cefi::bitfinex::handler::BitfinexEventHandlerImpl;
use meta_cefi::bitfinex::wallet::{TradeExecutionUpdate, WalletSnapshot};
use meta_cefi::bitfinex::websockets_tokio::{BitfinexSocketBackhandAsync, BitfinexWebSocketsAsync};
use meta_cefi::cefi_service::{get_bitfinex_trade_symbol, AccessKey};
use meta_common::models::MarcketChange;
use meta_tracing::{init_tracing, TraceConfig};
use meta_util::time::get_current_ts;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use ring::{hmac, rand};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{debug, Level};
use tungstenite::{
    connect, handshake::client::Response, protocol::WebSocket, stream::MaybeTlsStream, Message,
};
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("BTF_AK").expect("must provide BTF_AK");
    let secret_key = std::env::var("BTF_SK").expect("must provide BTF_SK");

    let config = TraceConfig {
        file_name_prefix: "bitfinex".to_string(),
        dir: "logs".to_string(),
        level: Level::DEBUG,
        flame: false,
        console: true,
    };
    let _guard = init_tracing(config);
    let (sender_market_change_event_reader, rx) = sync_channel::<MarcketChange>(1000);
    let (sender_wu_event_reader, _) = sync_channel::<WalletSnapshot>(1000);
    let (sender_order_event_reader, _) = sync_channel::<TradeExecutionUpdate>(1000);
    let handler_reader = Box::new(BitfinexEventHandlerImpl::new(
        Some(sender_market_change_event_reader),
        Some(sender_wu_event_reader),
        Some(sender_order_event_reader),
    ));
    let (mut ws, mut backhend) = BitfinexWebSocketsAsync::new(
        // Some(AccessKey { api_key: api_key.clone(), api_secret: secret_key.clone() }),
        handler_reader,
    )
    .await;

    tokio::spawn(async move {
        backhend.event_loop().await;
    });

    let request_id = get_current_ts().as_millis();
    (ws).auth(api_key.to_string(), secret_key.to_string(), false, &[]).await; // check error
    (ws).conf().await;
    ws.subscribe_books(
        get_bitfinex_trade_symbol(Asset::ARB, Asset::USD),
        meta_cefi::bitfinex::websockets::EventType::Trading,
        P0,
        "F0",
        100,
    )
    .await;

    loop {
        if let Ok(change) = rx.recv() {
            println!("change: {:?}", change);
        }
    }

    Ok(())
}
