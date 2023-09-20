#![allow(dead_code)]


use hyper::{
    client::{connect::Connect},
};
use meta_cefi::binance::{
    api::Binance,
    http::{Credentials},
    hyper::BinanceHttpClient,
    stream::user_data,
    trade::{
        self,
    },
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKeyResult {
    pub listen_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("BNB_AK").expect("must provide BNB_AK");
    let secret_key = std::env::var("BNB_SK").expect("must provide BNB_SK");

    let credentials = Credentials::from_hmac(api_key, secret_key);
    let client = BinanceHttpClient::default().credentials(credentials);
    new_listen_key(&client).await;
    Ok(())
    // match account.order_status("ETHUSDT", order_id) {
    //     Ok(answer) => println!("{:?}", answer),
    //     Err(e) => println!("Error: {}", e),
    // }
}

async fn get_order<T>(client: &BinanceHttpClient<T>)
where
    T: Connect + Clone + Send + Sync + 'static,
{
    let orig_client_order_id = "1695091606311";
    let request =
        trade::get_order::GetOrder::new("ETHUSDT").orig_client_order_id(orig_client_order_id);
    let _data = client.send(request).await.unwrap().into_body_str().await.unwrap();
}

async fn new_listen_key<T>(client: &BinanceHttpClient<T>)
where
    T: Connect + Clone + Send + Sync + 'static,
{
    let request = user_data::new_listen_key();
    let data = client.send(request).await.unwrap().into_body_str().await.unwrap();
    let ret = serde_json::from_str::<ListenKeyResult>(&data).unwrap();
    println!("data: {:?}", ret.listen_key);
}
