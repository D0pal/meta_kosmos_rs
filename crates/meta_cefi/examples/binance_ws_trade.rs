// let guard = init_tracing(app_config.log.into());
use core_affinity::CoreId;
use meta_address::enums::Asset;
use meta_cefi::{
    bitfinex::wallet::{OrderUpdateEvent, TradeExecutionUpdate, WalletSnapshot},
    cefi_service::{AccessKey, CefiService, CexConfig},
};
use meta_common::enums::CexExchange;
use meta_tracing::{init_tracing, TraceConfig};
use meta_util::time::get_current_ts;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{atomic::AtomicPtr, Arc},
    thread,
    time::Duration,
};
use tracing::Level;
use uuid::Uuid;

lazy_static::lazy_static! {
    pub static ref CORE_IDS: Vec<CoreId> = core_affinity::get_core_ids().unwrap();
}

fn main() {
    // core_affinity::set_for_current(CORE_IDS[4]);
    let config = TraceConfig {
        file_name_prefix: "binance".to_string(),
        dir: "logs".to_string(),
        level: Level::DEBUG,
        flame: false,
        console: true,
    };
    let _guard = init_tracing(config);
    let AK = std::env::var("BNB_AK").expect("must provide BTF_AK");
    let SK = std::env::var("BNB_SK").expect("must provide BTF_SK");
    // let (tx, rx) = mpsc::sync_channel(100);
    let mut map = BTreeMap::new();
    map.insert(CexExchange::BINANCE, AccessKey { api_key: AK, api_secret: SK });
    let cex_config = CexConfig { keys: Some(map) };

    let (tx_order, mut rx_order) = std::sync::mpsc::sync_channel::<TradeExecutionUpdate>(100);
    let (tx_wu, mut rx_wu) = std::sync::mpsc::sync_channel::<WalletSnapshot>(100);
    let mut cefi_service = CefiService::new(Some(cex_config), None, Some(tx_order), Some(tx_wu));

    // let cefi_service = &mut cefi_service as *mut CefiService;
    let cefi_service = RefCell::new(cefi_service);

    thread::spawn(move || {
        core_affinity::set_for_current(CORE_IDS[3]);

        loop {
            let ou_event = rx_order.recv();
            println!("receive order update event {:?}", ou_event);
        }
    });

    {
        let mut cefi_service_local = cefi_service.borrow_mut();
        cefi_service_local.connect_pair(CexExchange::BINANCE, Asset::ETH, Asset::USDT);
        thread::sleep(Duration::from_secs(5));
    }

    let request_id = get_current_ts().as_millis();

    println!("client order id: {:?}", request_id);
    {
        let mut cefi_service_2 = cefi_service.borrow_mut();
        (cefi_service_2).submit_order(
            request_id,
            CexExchange::BINANCE,
            Asset::ETH,
            Asset::USDT,
            Decimal::from_f64(-0.005f64).unwrap(),
        );
    }

    loop {}
}
