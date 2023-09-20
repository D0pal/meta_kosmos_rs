use rust_decimal::Decimal;

use crate::bitfinex::wallet::WalletSnapshot;

#[derive(Debug, Clone)]
pub struct TradeExecutionInfo {
    pub client_order_id: u64, // trade.new_client_order_id.parse::<u64>().unwrap(),
    pub order_id: u64,        // trade.order_id,   // Order id
    pub symbol: String,       // trade.symbol,
    pub exec_amount: Decimal, // decimal_from_str(&trade.qty) , // Positive means buy, negative means sell
    pub exec_price: Decimal,  // decimal_from_str(&trade.price),  // Execution price
    pub order_type: String,   // trade.order_type,
    pub fee: Option<Decimal>, // Some(decimal_from_str(&trade.commission)),         // Fee ('tu' only)
    pub fee_currency: Option<String>, // Some("BNB".to_string()), // Fee currency ('tu' only)
}

pub enum CexEvent {
    TradeExecution(TradeExecutionInfo),
    Balance(WalletSnapshot),
}
