use crate::bitfinex::handler::CexEvent;

use super::websockets::{BinanceEventHandler, BinanceWebsocketEvent};
use std::sync::mpsc::SyncSender;

unsafe impl Send for BinanceEventHandlerImpl {}
unsafe impl Sync for BinanceEventHandlerImpl {}

#[derive(Clone, Debug)]
pub struct BinanceEventHandlerImpl {
    pub sender: Option<SyncSender<CexEvent>>,
}

impl BinanceEventHandlerImpl {
    pub fn new(sender: Option<SyncSender<CexEvent>>) -> Self {
        Self { sender }
    }
}
impl BinanceEventHandler for BinanceEventHandlerImpl {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_data_event(&mut self, event: BinanceWebsocketEvent) {
        println!("got binance event: {:?}", event);
    }
}
