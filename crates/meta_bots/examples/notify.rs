use chrono::Utc;
use ethers::prelude::*;
use foundry_evm::decode::decode_revert;
use futures::future::join_all;
use futures_util::future::try_join_all;
use gumdrop::Options;
use meta_address::{
    enums::Asset, get_bot_contract_info, get_dex_address, get_rpc_info, get_token_info,
    ContractInfo, Token, TokenInfo,
};
use meta_bots::{
    venus::{notify_arbitrage_result, ArbitragePair, CexTradeInfo, DexTradeInfo},
    AppConfig, VenusConfig,
};
use meta_cefi::{bitfinex::wallet::TradeExecutionUpdate, cefi_service::CefiService};
use meta_common::{
    enums::{BotType, CexExchange, ContractType, DexExchange, Network, RpcProvider},
    models::{CurrentSpread, MarcketChange},
};
use meta_contracts::{
    bindings::{
        erc20::ERC20,
        flash_bots_router::{FlashBotsRouter, UniswapWethParams},
        quoter_v2::QuoterV2,
        swap_router::SwapRouter,
        uniswap_v2_pair::{SwapFilter, UniswapV2PairEvents},
        ExactInputSingleParams, ExactOutputParams, ExactOutputSingleParams,
        QuoteExactInputSingleParams, QuoteExactOutputSingleParams, WETH9,
    },
    wrappers::{
        calculate_price_diff, get_atomic_arb_call_params, Erc20Wrapper, UniswapV2,
        UniswapV2PairWrapper,
    },
};
use meta_dex::DexService;
use meta_integration::Lark;
use meta_tracing::init_tracing;
use meta_util::{
    defi::{get_swap_price_limit, get_token0_and_token1},
    ether::{address_from_str, decimal_from_wei, decimal_to_wei, tx_hash_from_str},
    get_price_delta_in_bp,
    time::get_current_ts,
};
use rust_decimal::{
    prelude::{FromPrimitive, Signed},
    Decimal,
};
use serde::Deserialize;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    collections::{BinaryHeap, HashMap},
    io::BufReader,
    ops::Sub,
    path::PathBuf,
    rc::Rc,
    str::FromStr,
    sync::{
        atomic::{AtomicPtr, Ordering},
        mpsc, Arc, Mutex, RwLock as SyncRwLock,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument::WithSubscriber, warn, Level};

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
    let weth_token_info = get_token_info(weth, network).unwrap();

    let swap_router_v2 = ContractType::UniV3SwapRouterV2;

    let rpc_info = get_rpc_info(network).unwrap();

    let V3_FEE = 500;

    // let base_token_info = get_token_info(base_token, config.network).unwrap();
    // let quote_token_info = get_token_info(quote_token, config.network).unwrap();

    println!("token_info {:?}", usdc_token_info);

    let rpc_url = rpc_info.ws_urls.get(&rpc_provider).unwrap();
    println!("rpc_url {:?}", rpc_url);
    let provider_ws = Provider::<Ws>::connect(rpc_url).await.expect("ws connect error");
    // let provider_ws = Provider::<Http>::connect(&rpc_info.httpUrls[0]).await;
    let provider_ws = provider_ws.interval(Duration::from_millis(200));
    let provider_ws = Arc::new(provider_ws);

    // let private_key = std::fs::read_to_string("/tmp/pk").unwrap().trim().to_string();
    // let wallet: LocalWallet =
    //     private_key.parse::<LocalWallet>().unwrap().with_chain_id(rpc_info.chain_id);
    // let wallet_address = wallet.address();
    // let wallet = SignerMiddleware::new(provider_ws.clone(), wallet);
    // let wallet = NonceManagerMiddleware::new(wallet, wallet_address);
    // let wallet = Arc::new(wallet);

    // let swap_router_contract_info = get_dex_address(dex, network, swap_router_v2).unwrap();
    // println!("router_address {:?}", swap_router_contract_info.address);

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
            network: network,
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
