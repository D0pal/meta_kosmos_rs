use meta_util::{int_from_hex_str};
use reqwest::header;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CommonClub48Result {
    jsonrpc: String,
    id: u64,
    result: String,
}

struct Club48Client {
    url: String,
    id: u64,
    client: reqwest::Client,
}
impl Club48Client {
    fn new(url: String) -> Club48Client {
        let mut default_header = header::HeaderMap::new();
        default_header.insert("Content-Type", header::HeaderValue::from_static("application/json"));
        let client = reqwest::ClientBuilder::new()
            .http2_keep_alive_while_idle(true)
            .default_headers(default_header)
            .build()
            .expect("unable to build http client");
        Club48Client { url, client, id: 0 }
    }
    async fn query_gas_price_floor(&mut self) -> Result<u64, reqwest::Error> {
        self.id += 1;
        let response = self
            .client
            .post(&self.url)
            .json(&serde_json::json!({
                "id": self.id,
                "jsonrpc": "2.0",
                "method": "eth_gasPrice"
            }))
            .send()
            .await?;
        let gas_price = response.json::<CommonClub48Result>().await.unwrap();
        Ok(int_from_hex_str(&gas_price.result))
    }

    /// send private tx to club 48
    /// return txHash
    async fn send_private_transaction(&mut self, tx: &str) -> Result<String, reqwest::Error> {
        self.id += 1;
        let response = self
            .client
            .post(&self.url)
            .json(&serde_json::json!({
                "id": self.id,
                "jsonrpc": "2.0",
                "method": "eth_sendPrivateRawTransaction",
                "params": [
                    tx
                ]
            }))
            .send()
            .await?;
        let info = response.json::<CommonClub48Result>().await.unwrap();
        Ok(info.result)
    }

    async fn send_bundled_transaction(
        &mut self,
        txs: Vec<&str>,
        max_timestamp: u64,
    ) -> Result<String, reqwest::Error> {
        self.id += 1;
        let response = self
            .client
            .post(&self.url)
            .json(&serde_json::json!({
                "id": self.id,
                "jsonrpc": "2.0",
                "method": "eth_sendPuissant",
                "params": [
                    {
                        "txs": txs,
                        "maxTimestamp": max_timestamp,
                        "acceptRevert": []
                    }
                ]
            }))
            .send()
            .await?;
        println!("status code: {:?}", response.status());
        let info = response.json::<CommonClub48Result>().await;
        match info {
            Ok(ret) => println!("{:?}", ret),
            Err(err) => eprint!("got error{}", err),
        }
        Ok("HI".to_string())
    }
}

#[cfg(test)]
mod test_client {
    use meta_util::{time::get_current_ts};
    use std::vec;

    use super::Club48Client;

    #[tokio::test]
    async fn test_query_gas_info() {
        let mut client = Club48Client::new("https://puissant-bsc.48.club".to_owned());
        client.query_gas_price_floor().await;
        client.query_gas_price_floor().await;
    }

    #[tokio::test]
    async fn test_send_private_tx() {
        let tx = "0xf86c03850df847580082520894dc8a5fc5222cb86b47bdbd8d5d45633b8d9ccff787038d7ea4c68000808193a0df6fc128d463758765b78a31f68baf106cbdab359919e498039c6ad61c171ac1a0589d446be554e571a5b09b35a5e7377b32f5e0bb78e00ece524c9739995c8646";
        let mut client = Club48Client::new("https://puissant-bsc.48.club".to_owned());
        client.send_private_transaction(tx).await;
    }

    #[tokio::test]
    async fn send_bundled_transaction() {
        let current_ts = get_current_ts().as_secs();
        let inputs = vec![
            "0xf86c0485104c533c0082520894dc8a5fc5222cb86b47bdbd8d5d45633b8d9ccff787038d7ea4c68000808194a06e14e6290eb35ba8c0ec9c29fcb52734a627a69ceaa8f2952c99b0b362db1999a0217eca68fb8bd59457ca4297ab9a29bc3a9a57443a88991984c253b4013348c7", 
            "0xf86b0584b2d05e0082520894dc8a5fc5222cb86b47bdbd8d5d45633b8d9ccff787038d7ea4c68000808194a0597dd4e6cf097609c57ead4514d61ab6fb37862f38b6538fafc3d9b20c38319ba06b4de3e37dd0eb684a87e8528632aced39dc9a7a7d93b88342834dfd003be7af"
            ];
        // let tx = "0xf86c03850df847580082520894dc8a5fc5222cb86b47bdbd8d5d45633b8d9ccff787038d7ea4c68000808193a0df6fc128d463758765b78a31f68baf106cbdab359919e498039c6ad61c171ac1a0589d446be554e571a5b09b35a5e7377b32f5e0bb78e00ece524c9739995c8646";
        let mut client = Club48Client::new("https://puissant-bsc.48.club".to_owned());
        client.send_bundled_transaction(inputs, current_ts + 60).await;
        // 0x36c2f08b7f134a0a856c606307504c5a
    }
}
