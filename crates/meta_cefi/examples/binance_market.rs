use meta_cefi::api::*;
use meta_cefi::config::*;
use meta_cefi::market::*;
use meta_cefi::errors::ErrorKind as BinanceLibErrorKind;

fn main() {
    // The general spot API endpoints; shown with
    // testnet=false and testnet=true
    // general(false);
    // general(true);

    // The market data API endpoint
    market_data();

    // The account data API and savings API endpoint examples need an API key. Change those lines locally
    // and uncomment the line below (and do not commit your api key :)).
    //account();
    //savings();
}

#[allow(dead_code)]
fn market_data() {
    let market: Market = Binance::new(None, None);

    // Order book at default depth
    // match market.get_depth("BNBETH") {
    //     Ok(answer) => println!("{:?}", answer),
    //     Err(e) => println!("Error: {}", e),
    // }
    // Order book at depth 500
    match market.get_custom_depth("BNBETH", 5000) {
        Ok(answer) => println!("{:?}", answer),
        Err(e) => println!("Error: {}", e),
    }

    // // Latest price for ALL symbols
    // match market.get_all_prices() {
    //     Ok(answer) => println!("{:?}", answer),
    //     Err(e) => println!("Error: {}", e),
    // }

    // // Latest price for ONE symbol
    // match market.get_price("KNCETH") {
    //     Ok(answer) => println!("{:?}", answer),
    //     Err(e) => println!("Error: {}", e),
    // }

    // // Current average price for ONE symbol
    // match market.get_average_price("KNCETH") {
    //     Ok(answer) => println!("{:?}", answer),
    //     Err(e) => println!("Error: {}", e),
    // }

    // // Best price/qty on the order book for ALL symbols
    // match market.get_all_book_tickers() {
    //     Ok(answer) => println!("{:?}", answer),
    //     Err(e) => println!("Error: {}", e),
    // }

    // // Best price/qty on the order book for ONE symbol
    // match market.get_book_ticker("BNBETH") {
    //     Ok(answer) => println!(
    //         "Bid Price: {}, Ask Price: {}",
    //         answer.bid_price, answer.ask_price
    //     ),
    //     Err(e) => println!("Error: {}", e),
    // }

    // // 24hr ticker price change statistics
    // match market.get_24h_price_stats("BNBETH") {
    //     Ok(answer) => println!(
    //         "Open Price: {}, Higher Price: {}, Lower Price: {:?}",
    //         answer.open_price, answer.high_price, answer.low_price
    //     ),
    //     Err(e) => println!("Error: {}", e),
    // }

    // // 10 latest (aggregated) trades
    // match market.get_agg_trades("BNBETH", None, None, None, Some(10)) {
    //     Ok(trades) => {
    //         let trade = &trades[0]; // You need to iterate over them
    //         println!(
    //             "{} BNB Qty: {}, Price: {}",
    //             if trade.maker { "SELL" } else { "BUY" },
    //             trade.qty,
    //             trade.price
    //         );
    //     }
    //     Err(e) => println!("Error: {}", e),
    // }


}
