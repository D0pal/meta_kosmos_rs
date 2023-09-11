use ethers::prelude::*;
use meta_address::{get_dex_address, get_rpc_info, get_token_info, Token};
use meta_common::enums::{ContractType, DexExchange, Network, RpcProvider};
use meta_dex::DexService;
use meta_util::ether::tx_hash_from_str;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    let network = Network::ARBI;
    let rpc_provider = RpcProvider::Quick;
    let dex = DexExchange::UniswapV3;
    let usdc = Token::USDC;
    let arb = Token::ARB;
    let weth = Token::WETH;
    let usdc_token_info = get_token_info(usdc, network).unwrap();
    let arb_token_info = get_token_info(arb, network).unwrap();
    let _weth_token_info = get_token_info(weth, network).unwrap();

    let swap_router_v2 = ContractType::UniV3SwapRouterV2;

    let rpc_info = get_rpc_info(network).unwrap();

    let V3_FEE = 500;

    println!("token_info {:?}", usdc_token_info);

    let rpc_url = rpc_info.ws_urls.get(&rpc_provider).unwrap();
    println!("rpc_url {:?}", rpc_url);
    let provider_ws = Provider::<Ws>::connect(rpc_url).await.expect("ws connect error");
    // let provider_ws = Provider::<Http>::connect(&rpc_info.httpUrls[0]).await;
    let provider_ws = provider_ws.interval(Duration::from_millis(200));
    let provider_ws = Arc::new(provider_ws);

    let receipt = provider_ws.get_transaction_receipt(tx_hash_from_str("0x0d23c9ba5ff75f9200995bf795a9fe424d8a498e43af5e6794359aeceab91af2")).await.unwrap().unwrap();
    println!("receipt: {:?}", receipt.status);
}