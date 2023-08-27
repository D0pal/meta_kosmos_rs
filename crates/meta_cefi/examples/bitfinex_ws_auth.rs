// let guard = init_tracing(app_config.log.into());
use meta_address::enums::Asset;
use meta_cefi::cefi_service::{AccessKey, CefiService, CexConfig};
use meta_common::enums::CexExchange;
use meta_tracing::{init_tracing, TraceConfig};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::borrow::BorrowMut;
use std::sync::atomic::AtomicPtr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{
    collections::BTreeMap,
    sync::{mpsc, Mutex},
};
use tracing::Level;

fn main() {
    let config = TraceConfig {
        file_name_prefix: "bitfinex_ob".to_string(),
        dir: "logs".to_string(),
        level: Level::DEBUG,
        flame: false,
        console: true,
    };
    let guard = init_tracing(config);

    // let (tx, rx) = mpsc::sync_channel(100);
    let mut map = BTreeMap::new();
    map.insert(
        CexExchange::BITFINEX,
        AccessKey {
            api_key: "".to_string(),
            api_secret: "".to_string(),
        },
    );
    let cex_config = CexConfig { keys: Some(map) };
    let mut cefi_service = CefiService::new(Some(cex_config), None);

    let cefi_service = &mut cefi_service as *mut CefiService;
    let cefi_service = Arc::new(AtomicPtr::new(cefi_service));

    let handle = {
        let mut cefi_service_clone = cefi_service.clone();
        thread::spawn(move || {
            let a = cefi_service_clone.load(std::sync::atomic::Ordering::Relaxed);
            unsafe {
                (*a).subscribe_book(CexExchange::BITFINEX, Asset::ARB, Asset::USD);
            }
        })
    };

    // handle.join();
    thread::sleep(Duration::from_secs(5));
    let a = cefi_service.load(std::sync::atomic::Ordering::Relaxed);
    unsafe {
        (*a).submit_order(
            CexExchange::BITFINEX,
            Asset::ARB,
            Asset::USD,
            Decimal::from_f64(-1.5f64).unwrap(),
        );
    }
    thread::sleep(Duration::from_secs(5));
    std::process::exit(1);
}
