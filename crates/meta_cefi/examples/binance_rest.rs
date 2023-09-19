#![allow(dead_code)]

// use crate::api::*;
// use binance::userstream::*;
use meta_cefi::binance::{websockets::*, api::Binance, account::Account};
use std::sync::atomic::AtomicBool;

fn main() {
    let api_key = std::env::var("BNB_AK").expect("must provide BNB_AK");
    let secret_key = std::env::var("BNB_SK").expect("must provide BNB_SK");

    let account: Account = Binance::new(Some(api_key), Some(secret_key));
    let order_id = "1695091606311";
    match account.order_status("ETHUSDT", order_id) {
        Ok(answer) => println!("{:?}", answer),
        Err(e) => println!("Error: {}", e),
    }
}
