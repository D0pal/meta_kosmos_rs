use meta_address::enums::Asset;
use meta_integration::Lark;
use meta_model::{ArbitrageOutcome, ArbitrageSummary};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use meta_common::enums::Network;
use chrono::prelude::*;

#[tokio::main]
async fn main() {
    let web_hook =
        "https://open.larksuite.com/open-apis/bot/v2/hook/722d27f3-fa80-4c79-8cf5-87970ce1712a";
    let lark = Lark::new(web_hook.to_string());
    let summary = ArbitrageSummary {
        datetime: Utc::now().to_rfc3339(),
        base: Asset::ARB,
        quote: Asset::USD,
        cex: ArbitrageOutcome {
            base_amount: Decimal::from_f64(10.0).unwrap(),
            quote_amount: Decimal::from_f64(-12.0).unwrap(),
            price: Decimal::from_f64(1.2).unwrap(),
            fee_token: Asset::ARB,
            fee_amount: Decimal::from_f64(0.01).unwrap(),
            id: "CID: 1234".to_string(),
            network: None,
        },
        dex: ArbitrageOutcome {
            base_amount: Decimal::from_f64(-10.0).unwrap(),
            quote_amount: Decimal::from_f64(12.12).unwrap(),
            price: Decimal::from_f64(1.212).unwrap(),
            fee_token: Asset::ETH,
            fee_amount: Decimal::from_f64(0.00012).unwrap(),
            id: "https://arbiscan.io/tx/0x5c673f2a4ef3f1de53ad3172c90d546ac10ee83fe36e8ee777365e43ee07eafe".to_string(),
            network: Some(Network::ARBI),
        },
    };
    lark.send_arbitrage_summary(summary).await;
}
