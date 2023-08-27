use meta_cefi::bitfinex::api::*;
use std::time::{Duration, Instant};
fn main() {
    let apiKey = "".to_string();
    let apiSecret = "".to_string();
    let btf = Bitfinex::new(Some(apiKey), Some(apiSecret));

    let start = Instant::now();
    let resp = btf.orders.submit_market_order("tARBUSD", -10);
    let elapsed = Instant::now().duration_since(start).as_millis();
    println!("total elapsed {:?} ms, resp {:?}", elapsed, resp);
}
