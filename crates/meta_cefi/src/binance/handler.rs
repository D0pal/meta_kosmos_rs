use super::websockets::{BinanceEventHandler, BinanceWebsocketEvent};

unsafe impl Send for BinanceEventHandlerImpl{}
unsafe impl Sync for BinanceEventHandlerImpl{}

#[derive(Clone, Debug)]
pub struct BinanceEventHandlerImpl {
    // sender: Option<SyncSender<MarcketChange>>, // send market change
    // trade_execution_sender: Option<SyncSender<TradeExecutionUpdate>>, // tu event, contains fee information
    // wu_sender: Option<SyncSender<WalletSnapshot>>,
    // order_book: Option<OrderBook>,
    // sequence: u32,
}

impl BinanceEventHandlerImpl {
    pub fn new(// sender: Option<SyncSender<MarcketChange>>,
        // wu_sender: Option<SyncSender<WalletSnapshot>>,
        // order_sender: Option<SyncSender<TradeExecutionUpdate>>,
    ) -> Self {
        Self {
            // order_book: None,
            // sequence: 0,
            // sender,
            // trade_execution_sender: order_sender,
            // wu_sender,
        }
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
