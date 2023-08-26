use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct CurrentSpread {
    pub best_bid: Decimal,
    pub best_ask: Decimal,
}

#[derive(Debug, Clone)]
pub struct MarcketChange {
    pub cex: Option<CurrentSpread>,
    pub dex: Option<CurrentSpread>,
}
