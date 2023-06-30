// let guard = init_tracing(app_config.log.into());
use meta_tracing::{init_tracing, TraceConfig};
use tracing::Level;
use meta_cefi::{
    enums::CEX,
    cefi_service::CefiService
};
use meta_common::enums::Asset;

fn main() {
    let config = TraceConfig {
         file_name_prefix: "bitfinex_ob".to_string(),
         dir: "logs".to_string(),
         level: Level::DEBUG,
         flame: false,
         console: false,
    };
    let guard = init_tracing(config);

    let cefi_service = CefiService::new(None);
    cefi_service.subscribe_book(CEX::BITFINEX, Asset::ETH, Asset::USD);

}