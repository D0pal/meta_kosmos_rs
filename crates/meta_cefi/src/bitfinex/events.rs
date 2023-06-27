use crate::bitfinex::book::{
    FundingCurrency as BookFundingCurrency, RawBook, TradingPair as BookTradingPair,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum NotificationEvent {
    Auth(AuthMessage),
    Info(InfoMessage),
    TradingSubscribed(TradingSubscriptionMessage),
    FundingSubscribed(FundingSubscriptionMessage),
    CandlesSubscribed(CandlesSubscriptionMessage),
    RawBookSubscribed(RawBookSubscriptionMessage),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DataEvent {
    // TickerTradingEvent (i32, TradingPair),
    // TickerFundingEvent (i32, FundingCurrency),
    // TradesTradingSnapshotEvent (i32, Vec<TradesTradingPair>),
    // TradesTradingUpdateEvent (i32, String, TradesTradingPair),
    // TradesFundingSnapshotEvent (i32, Vec<TradesFundingCurrency>),
    // TradesFundingUpdateEvent (i32, String, TradesFundingCurrency),
    BookTradingSnapshotEvent(i32, Vec<BookTradingPair>),
    BookTradingUpdateEvent(i32, BookTradingPair),
    BookFundingSnapshotEvent(i32, Vec<BookFundingCurrency>),
    BookFundingUpdateEvent(i32, BookFundingCurrency),
    RawBookEvent(i32, RawBook),
    RawBookUpdateEvent(i32, Vec<RawBook>),
    // CandlesSnapshotEvent (i32, Vec<Candle>),
    // CandlesUpdateEvent (i32, Candle),
    HeartbeatEvent(i32, String),
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
pub struct AuthMessage {
    pub event: String,
    pub status: String,
    pub chan_id: u32,
    pub code: Option<u32>,
    pub msg: Option<String>,
    pub user_id: Option<u32>,
    pub auth_id: Option<String>,
}

impl AuthMessage {
    pub fn is_ok(&self) -> bool {
        self.status == "OK"
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
pub struct InfoMessage {
    pub event: String,
    pub version: u16,
    pub server_id: String,
    pub platform: Platform,
}

#[derive(Debug, Deserialize)]
pub struct Platform {
    pub status: u16,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
pub struct TradingSubscriptionMessage {
    pub event: String,
    pub channel: String,
    pub chan_id: u32,
    pub symbol: String,
    pub pair: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
pub struct FundingSubscriptionMessage {
    pub event: String,
    pub channel: String,
    pub chan_id: u32,
    pub symbol: String,
    pub currency: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
pub struct CandlesSubscriptionMessage {
    pub event: String,
    pub channel: String,
    pub chan_id: u32,
    pub key: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
pub struct RawBookSubscriptionMessage {
    pub event: String,
    pub channel: String,
    pub chan_id: u32,
    pub symbol: String,
    pub prec: String,
    pub freq: String,
    pub len: String,
    pub pair: String,
}

#[cfg(test)]
mod test_events {
    use rust_decimal::{prelude::FromPrimitive, Decimal};
    use serde_json::{from_str};
    use super::DataEvent;

    #[test]
    fn should_deserilize_trading_book_event() {
        let data_str: &'static str = r#"[1,[[30367.1,7,1.1]]]"#;
        let event: DataEvent = from_str(data_str).unwrap();
        if let DataEvent::BookTradingSnapshotEvent(channel_id, snapshots) = event {
            assert_eq!(channel_id, 1);
            assert_eq!(snapshots.len(), 1);
            let snapshot = snapshots.get(0);
            assert!(snapshot.is_some());
            let snapshot = snapshot.unwrap();
            assert_eq!(snapshot.price, Decimal::from_f64(30367.1f64).unwrap());
            assert_eq!(snapshot.count, 7);
            assert_eq!(snapshot.amount, Decimal::from_f64(1.1f64).unwrap());
        } else {
            panic!("failed");
        }
    }
}
