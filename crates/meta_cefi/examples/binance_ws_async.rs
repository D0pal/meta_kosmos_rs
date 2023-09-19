
use meta_cefi::{
    binance::{
        handler::BinanceEventHandlerImpl,
        websockets_tokio::BinanceWebSocketClient,
    },
    cefi_service::AccessKey,
};
use meta_tracing::{init_tracing, TraceConfig};
use meta_util::time::get_current_ts;







use tracing::{Level};




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

    let _request_id = get_current_ts().as_millis();
    // ws.submit_order(request_id, "ETHUSDT", Decimal::from_f64(-0.005f64).unwrap()).await;
    ws.subscribe_books("ETHUSDT").await;

    loop {}

    Ok(())
}
