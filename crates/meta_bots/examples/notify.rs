use chrono::Utc;
use ethers::prelude::*;
use meta_address::{enums::Asset, get_rpc_info, get_token_info, Token};
use meta_bots::venus::{ArbitragePair, CexTradeInfo, DexTradeInfo};
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

    let _dex_service = DexService::new(provider_ws.clone(), network, dex);
    let web_hook =
        "https://open.larksuite.com/open-apis/bot/v2/hook/722d27f3-fa80-4c79-8cf5-87970ce1712a";
    let _lark = Lark::new(web_hook.to_string());

    let cid = 1694323690468u64;
    let _pair = ArbitragePair {
        datetime: Utc::now(),
        base: Asset::ARB,
        quote: Asset::USD,
        // { id: 1415532545, symbol: "tARBUSD", mts_create: 1694323690714, order_id: 126110503315, exec_amount: 12, exec_price: 0.87011, order_type: "EXCHANGE MARKET", order_price: 0.87011, maker: -1, fee: Some(-0.0048), fee_currency: Some("ARB"), cid: 1694323690468 }
        cex: CexTradeInfo {
            venue: CexExchange::BITFINEX,
            trade_info: Some(TradeExecutionUpdate {
                id: 1415532545u64, // Trade database id
                symbol: "tARBUSD".to_string(),
                mts_create: 1694323690714u64, // Client Order ID
                order_id: 126110503315u64,    // Order id

                exec_amount: Decimal::from_f64(12.0).unwrap(), // Positive means buy, negative means sell
                exec_price: Decimal::from_f64(0.87011).unwrap(), // Execution price
                order_type: "MARKET".to_string(),
                order_price: Decimal::from_f64(0.87011).unwrap(),

                maker: -1,                                      // 1 if true, -1 if false
                fee: Some(Decimal::from_f64(-0.0048).unwrap()), // Fee ('tu' only)
                fee_currency: Some("ARB".to_string()),          // Fee currency ('tu' only)
                cid,                                            // client order id
            }),
        },
        dex: DexTradeInfo {
            network,
            venue: dex,
            tx_hash: Some(tx_hash_from_str(
                "0xd6fc2a4b5a4ca352f7e76c02f8c4609d3dbc57c2b95a1712334b09ed9dcb7f01",
            )),
            base_token_info: arb_token_info,
            quote_token_info: usdc_token_info,
            v3_fee: Some(V3_FEE),
        },
    };
    // notify_arbitrage_result(&lark, &dex_service, cid.into(), &pair).await;
}
