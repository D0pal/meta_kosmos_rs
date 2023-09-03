#![allow(dead_code)]

// use crate::api::*;
// use binance::userstream::*;
use meta_cefi::binance::websockets::*;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    //user_stream();
    //user_stream_websocket();
    market_websocket();
    //kline_websocket();
    //all_trades_websocket();
    //last_price_for_one_symbol();
    // multiple_streams();
}

fn market_websocket() {
    let keep_running = AtomicBool::new(true); // Used to control the event loop
    let agg_trade = String::from("bnbbtc@depth");
    let mut web_socket: WebSockets<'_> = WebSockets::new(|event: WebsocketEvent| {
        match event {
            WebsocketEvent::Trade(trade) => {
                println!("Symbol: {}, price: {}, qty: {}", trade.symbol, trade.price, trade.qty);
            }
            WebsocketEvent::DepthOrderBook(depth_order_book) => {
                println!(
                    "depth order book, Symbol: {}, Bids: {:?}, Ask: {:?}",
                    depth_order_book.symbol, depth_order_book.bids, depth_order_book.asks
                );
            }
            WebsocketEvent::OrderBook(order_book) => {
                println!(
                    "order book, last_update_id: {}, Bids: {:?}, Ask: {:?}",
                    order_book.last_update_id, order_book.bids, order_book.asks
                );
            }
            WebsocketEvent::DiffOrderBook(order_book) => {
                println!(
                    "diff order book, final_update_id: {}, Bids: {:?}, Ask: {:?}",
                    order_book.final_update_id, order_book.bids, order_book.asks
                );
            }
            WebsocketEvent::AggrTrades(agg_trade) => {
                println!(
                    "aggregated_trade_id: {}, symbol: {}, price: {:?}, qty: {:?}",
                    agg_trade.aggregated_trade_id, agg_trade.symbol, agg_trade.price, agg_trade.qty
                );
            }
            _ => (),
        };

        Ok(())
    });

    println!("start connect to binance");
    web_socket.connect(&agg_trade).unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running) {
        println!("Error: {}", e);
    }
    web_socket.disconnect().unwrap();
    println!("disconnected");
}
