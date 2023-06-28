use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{debug, info};
use meta_common::enums::{Asset};
use crate::{
    bitfinex::{
        errors::*,
        symbol::*,
        common::*,
        events::{DataEvent, NotificationEvent},
        websockets::{EventHandler, WebSockets, EventType},
    },
    enums::CEX,
};

struct WebSocketHandler;

impl EventHandler for WebSocketHandler {
    fn on_connect(&mut self, event: NotificationEvent) {
        if let NotificationEvent::Info(info) = event {
            info!("bitfinex platform status: {:?}, version {}", info.platform, info.version);
        }
    }

    fn on_auth(&mut self, _event: NotificationEvent) {
        debug!("bitfinex on auth event {:?}", _event);
    }

    fn on_subscribed(&mut self, event: NotificationEvent) {
        if let NotificationEvent::TradingSubscribed(msg) = event {
            info!("bitfinex trading order book subscribed: {:?}", msg);
        }
    }

    fn on_data_event(&mut self, event: DataEvent) {
        if let DataEvent::BookTradingSnapshotEvent(channel, book_snapshot) = event {
            info!("bitfinex order book snapshot ({}) {:?}", channel, book_snapshot);
        } else if let DataEvent::BookTradingUpdateEvent(channel, book_update) = event {
            todo!()
        }
    }

    fn on_error(&mut self, message: Error) {
        println!("{:?}", message);
    }
}

#[derive(Debug, Clone)]
pub struct AccessKey {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Clone)]
pub struct CexConfig {
    keys: Option<BTreeMap<CEX, AccessKey>>,
}

pub struct CefiService {
    config: CexConfig,
    btf_sockets: BTreeMap<String, Arc<WebSockets>>,
}

impl CefiService {
    fn new(config: CexConfig) -> Self {
        Self { config, btf_sockets: BTreeMap::new() }
    }

    fn subscribe_book(&self, cex: CEX, base: Asset, quote: Asset) {
        let pair = get_pair(base, quote);
        match cex {
            CEX::BITFINEX => {
                if !self.btf_sockets.contains_key(&pair) {
                    let mut web_socket = WebSockets::new();
                    web_socket.add_event_handler(WebSocketHandler);
                    web_socket.connect().unwrap(); // check error
                    web_socket.subscribe_books(ETHUSD, EventType::Trading, P0, "F0", 100);
                    web_socket.event_loop().unwrap(); // check error
                }
            }
        }
    }
}

fn get_pair(base: Asset, quote: Asset) -> String {
    format!("{}_{}", base, quote)
}

#[cfg(test)]
mod test_cefi {
    use super::get_pair;
    use meta_common::enums::Asset;
    #[test]
    fn test_get_pair() {
        let ret = get_pair(Asset::ETH, Asset::USD);
        assert_eq!(ret, "ETH_USD");
    }
}