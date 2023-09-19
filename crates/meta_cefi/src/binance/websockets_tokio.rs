use crate::{
    binance::{
        http::request::Request,
        stream::Stream,
        trade::{
            self,
            order::{Side},
        },
        util::sign,
    },
    cefi_service::AccessKey,
    MessageChannel, WsBackendSenderAsync, WsMessage,
};

use futures_util::{SinkExt, TryStreamExt};
use meta_util::time::get_current_ts;
use rust_decimal::Decimal;
use serde_json::{json};
use std::sync::Arc;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    sync::{
        mpsc::{channel, error::TryRecvError, Receiver},
        RwLock,
    },
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{handshake::client::Response, protocol::Message, Error},
    MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info};
use url::Url;
use uuid::Uuid;

use super::{
    constants::{BINANCE_STREAM_WSS_BASE_URL, BINANCE_TRADE_WSS_URL},
    http::Credentials,
    hyper::BinanceHttpClient,
    stream::{
        market::BookTickerStream,
        user_data::{self, ListenKeyResult, UserDataStream},
    },
    websockets::{BinanceEventHandler, BinanceWebsocketEvent, Events},
};
/// Binance websocket client using Tungstenite.
pub struct BinanceWebSocketClient {
    credentials: Option<AccessKey>,
    sender: WsBackendSenderAsync, // send request to backend
    pub event_handler: Option<Arc<RwLock<Box<dyn BinanceEventHandler + Send + Sync>>>>,
    id: u64,
}

impl BinanceWebSocketClient {
    pub async fn new(
        credentials: Option<AccessKey>,
        hander: Box<dyn BinanceEventHandler + Send + Sync>,
    ) -> (BinanceWebSocketClient, BinanceSocketBackhandAsync) {
        let (socket_stream, _) =
            BinanceWebSocketClient::connect_async(BINANCE_STREAM_WSS_BASE_URL)
                .await
                .expect("Failed to connect");

        let (socket_trade, _) = BinanceWebSocketClient::connect_async(BINANCE_TRADE_WSS_URL)
            .await
            .expect("Failed to connect");

        // let wss: String = BINANCE_WEBSOCKET_TRADE_URL.to_string();
        // let url = Url::parse(&wss).unwrap();

        let (tx, rx) = channel::<WsMessage>(100);
        let sender = WsBackendSenderAsync { tx };

        let handler_box = Arc::new(RwLock::new(hander));
        // let handler: &'static Arc<RwLock<Box<dyn EventHandler>>> = &handler_box;
        let handle_clone = Arc::clone(&handler_box);
        let backhand =
            BinanceSocketBackhandAsync::new(socket_stream, socket_trade, rx, Some(handle_clone));
        let credentials_clone = credentials.clone();
        let mut websockets = BinanceWebSocketClient {
            credentials,
            sender,
            event_handler: Some(Arc::clone(&handler_box)),
            id: 0,
        };
        if let Some(ref ak) = credentials_clone {
            // start subscribe user data
            let credentials = Credentials::from_hmac(ak.api_key.clone(), ak.api_secret.clone());
            let client = BinanceHttpClient::default().credentials(credentials);
            let request = user_data::new_listen_key();
            let data = client.send(request).await.unwrap().into_body_str().await.unwrap();
            let lk = serde_json::from_str::<ListenKeyResult>(&data).unwrap();
            websockets.subscribe(vec![&UserDataStream::new(&lk.listen_key).into()]).await;
        }
        (websockets, backhand)
    }

    pub async fn connect_async(
        url: &str,
    ) -> Result<(WebSocketState<MaybeTlsStream<TcpStream>>, Response), Error> {
        let (socket, response) = connect_async(Url::parse(url).unwrap()).await?;

        info!("Connected to {}", url);
        debug!("Response HTTP code: {}", response.status());
        debug!("Response headers:");
        for (ref header, _value) in response.headers() {
            debug!("* {}", header);
        }

        Ok((WebSocketState::new(socket), response))
    }

    pub async fn subscribe_books<S>(&mut self, symbol: S)
    where
        S: Into<String>,
    {
        let symbol_str: String = symbol.into();
        self.subscribe(vec![&BookTickerStream::from_symbol(&symbol_str).into()]).await;
    }

    fn get_subscribe_message<'a>(
        &mut self,
        method: &str,
        params: impl IntoIterator<Item = &'a str>,
    ) -> Message {
        let mut params_str: String = params
            .into_iter()
            .map(|param| format!("\"{}\"", param))
            .collect::<Vec<String>>()
            .join(",");

        if !params_str.is_empty() {
            params_str = format!("\"params\": [{params}],", params = params_str)
        };

        let id = self.id;
        self.id += 1;

        let s = format!(
            "{{\"method\":\"{method}\",{params}\"id\":{id}}}",
            method = method,
            params = params_str,
            id = id
        );
        let message = Message::Text(s);
        debug!("Sent {}", message);
        message
        // self.socket.send(message).await.unwrap();

        // id
    }

    /// Sends `SUBSCRIBE` message for the given `streams`.
    ///
    /// `streams` are not validated. Invalid streams will be
    /// accepted by the server, but no data will be sent.
    /// Requests to subscribe an existing stream will be ignored
    /// by the server.
    ///
    /// Returns the message `id`. This should be used to match
    /// the request with a future response. Sent messages should
    /// not share the same message `id`.
    ///
    /// You should expect the server to respond with a similar
    /// message.
    /// ```json
    /// { "method": "SUBSCRIBE", "params": [ <streams> ], "id": <id> }
    /// ```
    pub async fn subscribe(&mut self, streams: impl IntoIterator<Item = &Stream>) {
        let msg = self.get_subscribe_message("SUBSCRIBE", streams.into_iter().map(|s| s.as_str()));
        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Stream, &msg.to_string()).await
        {
            error!("subscribe, error: {:?}", error_msg);
        }
    }

    /// Sends `UNSUBSCRIBE` message for the given `streams`.
    ///
    /// `streams` are not validated. Non-existing streams will be
    /// ignored by the server.
    ///
    /// Returns the message `id`. This should be used to match
    /// the request with a future response. Sent messages should
    /// not share the same message `id`.
    ///
    /// You should expect the server to respond with a similar
    /// message.
    /// ```json
    /// { "method": "UNSUBSCRIBE", "params": [ <streams> ], "id": <id> }
    /// ```
    pub async fn unsubscribe(&mut self, streams: impl IntoIterator<Item = &Stream>) {
        let msg =
            self.get_subscribe_message("UNSUBSCRIBE", streams.into_iter().map(|s| s.as_str()));
        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Stream, &msg.to_string()).await
        {
            error!("unsubscribe error: {:?}", error_msg);
        }
    }

    /// Sends `LIST_SUBSCRIPTIONS` message.
    ///
    /// Returns the message `id`. This should be used to match
    /// the request with a future response. Sent messages should
    /// not share the same message `id`.
    ///
    /// You should expect the server to respond with a similar
    /// message.
    /// ```json
    /// { "method": "LIST_SUBSCRIPTIONS", "params": [ <streams> ], "id": <id> }
    /// ```
    pub async fn subscriptions(&mut self) {
        let msg = self.get_subscribe_message("LIST_SUBSCRIPTIONS", vec![]);
        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Stream, &msg.to_string()).await
        {
            error!("subscriptions, error: {:?}", error_msg);
        }
    }

    pub async fn submit_order<S>(&mut self, client_order_id: u128, symbol: S, qty: Decimal)
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

            let _ts = request_order.timestamp;

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

            if let Err(error_msg) =
                self.sender.send(crate::MessageChannel::Trade, &msg.to_string()).await
            {
                error!("submit_order error: {:?}", error_msg);
            }
        }
    }
}

pub struct WebSocketState<T> {
    socket: WebSocketStream<T>,
    pub id: u64,
}

impl<T: AsyncRead + AsyncWrite + Unpin> WebSocketState<T> {
    pub fn new(socket: WebSocketStream<T>) -> Self {
        Self { socket, id: 0 }
    }

    pub async fn close(mut self) -> Result<(), Error> {
        self.socket.close(None).await
    }

    pub async fn write_message(
        &mut self,
        item: Message,
    ) -> std::result::Result<(), tokio_tungstenite::tungstenite::Error> {
        self.socket.send(item).await
    }
    pub async fn try_read_message(
        &mut self,
    ) -> std::result::Result<Option<Message>, tokio_tungstenite::tungstenite::Error> {
        self.socket.try_next().await
    }
}

impl<T> From<WebSocketState<T>> for WebSocketStream<T> {
    fn from(conn: WebSocketState<T>) -> WebSocketStream<T> {
        conn.socket
    }
}

impl<T> AsMut<WebSocketStream<T>> for WebSocketState<T> {
    fn as_mut(&mut self) -> &mut WebSocketStream<T> {
        &mut self.socket
    }
}

pub struct BinanceSocketBackhandAsync {
    rx: Receiver<WsMessage>, // any message received will send to trade socket
    pub socket_stream: WebSocketState<MaybeTlsStream<TcpStream>>,
    pub socket_trade: WebSocketState<MaybeTlsStream<TcpStream>>,
    event_handler: Option<Arc<RwLock<Box<dyn BinanceEventHandler + Send + Sync>>>>,
}

impl BinanceSocketBackhandAsync {
    pub fn new(
        socket_stream: WebSocketState<MaybeTlsStream<TcpStream>>,
        socket_trade: WebSocketState<MaybeTlsStream<TcpStream>>,
        rx: Receiver<WsMessage>,
        event_handler: Option<Arc<RwLock<Box<dyn BinanceEventHandler + Send + Sync>>>>,
    ) -> Self {
        Self { rx, socket_stream, socket_trade, event_handler }
    }

    pub async fn event_loop(&mut self) -> anyhow::Result<()> {
        loop {
            loop {
                match self.rx.try_recv() {
                    Ok(msg) => match msg {
                        WsMessage::Text(ty, text) => {
                            println!("msg to send: {:?}", text);
                            let time = get_current_ts().as_millis();
                            info!("socket write message {:?}, time: {:?}", text, time);
                            match ty {
                                MessageChannel::Stream => {
                                    let ret =
                                        self.socket_stream.write_message(Message::Text(text)).await;
                                    match ret {
                                        Err(e) => {
                                            eprintln!("error in write to socket stream {:?}", e);
                                        }
                                        Ok(()) => {
                                            println!("write to socket stream success");
                                        }
                                    }
                                }
                                MessageChannel::Trade => {
                                    let ret =
                                        self.socket_trade.write_message(Message::Text(text)).await;
                                    match ret {
                                        Err(e) => {
                                            eprintln!("error in socket write {:?}", e);
                                        }
                                        Ok(()) => {
                                            println!("write to socket success");
                                        }
                                    }
                                }
                            }
                        }
                        WsMessage::Close => {
                            println!("socket close");
                            // return self.socket_trade.close().await.map_err(|e| e.into());
                        }
                    },
                    Err(TryRecvError::Disconnected) => {
                        println!("disconnected from sender");
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        // println!("empty message to send");
                        break;
                    }
                }
            }

            let message_ret = self.socket_stream.try_read_message().await;

            match message_ret {
                Ok(None) => continue,
                Ok(Some(message)) => {
                    match message {
                        Message::Text(text) => {
                            // println!("got msg: {:?}", text);
                            if let Some(ref mut h) = self.event_handler {
                                let mut _g_ret = h.write().await;

                                let mut value: serde_json::Value = serde_json::from_str(&text)?;

                                if let Some(data) = value.get("data") {
                                    value = serde_json::from_str(&data.to_string())?;
                                }

                                if let Ok(events) = serde_json::from_value::<Events>(value) {
                                    let action = match events {
                                        Events::Vec(v) => BinanceWebsocketEvent::DayTickerAll(v),
                                        Events::BookTickerEvent(v) => {
                                            BinanceWebsocketEvent::BookTicker(v)
                                        }
                                        Events::BalanceUpdateEvent(v) => {
                                            BinanceWebsocketEvent::BalanceUpdate(v)
                                        }
                                        Events::AccountUpdateEvent(v) => {
                                            BinanceWebsocketEvent::AccountUpdate(v)
                                        }
                                        Events::OrderTradeEvent(v) => {
                                            BinanceWebsocketEvent::OrderTrade(v)
                                        }
                                        Events::AggrTradesEvent(v) => {
                                            BinanceWebsocketEvent::AggrTrades(v)
                                        }
                                        Events::TradeEvent(v) => BinanceWebsocketEvent::Trade(v),
                                        Events::DayTickerEvent(v) => {
                                            BinanceWebsocketEvent::DayTicker(v)
                                        }
                                        Events::KlineEvent(v) => BinanceWebsocketEvent::Kline(v),
                                        Events::DiffOrderBook(v) => {
                                            BinanceWebsocketEvent::DiffOrderBook(v)
                                        }
                                        Events::OrderBook(v) => BinanceWebsocketEvent::OrderBook(v),
                                        Events::DepthOrderBookEvent(v) => {
                                            BinanceWebsocketEvent::DepthOrderBook(v)
                                        }
                                    };
                                    _g_ret.on_data_event(action);
                                }
                                // let event: BinanceWebsocketEvent = from_str(&text)?;
                            }
                        }
                        Message::Binary(_) | Message::Pong(_) => {}
                        Message::Ping(_) => {
                            self.socket_stream.write_message(Message::Pong(vec![])).await?;
                        }
                        Message::Close(e) => {
                            error!("closed {:?}", e);
                        }
                        _ => {}
                    }
                }
                Err(e) => println!("error in read message {:?}", e),
            }
        }
    }
}
