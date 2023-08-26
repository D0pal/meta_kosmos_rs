use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Serialize, Deserialize)]
struct SubPendingTx {
    jsonrpc: String,
    id: u64,
    method: String,
}

#[tokio::main]
async fn main() {
    let request_builder = http::request::Request::builder()
    .header("Authorization", "YTc5MThiYWUtNDA2OS00NWE2LWEyMmQtMjk5M2Y1YmU3YjU4OmVkMjBhMWVhMDM5MGQzODBjMzk3OWVjYjFiYzI2MDlm")
    .header("Sec-WebSocket-Key", "YTc5MThiYWUtNDA2OS00NWE2LWEyMmQtMjk5M2Y1YmU3YjU4OmVkMjBhMWVhMDM5MGQzODBjMzk3OWVjYjFiYzI2MDlm")
    .header("Host", "singapore.bsc.blxrbdn.com")
    .header("Connection", "keep-alive, Upgrade")
    .header("Upgrade", "websocket")
    .header("Sec-WebSocket-Version", 13)
    .uri("wss://singapore.bsc.blxrbdn.com/ws") // wss://singapore.bsc.blxrbdn.com/ws; ws://127.0.0.1:9002
    .body(()).unwrap();

    // let request = Url::parse("wss://singapore.bsc.blxrbdn.com/ws").expect("unable to parse url");

    println!("Sec-WebSocket-Key {:?}", request_builder.headers().get("Sec-WebSocket-Key"));
    println!("Host {:?}", request_builder.headers().get("Host"));
    let (mut socket, _) = connect_async(request_builder).await.unwrap();

    // let sub = SubPendingTx {
    //     jsonrpc: ""
    // }
    // let json = serde_json::to_string(&state).unwrap();
    let sub =serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "subscribe", "params": ["newTxs", {"include": ["tx_hash"], "blockchain_network": "BSC-Mainnet"}]}).to_string();
    println!("send message {}", sub);
    // socket.send(Message::Text(sub));

    let (mut write, mut read) = socket.split();
    write.send(Message::Text(sub)).await;

    while let Some(msg) = read.next().await {
        let msg = msg.unwrap();
        println!("received msg: {:?}", msg);
        //     if msg.is_text() || msg.is_binary() {
        //         socket.send(msg).await?;
        //     }
    }

    // while let Some(msg) = read..await {

    // }
}
