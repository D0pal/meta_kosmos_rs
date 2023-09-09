use chrono::Utc;
use ethers::prelude::*;
use meta_address::{enums::Asset, get_rpc_info, get_token_info, Token};
use meta_bots::venus::{notify_arbitrage_result, ArbitragePair, CexTradeInfo, DexTradeInfo};
use meta_cefi::bitfinex::wallet::TradeExecutionUpdate;
use meta_common::enums::{CexExchange, ContractType, DexExchange, Network, RpcProvider};
use meta_dex::DexService;
use meta_integration::Lark;
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

    let _swap_router_v2 = ContractType::UniV3SwapRouterV2;

    let rpc_info = get_rpc_info(network).unwrap();

    const V3_FEE: u32 = 500;

    println!("token_info {:?}", usdc_token_info);

    let rpc_url = rpc_info.ws_urls.get(&rpc_provider).unwrap();
    println!("rpc_url {:?}", rpc_url);
    let provider_ws = Provider::<Ws>::connect(rpc_url).await.expect("ws connect error");
    let provider_ws = provider_ws.interval(Duration::from_millis(200));
    let provider_ws = Arc::new(provider_ws);

    let dex_service = DexService::new(provider_ws.clone(), network, dex);
    let web_hook =
        "https://open.larksuite.com/open-apis/bot/v2/hook/722d27f3-fa80-4c79-8cf5-87970ce1712a";
    let lark = Lark::new(web_hook.to_string());

    let pair = ArbitragePair {
        datetime: Utc::now(),
        base: Asset::ARB,
        quote: Asset::USD,
        cex: CexTradeInfo {
            venue: CexExchange::BITFINEX,
            trade_info: Some(TradeExecutionUpdate {
                id: 123456u64, // Trade database id
                symbol: "tARBUSD".to_string(),
                mts_create: 123456u64, // Client Order ID
                order_id: 123456u64,   // Order id

                exec_amount: Decimal::from_f64(-12.0).unwrap(), // Positive means buy, negative means sell
                exec_price: Decimal::from_f64(0.89).unwrap(),   // Execution price
                order_type: "MARKET".to_string(),
                order_price: Decimal::from_f64(0.89).unwrap(),

                maker: -1,                                    // 1 if true, -1 if false
                fee: Some(Decimal::from_f64(0.001).unwrap()), // Fee ('tu' only)
                fee_currency: Some("ARB".to_string()),        // Fee currency ('tu' only)
                cid: 12345678u64,                             // client order id
            }),
        },
        dex: DexTradeInfo {
            network,
            venue: dex,
            tx_hash: Some(tx_hash_from_str(
                "0xcba0d4fc27a32aaddece248d469beb430e29c1e6fecdd5db3383e1c8b212cdeb",
            )),
            base_token_info: arb_token_info,
            quote_token_info: usdc_token_info,
            v3_fee: Some(V3_FEE),
        },
    };
    notify_arbitrage_result(&lark, &dex_service, 123456u128, &pair).await;
}
