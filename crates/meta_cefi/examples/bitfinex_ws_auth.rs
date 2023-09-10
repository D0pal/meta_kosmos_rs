// let guard = init_tracing(app_config.log.into());
use meta_address::enums::Asset;
use meta_cefi::{
    bitfinex::wallet::{OrderUpdateEvent, TradeExecutionUpdate},
    cefi_service::{AccessKey, CefiService, CexConfig},
};
use meta_common::enums::CexExchange;
use meta_tracing::{init_tracing, TraceConfig};
use meta_util::time::get_current_ts;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{
    collections::BTreeMap,
    sync::{atomic::AtomicPtr, Arc},
    thread,
    time::Duration,
};
use tracing::Level;
use uuid::Uuid;

fn main() {
    let config = TraceConfig {
        file_name_prefix: "bitfinex_ob".to_string(),
        dir: "logs".to_string(),
        level: Level::DEBUG,
        flame: false,
        console: true,
    };
    let _guard = init_tracing(config);
    let AK = std::env::var("BTF_AK").expect("must provide BTF_AK");
    let SK = std::env::var("BTF_SK").expect("must provide BTF_SK");
    // let (tx, rx) = mpsc::sync_channel(100);
    let mut map = BTreeMap::new();
    map.insert(CexExchange::BITFINEX, AccessKey { api_key: AK, api_secret: SK });
    let cex_config = CexConfig { keys: Some(map) };

    let (tx_order, mut rx_order) = std::sync::mpsc::sync_channel::<TradeExecutionUpdate>(100);
    let mut cefi_service = CefiService::new(Some(cex_config), None, Some(tx_order));

    let cefi_service = &mut cefi_service as *mut CefiService;
    let cefi_service = Arc::new(AtomicPtr::new(cefi_service));

    thread::spawn(move || loop {
        let ou_event = rx_order.recv();
        println!("receive order update event {:?}", ou_event);
    });

    let _handle = {
        let cefi_service_clone = cefi_service.clone();
        thread::spawn(move || {
            let a = cefi_service_clone.load(std::sync::atomic::Ordering::Relaxed);
            unsafe {
                (*a).subscribe_book(CexExchange::BITFINEX, Asset::ARB, Asset::USD);
            }
        })
    };

    // handle.join();
    thread::sleep(Duration::from_secs(5));
    let request_id = get_current_ts().as_millis();
    let a = cefi_service.load(std::sync::atomic::Ordering::Relaxed);
    unsafe {
        (*a).submit_order(
            request_id,
            CexExchange::BITFINEX,
            Asset::ARB,
            Asset::USD,
            Decimal::from_f64(-200f64).unwrap(),
        );
    }
    thread::sleep(Duration::from_secs(5));
    std::process::exit(1);
}
