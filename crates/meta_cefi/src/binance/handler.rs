use meta_common::models::{CurrentSpread, MarcketChange};
use meta_util::decimal::decimal_from_str;

use crate::model::{CexEvent, TradeExecutionInfo};

use super::websockets::{BinanceEventHandler, BinanceWebsocketEvent};
use std::sync::mpsc::SyncSender;
use tracing::warn;

unsafe impl Send for BinanceEventHandlerImpl {}
unsafe impl Sync for BinanceEventHandlerImpl {}

#[derive(Clone, Debug)]
pub struct BinanceEventHandlerImpl {
    sender_market_change: Option<SyncSender<MarcketChange>>,
    sender_cex_event: Option<SyncSender<CexEvent>>,
}

impl BinanceEventHandlerImpl {
    pub fn new(
        sender_cex_event: Option<SyncSender<CexEvent>>,
        sender_market_change: Option<SyncSender<MarcketChange>>,
    ) -> Self {
        Self { sender_cex_event, sender_market_change }
    }
}
impl BinanceEventHandler for BinanceEventHandlerImpl {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_data_event(&mut self, event: BinanceWebsocketEvent) {
        match event {
            BinanceWebsocketEvent::BookTicker(ticker) => {
                if let Some(ref tx) = self.sender_market_change {
                    let ret = tx.send(MarcketChange {
                        cex: Some(CurrentSpread {
                            best_ask: decimal_from_str(&ticker.best_ask),
                            best_bid: decimal_from_str(&ticker.best_bid),
                        }),
                        dex: None,
                    });
                    match ret {
                        Ok(_) => {}
                        Err(e) => eprintln!("error in send binance market change event: {:?}", e),
                    }
                }
            }
            BinanceWebsocketEvent::OrderTrade(trade) => {
                if trade.event_type.eq("executionReport") && trade.order_status.eq("FILLED") {
                    if let Some(ref tx) = self.sender_cex_event {
                        let _ = tx.send(CexEvent::TradeExecution(TradeExecutionInfo {
                            symbol: trade.symbol,
                            order_id: trade.order_id,
                            exec_amount: decimal_from_str(&trade.qty),
                            exec_price: decimal_from_str(&trade.price_last_filled_trade),
                            order_type: trade.order_type,
                            fee: Some(decimal_from_str(&trade.commission)),
                            fee_currency: Some("BNB".to_string()),
                            client_order_id: trade.new_client_order_id.parse::<u64>().unwrap(),
                        }));
                    }
                }
            }
            _ => {
                warn!("got un handled binance event: {:?}", event);
            }
        }
    }
}
