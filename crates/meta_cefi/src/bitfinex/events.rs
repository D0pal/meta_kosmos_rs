use crate::bitfinex::book::{
    FundingCurrency as BookFundingCurrency, RawBook, TradingOrderBookLevel,
};
use serde::Deserialize;

use super::wallet::{
    FundingCreditSnapshot, NewOrderOnReq, OrderUpdateEvent, PositionSnapshot, TeEvent, TuEvent,
    WalletSnapshot, BU,
};

pub type SEQUENCE = u32;

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
    HeartbeatEvent(i32, String, SEQUENCE),     // "hb"
    CheckSumEvent(i32, String, i64, SEQUENCE), // "cs"
    WalletSnapshotEvent(i32, String, Vec<WalletSnapshot>, SEQUENCE, i32), // "ws"
    WalletUpdateEvent(i32, String, WalletSnapshot, SEQUENCE, i32), // "ws"
    PositionSnapshotEvent(i32, String, Vec<PositionSnapshot>, SEQUENCE, i32), // "ps"
    FundingCreditSnapshotEvent(i32, String, Vec<FundingCreditSnapshot>, SEQUENCE, i32), // fcs
    BuEvent(i32, String, BU, SEQUENCE, i32),   // bu
    TeEvent(i32, String, TeEvent, SEQUENCE, i32),
    OrderUpdateEvent(i32, String, OrderUpdateEvent, SEQUENCE, i32),
    TuEvent(i32, String, TuEvent, SEQUENCE, i32),
    NewOrderOnReq(i32, String, NewOrderOnReq, SEQUENCE),

    // TickerTradingEvent (i32, TradingPair),
    // TickerFundingEvent (i32, FundingCurrency),
    // TradesTradingSnapshotEvent (i32, Vec<TradesTradingPair>),
    // TradesTradingUpdateEvent (i32, String, TradesTradingPair),
    // TradesFundingSnapshotEvent (i32, Vec<TradesFundingCurrency>),
    // TradesFundingUpdateEvent (i32, String, TradesFundingCurrency),
    BookTradingSnapshotEvent(i32, Vec<TradingOrderBookLevel>, SEQUENCE),
    BookTradingUpdateEvent(i32, TradingOrderBookLevel, SEQUENCE),
    BookFundingSnapshotEvent(i32, Vec<BookFundingCurrency>),
    BookFundingUpdateEvent(i32, BookFundingCurrency),
    RawBookEvent(i32, RawBook),
    RawBookUpdateEvent(i32, Vec<RawBook>),
    // CandlesSnapshotEvent (i32, Vec<Candle>),
    // CandlesUpdateEvent (i32, Candle),
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
    use super::{DataEvent, NotificationEvent};
    use rust_decimal::{prelude::FromPrimitive, Decimal};
    use serde_json::from_str;

    #[test]
    fn should_deserilize_trading_book_event() {
        let data_str: &'static str = r#"[1,[[30367.1,7,1.1]]]"#;
        let event: DataEvent = from_str(data_str).unwrap();
        if let DataEvent::BookTradingSnapshotEvent(channel_id, snapshots, seq) = event {
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

    #[test]
    fn should_deserilize_wallet_snpahsot_event() {
        let data_str = "[0,\"ws\",[[\"exchange\",\"ETH\",0.01050273,0,null,null,null],[\"exchange\",\"OMG\",0.9,0,null,null,null]],2,3437]";
        let event: DataEvent = from_str(data_str).unwrap();
        println!("event {:?}", event);
    }

    #[test]
    fn should_deserilize_wallet_update_event() {
        let data_str = "[0,\"wu\",[\"exchange\",\"ETH\",0.01050273,0,0.01050273,null,null],7,3437]";
        let event: DataEvent = from_str(data_str).unwrap();
        println!("event {:?}", event);
    }

    #[test]
    fn should_deserialize_on_req_event() {
        let data_str = "[0,\"n\",[1693149294,\"on-req\",null,null,[125362310565,0,1693149294631,\"tARBUSD\",1693149294892,1693149294892,-10,-10,\"EXCHANGE MARKET\",null,null,null,0,\"ACTIVE\",null,null,0.96033,0,0,0,null,null,null,0,0,null,null,null,\"API>BFX\",null,null,{}],null,\"SUCCESS\",\"Submitting exchange market sell order for -10 ARB.\"],202]";
        let event: DataEvent = from_str(data_str).unwrap();
        println!("event {:?}", event);
    }

    #[test]
    fn test_te_event() {
        let data = "[0,\"te\",[1410994544,\"tARBUSD\",1693152682775,125268311676,-1,0.95851,\"EXCHANGE MARKET\",0.95851,-1,null,null,1693152682536],185,3452]";
        let event: DataEvent = from_str(data).unwrap();
        println!("event {:?}", event);
    }

    #[test]
    fn test_oc_event() {
        let data =    "[0,\"oc\",[125271920288,0,1693153165935,\"tARBUSD\",1693153166200,1693153166202,0,1,\"EXCHANGE MARKET\",null,null,null,0,\"EXECUTED @ 0.95902(1.0)\",null,null,0.9591,0.95902,0,0,null,null,null,0,0,null,null,null,\"API>BFX\",null,null,{}],197,3459]";
        let event: DataEvent = from_str(data).unwrap();
        println!("event {:?}", event);
    }

    #[test]
    fn should_deserilize_auth_event() {
        let data_str: &'static str = "{\"event\":\"auth\",\"status\":\"OK\",\"chanId\":0,\"userId\":1345240,\"auth_id\":\"b950467e-f808-46ce-8fc1-a4e29059c97b\",\"caps\":{\"orders\":{\"read\":1,\"write\":1},\"account\":{\"read\":1,\"write\":0},\"funding\":{\"read\":1,\"write\":0},\"history\":{\"read\":1,\"write\":0},\"wallets\":{\"read\":1,\"write\":1},\"withdraw\":{\"read\":0,\"write\":1},\"positions\":{\"read\":0,\"write\":0},\"ui_withdraw\":{\"read\":0,\"write\":0}}}";
        let event: NotificationEvent = from_str(data_str).unwrap();
        println!("event: {:?}", event);
    }

    #[test]
    fn should_deserilize_check_sum_event() {
        let data_str: &'static str = "[60715,\"cs\",-297359522,67]";
        let slice = &data_str[8..10];
        println!("slice {:?}", slice);
        let event: NotificationEvent = from_str(data_str).unwrap();
        println!("event: {:?}", event);
    }
}
