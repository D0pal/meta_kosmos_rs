use crate::{
    binance::{
        config::Config,
        errors::Result,
        http::{request::Request, Credentials},
        model::{
            AccountUpdateEvent, AggrTradesEvent, BalanceUpdateEvent, BookTickerEvent,
            DayTickerEvent, DepthOrderBookEvent, DiffOrderBookEvent, KlineEvent, OrderBook,
            OrderTradeEvent, TradeEvent,
        },
        trade::{
            self,
            order::{Side, TimeInForce},
        },
        util::sign,
    },
    cefi_service::AccessKey,
    WsBackendSender, WsMessage,
};
use error_chain::bail;
use meta_util::time::get_current_ts;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use std::{
    net::TcpStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc, RwLock,
    },
};
use tracing::{debug, error, info};
use tungstenite::{
    connect, handshake::client::Response, protocol::WebSocket, stream::MaybeTlsStream, Message,
};
use url::Url;
use uuid::Uuid;

use super::constants::BINANCE_TRADE_WSS_URL;

unsafe impl Send for BinanceSocketBackhand {}
unsafe impl Sync for BinanceSocketBackhand {}

pub trait BinanceEventHandler {
    fn on_data_event(&mut self, event: BinanceWebsocketEvent);
    fn as_any(&self) -> &dyn std::any::Any;
}

#[allow(clippy::all)]
enum WebsocketAPI {
    Default,
    MultiStream,
    Custom(String),
}

impl WebsocketAPI {
    fn params(self, subscription: &str) -> String {
        match self {
            WebsocketAPI::Default => format!("wss://stream.binance.com:9443/ws/{}", subscription),
            WebsocketAPI::MultiStream => {
                format!("wss://stream.binance.com:9443/stream?streams={}", subscription)
            }
            WebsocketAPI::Custom(url) => format!("{}/{}", url, subscription),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BinanceWebsocketEvent {
    AccountUpdate(AccountUpdateEvent),
    BalanceUpdate(BalanceUpdateEvent),
    OrderTrade(OrderTradeEvent),
    AggrTrades(AggrTradesEvent),
    Trade(TradeEvent),
    OrderBook(OrderBook),
    DayTicker(DayTickerEvent),
    DayTickerAll(Vec<DayTickerEvent>),
    Kline(KlineEvent),
    DiffOrderBook(DiffOrderBookEvent),
    DepthOrderBook(DepthOrderBookEvent),
    BookTicker(BookTickerEvent),
}

pub struct BinanceWebSockets {
    credentials: Option<AccessKey>,
    sender: WsBackendSender, // send request to backend,
    handler: Option<Arc<RwLock<Box<dyn BinanceEventHandler>>>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Events {
    Vec(Vec<DayTickerEvent>),
    BalanceUpdateEvent(BalanceUpdateEvent),
    DayTickerEvent(DayTickerEvent),
    BookTickerEvent(BookTickerEvent),
    AccountUpdateEvent(AccountUpdateEvent),
    OrderTradeEvent(OrderTradeEvent),
    AggrTradesEvent(AggrTradesEvent),
    TradeEvent(TradeEvent),
    KlineEvent(KlineEvent),
    OrderBook(OrderBook),
    DiffOrderBook(DiffOrderBookEvent),
    DepthOrderBookEvent(DepthOrderBookEvent),
}

impl BinanceWebSockets {
    pub fn new(
        credentials: Option<AccessKey>,
        subscription: &str,
        hander: Box<dyn BinanceEventHandler>,
    ) -> (BinanceWebSockets, BinanceSocketBackhand) {
        let socket_stream = Self::subscribe(subscription).unwrap();

        let wss: String = BINANCE_TRADE_WSS_URL.to_string();
        let url = Url::parse(&wss).unwrap();

        match connect(url) {
            Ok(answer) => {
                let (tx, rx) = channel::<WsMessage>();
                let sender = WsBackendSender { tx };

                let handler_box = Arc::new(RwLock::new(hander));
                // let handler: &'static Arc<RwLock<Box<dyn EventHandler>>> = &handler_box;
                let handle_clone = Arc::clone(&handler_box);
                let backhand = BinanceSocketBackhand::new(
                    Some(socket_stream),
                    answer.0,
                    rx,
                    Some(handle_clone),
                );
                let websockets = BinanceWebSockets {
                    credentials,
                    sender,
                    handler: Some(Arc::clone(&handler_box)),
                };
                (websockets, backhand)
            }
            Err(e) => {
                error!("error in connect socket {:?}", e);
                std::process::exit(1);
            }
        }
    }

    pub fn submit_order<S>(&mut self, client_order_id: u128, symbol: S, qty: Decimal)
    where
        S: Into<String>,
    {
        if let Some(ref ak) = self.credentials {
            let symbol_str: String = symbol.into();
            let credentials = Credentials::from_hmac(ak.api_key.clone(), ak.api_secret.clone());

            let side = if qty.is_sign_positive() { Side::Buy } else { Side::Sell };
            let mut request_order = trade::new_order(&symbol_str, side, "MARKET", &ak.api_key)
                .quantity(qty.abs())
                .new_client_order_id(&client_order_id.to_string());

            let ts = request_order.timestamp;

            let request: Request = request_order.clone().into();
            let params = request.params();
            println!("params: {:?}", params);

            let query_string = request.get_payload_to_sign();

            let signature = sign(&query_string, &credentials.signature).unwrap();
            let encoded_signature: String =
                url::form_urlencoded::byte_serialize(signature.as_bytes()).collect();

            let id = Uuid::new_v4().to_string();

            request_order.signature = Some(encoded_signature);
            let json_value = serde_json::to_value(&request_order).expect("Serialization failed");

            let msg = json!(
            {
                "id": id,
                "method": "order.place",
                "params":  json_value
            });

            println!("order to send {:?}", msg.to_string());

            if let Err(error_msg) = self.sender.send(crate::MessageChannel::Trade, &msg.to_string())
            {
                error!("submit_order error: {:?}", error_msg);
            }
        }
    }

    pub fn subscribe(subscription: &str) -> Result<WebSocket<MaybeTlsStream<TcpStream>>> {
        let wss = WebsocketAPI::Default.params(subscription);
        let url = Url::parse(&wss)?;
        Self::connect_wss(url)
    }

    // pub fn connect_with_config(&mut self, subscription: &str, config: &Config) -> Result<()> {
    //     self.connect_wss(&WebsocketAPI::Custom(config.ws_endpoint.clone()).params(subscription))
    // }

    // pub fn connect_multiple_streams(&mut self, endpoints: &[String]) -> Result<()> {
    //     self.connect_wss(&WebsocketAPI::MultiStream.params(&endpoints.join("/")))
    // }

    fn connect_wss(url: Url) -> Result<WebSocket<MaybeTlsStream<TcpStream>>> {
        match connect(url) {
            Ok(answer) => Ok(answer.0),
            Err(e) => bail!(format!("Error during handshake {}", e)),
        }
    }

    // pub fn disconnect(&mut self) -> Result<()> {
    //     if let Some(ref mut socket) = self.socket {
    //         socket.0.close(None)?;
    //         return Ok(());
    //     }
    //     bail!("Not able to close the connection");
    // }
}

pub struct BinanceSocketBackhand {
    rx: Receiver<WsMessage>, // any message received will send to trade socket
    pub socket_stream: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    pub socket_trade: WebSocket<MaybeTlsStream<TcpStream>>,
    event_handler: Option<Arc<RwLock<Box<dyn BinanceEventHandler>>>>,
}

impl BinanceSocketBackhand {
    pub fn new(
        socket_stream: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
        socket_trade: WebSocket<MaybeTlsStream<TcpStream>>,
        rx: Receiver<WsMessage>,
        event_handler: Option<Arc<RwLock<Box<dyn BinanceEventHandler>>>>,
    ) -> Self {
        Self { rx, socket_stream, socket_trade, event_handler }
    }

    pub fn event_loop(&mut self) -> Result<()> {
        loop {
            loop {
                match self.rx.try_recv() {
                    Ok(msg) => match msg {
                        WsMessage::Text(_, text) => {
                            println!("msg to send: {:?}", text);
                            let time = get_current_ts().as_millis();
                            info!("socket write message {:?}, time: {:?}", text, time);
                            let ret = self.socket_trade.write_message(Message::Text(text));
                            match ret {
                                Err(e) => error!("error in socket write {:?}", e),
                                Ok(()) => {}
                            }
                        }
                        WsMessage::Close => {
                            info!("socket close");
                            return self.socket_trade.close(None).map_err(|e| e.into());
                        }
                    },
                    Err(TryRecvError::Disconnected) => {
                        println!("disconnected");
                        bail!("Disconnected")
                    }
                    Err(TryRecvError::Empty) => {
                        // println!("empty message to send");
                        break;
                    }
                }
            }
            // loop {

            // }

            // if let Some(ref mut socket) = self.socket_stream {
            //     let message_ret = socket.read_message();
            //     match message_ret {
            //         Ok(message) => {
            //             match message {
            //                 Message::Text(text) => {
            //                     // println!("got msg: {:?}", text);
            //                     if let Some(ref mut h) = self.event_handler {
            //                         let mut _g_ret = h.write();
            //                         match _g_ret {
            //                             Ok(mut _g) => {
            //                                 let mut value: serde_json::Value =
            //                                     serde_json::from_str(&text)?;

            //                                 if let Some(data) = value.get("data") {
            //                                     value = serde_json::from_str(&data.to_string())?;
            //                                 }

            //                                 if let Ok(events) =
            //                                     serde_json::from_value::<Events>(value)
            //                                 {
            //                                     let action = match events {
            //                                         Events::Vec(v) => {
            //                                             BinanceWebsocketEvent::DayTickerAll(v)
            //                                         }
            //                                         Events::BookTickerEvent(v) => {
            //                                             BinanceWebsocketEvent::BookTicker(v)
            //                                         }
            //                                         Events::BalanceUpdateEvent(v) => {
            //                                             BinanceWebsocketEvent::BalanceUpdate(v)
            //                                         }
            //                                         Events::AccountUpdateEvent(v) => {
            //                                             BinanceWebsocketEvent::AccountUpdate(v)
            //                                         }
            //                                         Events::OrderTradeEvent(v) => {
            //                                             BinanceWebsocketEvent::OrderTrade(v)
            //                                         }
            //                                         Events::AggrTradesEvent(v) => {
            //                                             BinanceWebsocketEvent::AggrTrades(v)
            //                                         }
            //                                         Events::TradeEvent(v) => {
            //                                             BinanceWebsocketEvent::Trade(v)
            //                                         }
            //                                         Events::DayTickerEvent(v) => {
            //                                             BinanceWebsocketEvent::DayTicker(v)
            //                                         }
            //                                         Events::KlineEvent(v) => {
            //                                             BinanceWebsocketEvent::Kline(v)
            //                                         }
            //                                         Events::DiffOrderBook(v) => {
            //                                             BinanceWebsocketEvent::DiffOrderBook(v)
            //                                         }
            //                                         Events::OrderBook(v) => {
            //                                             BinanceWebsocketEvent::OrderBook(v)
            //                                         }
            //                                         Events::DepthOrderBookEvent(v) => {
            //                                             BinanceWebsocketEvent::DepthOrderBook(v)
            //                                         }
            //                                     };
            //                                     _g.on_data_event(action);
            //                                 }
            //                                 // let event: BinanceWebsocketEvent = from_str(&text)?;
            //                             }
            //                             Err(e) => {
            //                                 error!("error in acquire wirte lock");
            //                                 std::process::exit(1);
            //                             }
            //                         }
            //                     }
            //                 }
            //                 Message::Binary(_) => {}
            //                 Message::Ping(_) | Message::Pong(_) => {}
            //                 Message::Close(e) => {
            //                     bail!(format!("Disconnected {:?}", e));
            //                 }
            //                 _ => {}
            //             }
            //         }
            //         Err(e) => println!("error in read message {:?}", e),
            //     }
            // }

            // println!(
            //     "start read trade stream message: can read {:?}, config: {:?}",
            //     self.socket_trade.can_read(),
            //     self.socket_trade.get_config()
            // );
            // let trade_socket_message = self.socket_trade.read_message();
            // println!("end read stream message");
            // match trade_socket_message {
            //     Ok(message) => match message {
            //         Message::Text(msg) => {
            //             println!("receive msg: {:?}", msg);
            //             // if let Err(e) = self.handle_msg(&msg) {
            //             //     bail!(format!("Error on handling stream message: {}", e));
            //             // }
            //         }
            //         Message::Ping(_) => {
            //             self.socket_trade.write_message(Message::Pong(vec![])).unwrap();
            //         }
            //         Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => (),
            //         Message::Close(e) => bail!(format!("Disconnected {:?}", e)),
            //     },
            //     Err(e) => {
            //         eprintln!("error in read trade socket {:?}", e);
            //     }
            // }
        }
    }
}
