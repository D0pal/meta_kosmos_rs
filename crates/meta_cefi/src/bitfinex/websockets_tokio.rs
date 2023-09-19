use super::{
    events::{DataEvent, NotificationEvent},
    websockets::{
        BitfinexEventHandler, EventType, AUTH, CONF, DEAD_MAN_SWITCH_FLAG, INFO, SUBSCRIBED,
        WEBSOCKET_URL,
    },
};
use crate::{
    bitfinex::{
        auth,
        common::{CONF_FLAG_SEQ_ALL, CONF_OB_CHECKSUM},
        errors::*,
        orders::OrderType,
    }, WsBackendSenderAsync, WsMessage,
};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use meta_util::time::get_current_ts;
use rust_decimal::Decimal;
use serde_json::{from_str, json};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{channel, error::TryRecvError, Receiver},
        RwLock,
    },
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{protocol::Message, Error},
    MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, warn};
use url::Url;

pub struct BitfinexWebSocketsAsync {
    sender: WsBackendSenderAsync, // send request to backend
    pub event_handler: Option<Arc<RwLock<Box<dyn BitfinexEventHandler + Send + Sync>>>>,
}

impl BitfinexWebSocketsAsync {
    pub async fn new(
        hander: Box<dyn BitfinexEventHandler + Send + Sync>,
    ) -> (BitfinexWebSocketsAsync, BitfinexSocketBackhandAsync) {
        let socket_stream = Self::connect_async(WEBSOCKET_URL).await.expect("Failed to connect");

        let (tx, rx) = channel::<WsMessage>(100);
        let sender = WsBackendSenderAsync { tx };

        let handler_box = Arc::new(RwLock::new(hander));
        let backhand =
            BitfinexSocketBackhandAsync::new(socket_stream, rx, Some(Arc::clone(&handler_box)));
        let websockets =
            BitfinexWebSocketsAsync { sender, event_handler: Some(Arc::clone(&handler_box)) };
        (websockets, backhand)
    }

    pub async fn connect_async(
        url: &str,
    ) -> std::result::Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
        let (socket, response) = connect_async(Url::parse(url).unwrap()).await?;

        info!("Connected to {}", url);
        debug!("Response HTTP code: {}", response.status());
        debug!("Response headers:");
        for (ref header, _value) in response.headers() {
            debug!("* {}", header);
        }

        Ok(socket)
    }

    // { event: 'conf', flags: CONF_FLAG_SEQ_ALL + CONF_OB_CHECKSUM }
    /// set configuration, defaults to seq and checksum
    pub async fn conf(&mut self) {
        let msg = json!(
        {
            "event": "conf",
            "flags": CONF_FLAG_SEQ_ALL + CONF_OB_CHECKSUM
        });

        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Trade, &msg.to_string()).await
        {
            error!("conf error: {:?}", error_msg);
        }
    }

    // pub fn add_event_handler<H>(&mut self, handler: H)
    // where
    //     H: EventHandler + 'static,
    // {
    //     self.event_handler = Some(Box::new(handler));
    // }

    /// Authenticates the connection.
    ///
    /// The connection will be authenticated until it is disconnected.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key
    /// * `api_secret` - The API secret
    /// * `dms` - Whether the dead man switch is enabled. If true, all account orders will be
    ///           cancelled when the socket is closed.
    pub async fn auth<S>(
        &mut self,
        api_key: S,
        api_secret: S,
        dms: bool,
        filters: &[&str],
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        let nonce = auth::generate_nonce()?;
        let auth_payload = format!("AUTH{}", nonce);
        let signature =
            auth::sign_payload(api_secret.as_ref().as_bytes(), auth_payload.as_bytes())?;

        let msg = json!({
            "event": "auth",
            "apiKey": api_key.as_ref(),
            "authSig": signature,
            "authNonce": nonce,
            "authPayload": auth_payload,
            "dms": if dms {Some(DEAD_MAN_SWITCH_FLAG)} else {None},
            "filters": filters,
        });

        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Trade, &msg.to_string()).await
        {
            error!("auth error: {:?}", error_msg);
        }

        Ok(())
    }

    pub async fn subscribe_ticker<S>(&mut self, symbol: S, et: EventType)
    where
        S: Into<String>,
    {
        let local_symbol = self.format_symbol(symbol.into(), et);
        let msg = json!({"event": "subscribe", "channel": "ticker", "symbol": local_symbol });

        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Stream, &msg.to_string()).await
        {
            error!("subscribe_ticker error: {:?}", error_msg);
        }
    }

    pub async fn subscribe_trades<S>(&mut self, symbol: S, et: EventType)
    where
        S: Into<String>,
    {
        let local_symbol = self.format_symbol(symbol.into(), et);
        let msg = json!({"event": "subscribe", "channel": "trades", "symbol": local_symbol });

        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Trade, &msg.to_string()).await
        {
            error!("subscribe_trades error: {:?}", error_msg);
        }
    }

    pub async fn subscribe_candles<S>(&mut self, symbol: S, timeframe: S)
    where
        S: Into<String>,
    {
        let key: String = format!("trade:{}:t{}", timeframe.into(), symbol.into());
        let msg = json!({"event": "subscribe", "channel": "candles", "key": key });

        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Trade, &msg.to_string()).await
        {
            error!("subscribe_candles error: {:?}", error_msg);
        }
    }

    /// subscribe order book
    /// The Order Books channel allows you to keep track of the state of the Bitfinex order book.
    /// Tt is provided on a price aggregated basis with customizable precision.
    /// Upon connecting, you will receive a snapshot of the book
    /// followed by updates for any changes to the state of the book.
    /// # Arguments
    /// prec: Level of price aggregation (P0, P1, P2, P3, P4). The default is P0. P0 has 5 Number of significant figures;
    ///       while P4 has 1 Number of significant figures
    /// freq: Frequency of updates (F0, F1). F0=realtime / F1=2sec. The default is F0.
    /// len: Number of price points ("1", "25", "100", "250") [default="25"]
    pub async fn subscribe_books<S, P, F>(
        &mut self,
        symbol: S,
        et: EventType,
        prec: P,
        freq: F,
        len: u32,
    ) where
        S: Into<String>,
        P: Into<String>,
        F: Into<String>,
    {
        let msg = json!(
        {
            "event": "subscribe",
            "channel": "book",
            "symbol": self.format_symbol(symbol.into(), et),
            "prec": prec.into(),
            "freq": freq.into(),
            "len": len
        });

        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Trade, &msg.to_string()).await
        {
            error!("subscribe_books error: {:?}", error_msg);
        }
    }

    pub async fn submit_order<S>(&mut self, client_order_id: u128, symbol: S, qty: Decimal)
    where
        S: Into<String>,
    {
        let symbol_str: String = symbol.into();
        let qty_str: String = qty.to_string();
        info!("websockets submit order symbol: {:?}, qty {:?}", symbol_str, qty_str);
        let msg = json!(
        [
            0,
            "on", // order new
            null,
            {
                "gid": 0,
                "cid": client_order_id,
                "type": OrderType::EXCHANGE_MARKET.to_string(),
                "symbol": symbol_str,
                "amount": qty_str
                // "meta":option
            }
        ]);

        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Trade, &msg.to_string()).await
        {
            // self.error_hander(error_msg);
            error!(
                "submit_order error, order is: {:?}, error is: {:?}",
                msg.to_string(),
                error_msg
            );
        }
    }

    pub async fn subscribe_raw_books<S>(&mut self, symbol: S, et: EventType)
    where
        S: Into<String>,
    {
        let msg = json!(
        {
            "event": "subscribe",
            "channel": "book",
            "prec": "R0",
            "pair": self.format_symbol(symbol.into(), et)
        });

        if let Err(error_msg) =
            self.sender.send(crate::MessageChannel::Stream, &msg.to_string()).await
        {
            error!("subscribe_raw_books error: {:?}", error_msg);
        }
    }

    // fn error_hander(&mut self, error_msg: Error) {
    //     if let Some(ref mut h) = self.event_handler {
    //         h.on_error(error_msg);
    //     }
    // }

    fn format_symbol(&mut self, symbol: String, et: EventType) -> String {
        match et {
            EventType::Funding => format!("f{}", symbol),
            EventType::Trading => format!("t{}", symbol),
        }
    }
}

pub struct BitfinexSocketBackhandAsync {
    rx: Receiver<WsMessage>,
    pub socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    event_handler: Option<Arc<RwLock<Box<dyn BitfinexEventHandler + Send + Sync>>>>,
}

impl BitfinexSocketBackhandAsync {
    pub fn new(
        socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
        rx: Receiver<WsMessage>,
        event_handler: Option<Arc<RwLock<Box<dyn BitfinexEventHandler + Send + Sync>>>>,
    ) -> Self {
        Self { rx, socket, event_handler }
    }

    pub async fn event_loop(&mut self) -> anyhow::Result<()> {
        loop {
            loop {
                match self.rx.try_recv() {
                    Ok(msg) => match msg {
                        WsMessage::Text(_, text) => {
                            let time = get_current_ts().as_millis();
                            info!("socket write message {:?}, time: {:?}", text, time);
                            let ret = self.socket.send(Message::Text(text)).await;
                            match ret {
                                Err(e) => {
                                    error!("error in socket write {:?}", e);
                                }
                                Ok(()) => {}
                            }
                        }
                        WsMessage::Close => {
                            error!("socket close");
                            return self.socket.close(None).await.map_err(|e| e.into());
                        }
                    },
                    Err(TryRecvError::Disconnected) => {
                        error!("disconnected from sender");
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        break;
                    }
                }
            }

            let message_ret = self.socket.try_next().await;

            match message_ret {
                Ok(None) => continue,
                Ok(Some(message)) => match message {
                    Message::Text(text) => {
                        println!("got msg: {:?}", text);
                        if let Some(ref mut h) = self.event_handler {
                            let mut _g = h.write().await;
                            if text.contains(INFO) {
                                let event: NotificationEvent = from_str(&text)?;
                                _g.on_connect(event);
                            } else if text.contains(SUBSCRIBED) {
                                let event: NotificationEvent = from_str(&text)?;
                                _g.on_subscribed(event);
                            } else if text.contains(AUTH) {
                                let event: NotificationEvent = from_str(&text)?;
                                _g.on_auth(event);
                            } else if text.contains(CONF) {
                                info!("got conf msg: {:?}", text);
                            } else {
                                debug!("receive raw event text: {:?}", text);
                                let event_ret = from_str::<DataEvent>(&text);
                                match event_ret {
                                    Ok(event) => {
                                        println!("parsed event: {:?}", event);
                                        _g.on_data_event(event);
                                    }
                                    Err(e) => {
                                        warn!("err {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    Message::Binary(_) | Message::Pong(_) => {}
                    Message::Ping(_) => {
                        self.socket.send(Message::Pong(vec![])).await;
                    }
                    Message::Close(e) => {
                        error!("Disconnected {:?}", e);
                    }
                    _ => {}
                },
                Err(e) => {
                    error!("error in read message {:?}", e)
                }
            }
        }
    }
}
