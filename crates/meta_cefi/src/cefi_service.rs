use crate::{
    binance::{
        handler::BinanceEventHandlerImpl, util::get_binance_symbol,
        websockets_tokio::BinanceWebSocketClient,
    },
    bitfinex::{
        book::TradingOrderBookLevel,
        common::*,
        handler::BitfinexEventHandlerImpl,
        wallet::{TradeExecutionUpdate, WalletSnapshot},
        websockets::{BitfinexEventHandler, EventType, WebSockets},
    },
    get_cex_pair,
};
use meta_address::enums::Asset;
use meta_common::{
    enums::CexExchange,
    models::{CurrentSpread, MarcketChange},
};
use meta_util::time::get_current_ts;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    sync::{mpsc::SyncSender, Arc, RwLock},
};
use tokio::sync::RwLock as TokioRwLock;
extern crate core_affinity;
use core_affinity::CoreId;
use lazy_static::lazy_static;
use tracing::{error, info, warn};

lazy_static! {
    pub static ref CORE_IDS: Vec<CoreId> = core_affinity::get_core_ids().unwrap();
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessKey {
    pub api_key: String,
    pub api_secret: String,
}

pub type KeyedOrderBook = BTreeMap<Decimal, TradingOrderBookLevel>;

#[derive(Debug, Clone)]
pub struct CexConfig {
    pub keys: Option<BTreeMap<CexExchange, AccessKey>>,
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    bids: Vec<Decimal>,
    asks: Vec<Decimal>,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub bids: KeyedOrderBook,
    pub asks: KeyedOrderBook,
}

pub struct CefiService {
    config: Option<CexConfig>,
    sender_market_change_event: Option<SyncSender<MarcketChange>>,
    sender_order_event: Option<SyncSender<TradeExecutionUpdate>>, // send order update event
    sender_wu_event: Option<SyncSender<WalletSnapshot>>,
    bitfinex_sockets: BTreeMap<String, Arc<RwLock<WebSockets>>>, // (pair, (socket))
    binance_sockets: BTreeMap<String, Arc<TokioRwLock<BinanceWebSocketClient>>>, // (pair, (socket))
}

unsafe impl Send for CefiService {}
unsafe impl Sync for CefiService {}

impl CefiService {
    pub fn new(
        config: Option<CexConfig>,
        sender_market_change_event: Option<SyncSender<MarcketChange>>,
        sender_order_event: Option<SyncSender<TradeExecutionUpdate>>,
        sender_wu_event: Option<SyncSender<WalletSnapshot>>,
    ) -> Self {
        Self {
            config,
            sender_market_change_event,
            bitfinex_sockets: BTreeMap::new(),
            binance_sockets: BTreeMap::new(),
            sender_order_event,
            sender_wu_event,
        }
    }

    pub async fn connect_pair(&mut self, cex: CexExchange, base: Asset, quote: Asset) {
        let pair = get_pair(base, quote);
        match cex {
            CexExchange::BITFINEX => {
                let ak = self.config.as_ref().unwrap().keys.as_ref().unwrap().get(&cex).unwrap();
                if !self.bitfinex_sockets.contains_key(&pair) {
                    let sender_market_change_event_reader = self.sender_market_change_event.clone();
                    let sender_wu_event_reader = self.sender_wu_event.clone();
                    let sender_order_event_reader = self.sender_order_event.clone();
                    let handler_reader = BitfinexEventHandlerImpl::new(
                        sender_market_change_event_reader,
                        sender_wu_event_reader,
                        sender_order_event_reader,
                    );

                    let (mut socket_reader, mut socket_reader_backhand) =
                        WebSockets::new(Box::new(handler_reader));

                    (socket_reader).auth(
                        ak.api_key.to_string(),
                        ak.api_secret.to_string(),
                        false,
                        &[],
                    ); // check error
                    (socket_reader).conf();
                    (socket_reader).subscribe_books(
                        get_bitfinex_trade_symbol(base, quote),
                        EventType::Trading,
                        P0,
                        "F0",
                        100,
                    );

                    {
                        std::thread::spawn(move || {
                            let success = core_affinity::set_for_current(CORE_IDS[1]);
                            if !success {
                                warn!("bind core failure");
                            }
                            socket_reader_backhand.event_loop().unwrap();
                        });
                    }

                    self.bitfinex_sockets
                        .insert(pair.to_owned(), Arc::new(RwLock::new(socket_reader)));
                }
            }
            CexExchange::BINANCE => {
                if !self.binance_sockets.contains_key(&pair) {
                    let _sender_market_change_event_reader = self.sender_market_change_event.clone();
                    let _sender_wu_event_reader = self.sender_wu_event.clone();
                    let _sender_order_event_reader = self.sender_order_event.clone();
                    let handler_reader = BinanceEventHandlerImpl::new(
                        // sender_market_change_event_reader,
                        // sender_wu_event_reader,
                        // sender_order_event_reader,
                    );

                    let credential = self
                        .config
                        .as_ref()
                        .unwrap()
                        .keys
                        .as_ref()
                        .unwrap()
                        .get(&cex).cloned();

                    let (mut socket_client, mut socket_reader_backhand) =
                        BinanceWebSocketClient::new(credential, Box::new(handler_reader)).await;

                    {
                        tokio::spawn(async move {
                            socket_reader_backhand.event_loop().await;
                        });
                    }

                    socket_client.subscribe_books(get_binance_symbol(base, quote)).await;
                    self.binance_sockets
                        .insert(pair.to_owned(), Arc::new(TokioRwLock::new(socket_client)));
                }
            }
        }
    }

    pub async fn submit_order(
        &mut self,
        client_order_id: u128,
        cex: CexExchange,
        base: Asset,
        quote: Asset,
        amount: Decimal,
    ) {
        let pair = get_pair(base, quote);
        let time = get_current_ts().as_millis();
        info!(
            "start submit cex order cex: {:?}, pair: {:?}, amount: {:?}, ts: {:?}",
            cex, pair, amount, time
        );
        match cex {
            CexExchange::BITFINEX => {
                let symbol = get_cex_pair(cex, base, quote);
                if self.bitfinex_sockets.contains_key(&pair) {
                    let socket_reader = self.bitfinex_sockets.get(&pair).unwrap();
                    let mut _g_ret = socket_reader.write();
                    match _g_ret {
                        Ok(mut _g) => (_g).submit_order(client_order_id, symbol, amount),
                        Err(_e) => {
                            error!("error in acquire write lock");
                            std::process::exit(1);
                        }
                    }
                }
            }
            CexExchange::BINANCE => {
                let symbol = get_cex_pair(cex, base, quote);
                if self.binance_sockets.contains_key(&pair) {
                    let socket_reader = self.binance_sockets.get(&pair).unwrap();
                    let mut _g = socket_reader.write().await;
                    (_g).submit_order(client_order_id, symbol, amount).await;
                }
            }
        }
    }

    pub fn get_spread(&self, cex: CexExchange, base: Asset, quote: Asset) -> Option<CurrentSpread> {
        let pair = get_pair(base, quote);
        let mut best_ask = Decimal::default();
        let mut best_bid = Decimal::default();
        match cex {
            CexExchange::BITFINEX => {
                if self.bitfinex_sockets.contains_key(&pair) {
                    let web_socket = self.bitfinex_sockets.get(&pair);
                    if let Some(socket_reader) = web_socket {
                        let socket_reader_ret = socket_reader.read();
                        match socket_reader_ret {
                            Ok(_g) => {
                                if let Some(ref handler) = (_g).event_handler {
                                    let _g_ret = handler.read();
                                    match _g_ret {
                                        Ok(_g) => {
                                            let btf_handler = (_g.as_any())
                                                .downcast_ref::<BitfinexEventHandlerImpl>();

                                            if let Some(btf) = btf_handler {
                                                if let Some(ref ob) = btf.order_book {
                                                    if let Some((_, ask_level)) =
                                                        ob.asks.first_key_value()
                                                    {
                                                        best_ask = ask_level.price;
                                                    }
                                                    if let Some((_, bid_level)) =
                                                        ob.bids.last_key_value()
                                                    {
                                                        best_bid = bid_level.price;
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("acquie lock error {:?}", e);
                                            std::process::exit(1);
                                        }
                                    }
                                }
                            }
                            Err(_e) => {
                                error!("unable to acquire read lock");
                                std::process::exit(1);
                            }
                        }
                    }
                }
            }
            CexExchange::BINANCE => unimplemented!(),
        }
        if best_ask.is_zero() || best_bid.is_zero() {
            None
        } else {
            Some(CurrentSpread { best_bid, best_ask })
        }
    }
}

pub fn get_pair(base: Asset, quote: Asset) -> String {
    format!("{}_{}", base, quote)
}

pub fn get_bitfinex_trade_symbol(base: Asset, quote: Asset) -> String {
    format!("{}{}", base, quote)
}

pub fn construct_order_book(levels: Vec<TradingOrderBookLevel>) -> OrderBook {
    let bids: KeyedOrderBook = levels
        .iter()
        .filter(|x| x.amount.is_sign_positive())
        .map(|y| (y.price, y.clone()))
        .collect();

    let asks: KeyedOrderBook = levels
        .iter()
        .filter(|x| x.amount.is_sign_negative())
        .map(|y| {
            (y.price, {
                let mut l = y.clone();
                l.amount = l.amount.abs();
                l
            })
        })
        .collect();
    OrderBook { bids, asks }
}

pub fn update_order_book(ob: &mut OrderBook, book_update: TradingOrderBookLevel) {
    if book_update.count < 1 {
        // remove a price level
        if book_update.amount.is_sign_positive() {
            ob.bids.remove_entry(&book_update.price);
        } else {
            ob.asks.remove_entry(&book_update.price);
        }
    } else {
        // add or update a price level
        if !book_update.amount.is_sign_negative() {
            ob.bids
                .entry(book_update.price)
                .and_modify(|x| x.amount = book_update.amount.abs())
                .or_insert(book_update);
        } else {
            let mut cloned_level = book_update.clone();
            cloned_level.amount = book_update.amount.abs();
            ob.asks
                .entry(book_update.price)
                .and_modify(|x| x.amount = book_update.amount.abs())
                .or_insert(cloned_level);
        }
    }
}
#[cfg(test)]
mod test_cefi {
    use std::vec;

    use crate::{
        bitfinex::{book::TradingOrderBookLevel, events::DataEvent},
        util::to_decimal,
    };

    use super::*;
    use meta_address::enums::Asset;
    use rust_decimal::prelude::ToPrimitive;
    use serde_json::from_str;
    #[test]
    fn test_get_pair() {
        let ret = get_pair(Asset::ETH, Asset::USD);
        assert_eq!(ret, "ETH_USD");
    }

    #[test]
    fn test_get_bitfinex_trade_symbol() {
        let ret = get_bitfinex_trade_symbol(Asset::ARB, Asset::USD);
        assert_eq!(ret, "ARBUSD");
    }

    #[test]
    fn should_construct_order_book() {
        let data_str: &'static str = r#"[1,[[1000.1,7,1.1],[1003.4,1,-2.1],[1004.4,4,-5.1],[1000.2,5,2.1],[1002.4,2,-3.1],[999.2,3,3.1]],1]"#;
        let event: DataEvent = from_str(data_str).unwrap();
        if let DataEvent::BookTradingSnapshotEvent(_channel, book_snapshot, _seq) = event {
            let ob = construct_order_book(book_snapshot);
            let bid_book = ob.bids;
            let ask_book = ob.asks;

            assert_eq!(
                bid_book.keys().filter_map(|x| x.to_f64()).collect::<Vec<f64>>(),
                vec![999.2f64, 1000.1f64, 1000.2f64]
            );
            let (best_bid_key, best_bid_val) = bid_book.last_key_value().unwrap();
            assert_eq!(best_bid_key.to_f64(), Some(1000.2));
            assert_eq!(best_bid_val.count, 5);
            assert_eq!(best_bid_val.amount, to_decimal(2.1));

            assert_eq!(
                ask_book.keys().filter_map(|x| x.to_f64()).collect::<Vec<f64>>(),
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

    #[test]
    fn should_update_order_book() {
        let data_str: &'static str = r#"[1,[[1000.1,7,1.1],[1003.4,1,-2.1],[1004.4,4,-5.1],[1000.2,5,2.1],[1002.4,2,-3.1],[999.2,3,3.1]],1]"#;
        let event: DataEvent = from_str(data_str).unwrap();
        if let DataEvent::BookTradingSnapshotEvent(_channel, book_snapshot, _seq) = event {
            let mut ob = construct_order_book(book_snapshot);

            // remove a bid
            update_order_book(
                &mut ob,
                TradingOrderBookLevel {
                    price: to_decimal(1000.1f64),
                    amount: to_decimal(1.1),
                    count: 0,
                },
            );
            // add a bid
            update_order_book(
                &mut ob,
                TradingOrderBookLevel {
                    price: to_decimal(1000.1f64),
                    amount: to_decimal(1.1),
                    count: 2,
                },
            );
            println!("{:?}", ob);
        } else {
            panic!("test data deser failed");
        }
    }

    #[test]
    fn test_iter() {
        let a = [1, 2, 3];

        let mut iter = a.iter().rev();

        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));

        assert_eq!(iter.next(), None);
        println!("a {:?}", a);
    }
}
