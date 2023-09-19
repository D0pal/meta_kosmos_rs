use crate::{
    binance::{
        handler::BinanceEventHandlerImpl, util::get_binance_symbol, websockets::BinanceWebSockets,
        websockets_tokio::BinanceWebSocketClient,
    },
    bitfinex::{
        book::TradingOrderBookLevel,
        common::*,
        errors::*,
        events::{DataEvent, NotificationEvent, SEQUENCE},
        wallet::{TradeExecutionUpdate, WalletSnapshot},
        websockets::{BitfinexEventHandler, EventType, WebSockets},
    },
    get_cex_pair, cefi_service::{OrderBook, construct_order_book, update_order_book},
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
    sync::{mpsc::SyncSender},
};
extern crate core_affinity;
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug)]
pub struct BitfinexEventHandlerImpl {
    sender: Option<SyncSender<MarcketChange>>, // send market change
    trade_execution_sender: Option<SyncSender<TradeExecutionUpdate>>, // tu event, contains fee information
    wu_sender: Option<SyncSender<WalletSnapshot>>,
    pub order_book: Option<OrderBook>,
    sequence: u32,
}

impl BitfinexEventHandlerImpl {
    pub fn new(
        sender: Option<SyncSender<MarcketChange>>,
        wu_sender: Option<SyncSender<WalletSnapshot>>,
        order_sender: Option<SyncSender<TradeExecutionUpdate>>,
    ) -> Self {
        Self {
            order_book: None,
            sequence: 0,
            sender,
            trade_execution_sender: order_sender,
            wu_sender,
        }
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

    fn log_order_book(&self) {
        debug!("new order book");

        if let Some(ref ob) = self.order_book {
            let asks_levels = if ob.asks.len() > 5 { 5 } else { ob.asks.len() };
            let mut iter = ob.bids.iter().rev().zip(ob.asks.iter());

            for _i in 0..asks_levels {
                if let Some((bid_item, ask_item)) = iter.next() {
                    let (p, level) = bid_item;
                    let (ask_p, ask_level) = ask_item;
                    debug!(
                        "{:>8?} {:>8?} | {:>8?} {:>8?}",
                        level.amount, p, ask_p, ask_level.amount
                    );
                }
            }
        }
    }
}
impl BitfinexEventHandler for BitfinexEventHandlerImpl {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
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

    fn on_checksum(&mut self, _event: i64) {
        // debug!("received checksum event: {:?}", event);
        // match event {
        //     DataEvent::CheckSumEvent(_a, _b, _c, sequence) => self.check_sequence(sequence),
        //     _ => panic!("checksum event expected"),
        // }
    }

    fn on_heart_beat(&mut self, _channel: i32, _data: String, _seq: SEQUENCE) {}

    fn on_data_event(&mut self, event: DataEvent) {
        println!("handle data event {:?}", event);
        if let DataEvent::HeartbeatEvent(a, b, seq) = event {
            debug!("handle heart beat event");
            self.check_sequence(seq);
            self.on_heart_beat(a, b, seq);
        } else if let DataEvent::CheckSumEvent(_a, _b, data, seq) = event {
            debug!("handle checksum event");
            self.check_sequence(seq);
            self.on_checksum(data);
        } else if let DataEvent::FundingCreditSnapshotEvent(_, _, _, seq, _) = event {
            debug!("handle fcs event {:?}", event);
            self.check_sequence(seq);
        } else if let DataEvent::NewOrderOnReq(_, _, _, seq) = event {
            debug!("handle on req event {:?}", event);
            self.check_sequence(seq);
        } else if let DataEvent::WalletUpdateEvent(_, _, wu, seq, _) = event {
            debug!("handle on wu event {:?}", wu);
            self.check_sequence(seq);
            match self.wu_sender {
                Some(ref tx) => {
                    let _ = tx.send(wu);
                }
                None => warn!("no wu sender"),
            };
        } else if let DataEvent::TradeExecutionEvent(_, ty, e, seq, _) = event {
            debug!("handle on trade execution update event type {:?}, {:?}", ty, e);
            self.check_sequence(seq);
            if ty.eq("tu") {
                match self.trade_execution_sender {
                    Some(ref tx) => {
                        let _ = tx.send(e);
                    }
                    None => warn!("no tx sender"),
                };
            }
        } else if let DataEvent::BuEvent(_, _, _, seq, _) = event {
            debug!("handle on bu event {:?}", event);
            self.check_sequence(seq);
        } else if let DataEvent::OrderUpdateEvent(_, order_event_type, _, seq, _) = event {
            debug!("handle order update type {:?}", order_event_type);
            self.check_sequence(seq);
        } else if let DataEvent::TuEvent(_, _, _, seq, _) = event {
            debug!("handle on tu event {:?}", event);
            self.check_sequence(seq);
        } else if let DataEvent::BookTradingSnapshotEvent(channel, book_snapshot, seq) = event {
            debug!("handle ob snapshot event sequence {:?}", { seq });
            info!("bitfinex order book snapshot channel({}) sequence({})", channel, seq);
            self.check_sequence(seq);
            self.order_book = Some(construct_order_book(book_snapshot));
        } else if let DataEvent::BookTradingUpdateEvent(channel, book_update, seq) = event {
            debug!("handle ob update event sequence {:?}", { seq });
            debug!(
                "bitfinex order book update channel({}) sequence({}) {:?}",
                channel, seq, book_update
            );
            self.check_sequence(seq);
            let prev_best_bid = self.order_book.as_ref().map_or(Decimal::default(), |ob| {
                ob.bids.last_key_value().map_or(Decimal::default(), |x| *x.0)
            });

            let prev_best_ask = self.order_book.as_ref().map_or(Decimal::default(), |ob| {
                ob.asks.first_key_value().map_or(Decimal::default(), |x| *x.0)
            });

            if let Some(ref mut ob) = self.order_book {
                update_order_book(ob, book_update);
            }
            let current_best_bid = self.order_book.as_ref().map_or(Decimal::default(), |ob| {
                ob.bids.last_key_value().map_or(Decimal::default(), |x| *x.0)
            });

            let current_best_ask = self.order_book.as_ref().map_or(Decimal::default(), |ob| {
                ob.asks.first_key_value().map_or(Decimal::default(), |x| *x.0)
            });

            if !current_best_ask.eq(&prev_best_ask) || !current_best_bid.eq(&prev_best_bid) {
                if let Some(ref tx) = self.sender {
                    // println!(
                    //     "send cex price change, current_best_ask: {:?}, current_best_bid: {:?} ",
                    //     current_best_ask, current_best_bid
                    // );
                    let ret = tx.send(MarcketChange {
                        cex: Some(CurrentSpread {
                            best_ask: current_best_ask,
                            best_bid: current_best_bid,
                        }),
                        dex: None,
                    });
                    match ret {
                        Ok(_) => {}
                        Err(e) => {
                            error!("error in send marcket change in bitfinex {:?}", e);
                        }
                    }
                }
            }

            // self.log_order_book();
        }
    }

    fn on_error(&mut self, message: Error) {
        error!("{:?}", message);
    }
}
