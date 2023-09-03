use ethers::prelude::*;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashbotsResult {
    pub status: String,
    pub hash: TxHash,
    pub max_block_number: u64,
    pub fast_mode: bool,
    pub seen_in_mempool: bool,
    pub sim_error: Option<String>,
}

pub struct MevClient {
    url: String,
    client: Client,
}

impl MevClient {
    pub fn new() -> Self {
        let mut default_header = header::HeaderMap::new();
        default_header.insert("Content-Type", header::HeaderValue::from_static("application/json"));
        let client = reqwest::ClientBuilder::new()
            .http2_keep_alive_while_idle(true)
            .default_headers(default_header)
            .build()
            .expect("unable to build http client");
        MevClient { url: "https://protect.flashbots.net".to_string(), client }
    }

    pub async fn get_private_tx(&self, tx_hash: TxHash) -> anyhow::Result<FlashbotsResult> {
        let url = format!("{}/tx/{:?}", self.url, tx_hash);
        let response = self.client.get(&url).send().await;
        match response {
            Ok(res) => {
                let info = res.json::<FlashbotsResult>().await;
                match info {
                    Ok(ret) => Ok(ret),
                    Err(_e) => todo!(),
                }
            }
            Err(_e) => todo!(),
        }
    }
}
