// let guard = init_tracing(app_config.log.into());
use meta_cefi::cefi_service::CefiService;
use meta_common::enums::{Asset, CexExchange};
use meta_tracing::{init_tracing, TraceConfig};
use std::sync::mpsc;
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
    let cefi_service = CefiService::new(None, None);
    cefi_service.subscribe_book(CexExchange::BITFINEX, Asset::ETH, Asset::USD);
}
