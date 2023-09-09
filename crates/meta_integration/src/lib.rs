use std::ops::Add;

use meta_address::enums::Asset;
use meta_model::ArbitrageSummary;
use reqwest::{header, Client};
use chrono::Utc;

#[derive(Debug)]
pub struct Lark {
    url: String,
    client: Client,
}

impl Lark {
    pub fn new(web_hook: String) -> Self {
        let mut default_header = header::HeaderMap::new();
        default_header.insert("Content-Type", header::HeaderValue::from_static("application/json"));
        let client = reqwest::ClientBuilder::new()
            .http2_keep_alive_while_idle(true)
            .default_headers(default_header)
            .build()
            .expect("unable to build http client");
       
        Self { client, url: web_hook }
    }

    pub async fn send_arbitrage_summary(&self, summary: ArbitrageSummary) {
        let mut base_net = summary.cex.base_amount.checked_add(summary.dex.base_amount).unwrap();
        let mut quote_net = summary.cex.quote_amount.checked_add(summary.dex.quote_amount).unwrap();

        if summary.cex.fee_token.eq(&summary.base) {
            base_net = base_net.add(summary.cex.fee_amount);
        } else {
            quote_net = quote_net.add(summary.cex.fee_amount);
        }

        let gas_fee_net = summary.dex.fee_amount;
        let content = format!(
            r#"
            base: {:?}, quote: {:?}
            time: {:?},
            bitfinex: {:?}({:?}), {:?}({:?}), price({:?}); fee: {:?}({:?}); ID: {:?}
            uniswap({:?}): {:?}({:?}), {:?}({:?}) price({:?}); gas fee: {:?}({:?}); hash: {:?}
            net: {:?}({:?}), {:?}({:?}), {:?}({:?})
        "#,
            summary.base,
            summary.quote,
            summary.datetime,
            summary.base, // start of cex
            summary.cex.base_amount,
            summary.quote,
            summary.cex.quote_amount,
            summary.cex.price,
            summary.cex.fee_token,
            summary.cex.fee_amount,
            summary.cex.id,
            summary.dex.network.map_or("".to_string(), |e| e.to_string()), // start of dex
            summary.base,
            summary.dex.base_amount,
            summary.quote,
            summary.dex.quote_amount,
            summary.dex.price,
            summary.dex.fee_token,
            summary.dex.fee_amount,
            summary.dex.id,
            summary.base, // start of net
            base_net,
            summary.quote,
            quote_net,
            summary.dex.fee_token,
            gas_fee_net,
        );
        let response = self
            .client
            .post(&self.url)
            .json(&serde_json::json!({
                "msg_type": "text",
                "content": {
                    "text": content,
                }
            }))
            .send()
            .await;
    }
}
