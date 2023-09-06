use std::time::Instant;
use meta_address::enums::Asset;
use meta_common::enums::Network;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct ArbitrageOutcome {
    pub base: Decimal,
    pub quote: Decimal,
    pub price: Decimal,
    pub fee_token: Asset,
    pub fee_amount: Decimal,
    pub id: String,
    pub network: Option<Network>
}

#[derive(Debug)]
pub struct ArbitrageSummary {
    pub timestamp: Instant,
    pub base: Asset,
    pub quote: Asset,
    pub cex: ArbitrageOutcome,
    pub dex: ArbitrageOutcome,
}