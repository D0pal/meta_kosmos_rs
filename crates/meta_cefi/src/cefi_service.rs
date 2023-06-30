use crate::{
    bitfinex::{
        book::TradingOrderBookLevel,
        common::*,
        errors::*,
        events::{DataEvent, NotificationEvent, SEQUENCE},
        symbol::*,
        websockets::{EventHandler, EventType, WebSockets},
    },
    enums::CEX,
};
use meta_common::enums::Asset;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::{
    borrow::Borrow,
    collections::{BTreeMap, VecDeque},
};
use tracing::{debug, info};

#[derive(Clone, Debug)]
pub struct BitfinexEventHandler {
    order_book: Option<OrderBook>,
    sequence: u32,
}
impl BitfinexEventHandler {
    pub fn new() -> Self {
        Self { order_book: None, sequence: 0 }
    }

    fn check_sequence(&mut self, seq: u32) {
        if self.sequence == 0 {
            self.sequence = seq;
        } else {
            if seq - self.sequence != 1 {
                panic!("out of sequence current {} received {}", self.sequence, seq);
            }
            self.sequence = seq;
        }
    }
}
impl EventHandler for BitfinexEventHandler {
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

    fn on_checksum(&mut self, event: NotificationEvent) {
        debug!("received checksum event: {:?}", event);
        match event {
            NotificationEvent::CheckSumEvent(_a, _b, _c, sequence) => self.check_sequence(sequence),
            _ => panic!("checksum event expected"),
        }
    }

    fn on_heart_beat(&mut self, channel: i32, data: String, seq: SEQUENCE) {
        debug!("received heart beat event: {:?}", data);
        self.check_sequence(seq);
    }

    fn on_data_event(&mut self, event: DataEvent) {
        if let DataEvent::BookTradingSnapshotEvent(channel, book_snapshot, seq) = event {
            info!(
                "bitfinex order book snapshot channel({}) sequence({}) {:?}",
                channel, seq, book_snapshot
            );
            self.check_sequence(seq);
            let (bids, asks) = construct_ob_snapshot(book_snapshot);
            self.order_book = Some(OrderBook { bids, asks });
        } else if let DataEvent::BookTradingUpdateEvent(channel, book_update, seq) = event {
            debug!(
                "bitfinex order book update channel({}) sequence({}) {:?}",
                channel, seq, book_update
            );
            self.check_sequence(seq);
            if book_update.count < 0 {
                debug!("delete level");
            } else {
                debug!("add or update level");
            }
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

pub type KeyedOrderBook = BTreeMap<Decimal, TradingOrderBookLevel>;

#[derive(Debug, Clone)]
pub struct CexConfig {
    keys: Option<BTreeMap<CEX, AccessKey>>,
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    bids: Vec<Decimal>,
    asks: Vec<Decimal>,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    // msg_count: u32, // 0 means snapshot; a value > 0 menas order book update
    bids: KeyedOrderBook,
    asks: KeyedOrderBook,
}

pub struct CefiService {
    config: Option<CexConfig>,
    btf_sockets: BTreeMap<String, Arc<WebSockets>>,
    btf_orderbooks: BTreeMap<String, OrderBook>,
}

impl CefiService {
    pub fn new(config: Option<CexConfig>) -> Self {
        Self { config, btf_sockets: BTreeMap::new(), btf_orderbooks: BTreeMap::new() }
    }

    pub fn subscribe_book(&self, cex: CEX, base: Asset, quote: Asset) {
        let pair = get_pair(base, quote);
        match cex {
            CEX::BITFINEX => {
                if !self.btf_sockets.contains_key(&pair) {
                    let mut web_socket = WebSockets::new();
                    let handler = BitfinexEventHandler::new();
                    web_socket.add_event_handler(handler);
                    web_socket.connect().unwrap(); // check error
                    web_socket.conf();
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

fn construct_ob_snapshot(levels: Vec<TradingOrderBookLevel>) -> (KeyedOrderBook, KeyedOrderBook) {
    let bid_book: KeyedOrderBook = levels
        .iter()
        .filter(|x| x.amount.is_sign_positive())
        .map(|y| (y.price.clone(), y.clone()))
        .collect();

    let ask_book: KeyedOrderBook = levels
        .iter()
        .filter(|x| x.amount.is_sign_negative())
        .map(|y| {
            (y.price.clone(), {
                let mut l = y.clone();
                l.amount = l.amount.abs();
                l
            })
        })
        .collect();
    (bid_book, ask_book)
}

#[cfg(test)]
mod test_cefi {
    use std::vec;

    use crate::{
        bitfinex::{book::TradingOrderBookLevel, events::DataEvent},
        cefi_service::KeyedOrderBook,
        util::to_decimal,
    };

    use super::{construct_ob_snapshot, get_pair};
    use meta_common::enums::Asset;
    use rust_decimal::{
        prelude::{FromPrimitive, ToPrimitive},
        Decimal,
    };
    use serde_json::from_str;
    #[test]
    fn test_get_pair() {
        let ret = get_pair(Asset::ETH, Asset::USD);
        assert_eq!(ret, "ETH_USD");
    }

    #[test]
    fn test_construct_ob_snapshot() {
        let data_str: &'static str = r#"[1,[[1000.1,7,1.1],[1003.4,1,-2.1],[1004.4,4,-5.1],[1000.2,5,2.1],[1002.4,2,-3.1],[999.2,3,3.1]],1]"#;
        let event: DataEvent = from_str(data_str).unwrap();
        if let DataEvent::BookTradingSnapshotEvent(channel, book_snapshot, seq) = event {
            let (bid_book, ask_book) = construct_ob_snapshot(book_snapshot);

            assert_eq!(
                bid_book.keys().into_iter().filter_map(|x| x.to_f64()).collect::<Vec<f64>>(),
                vec![999.2f64, 1000.1f64, 1000.2f64]
            );
            let (best_bid_key, best_bid_val) = bid_book.last_key_value().unwrap();
            assert_eq!(best_bid_key.to_f64(), Some(1000.2));
            assert_eq!(best_bid_val.count, 5);
            assert_eq!(best_bid_val.amount, to_decimal(2.1));

            assert_eq!(
                ask_book.keys().into_iter().filter_map(|x| x.to_f64()).collect::<Vec<f64>>(),
                vec![1002.4, 1003.4, 1004.4]
            );

            let (best_ask_key, best_ask_val) = ask_book.first_key_value().unwrap();
            assert_eq!(best_ask_key.to_f64(), Some(1002.4));
            assert_eq!(best_ask_val.count, 2);
            assert_eq!(best_ask_val.amount, to_decimal(3.1));
        } else {
            panic!("test data deser failed");
        }
    }

    fn test_ob_update() {
        let data_str: &'static str = r#"[1,[[1000.1,7,1.1],[1003.4,1,-2.1],[1004.4,4,-5.1],[1000.2,5,2.1],[1002.4,2,-3.1],[999.2,3,3.1]],1]"#;
        let event: DataEvent = from_str(data_str).unwrap();
        if let DataEvent::BookTradingSnapshotEvent(channel, book_snapshot, seq) = event {
            let (bid_book, ask_book) = construct_ob_snapshot(book_snapshot);

            // // add bid level
            // let book_update = TradingOrderBookLevel {

            // }
        } else {
            panic!("test data deser failed");
        }
    }
}
