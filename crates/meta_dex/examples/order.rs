use ethers::prelude::*;
// use foundry_evm::decode::decode_revert;
// use futures::future::join_all;
// use futures_util::future::try_join_all;
// use gumdrop::Options;
use meta_address::{
    enums::Asset, get_bot_contract_info, get_dex_address, get_rpc_info, get_token_info,
    ContractInfo, Token, TokenInfo,
};
// use meta_bots::{AppConfig, VenusConfig};
// use meta_cefi::cefi_service::CefiService;
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
use meta_dex::{enums::to_token_info, DexService};
// use meta_tracing::init_tracing;
use meta_util::{
    defi::{get_swap_price_limit, get_token0_and_token1},
    ether::{address_from_str, decimal_from_wei, decimal_to_wei},
    get_price_delta_in_bp,
    time::get_current_ts,
};
use rust_decimal::{
    prelude::{FromPrimitive, Signed},
    Decimal,
};
// use serde::Deserialize;
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

    println!("token_info {:?}", usdc_token_info);

    let rpc_url = rpc_info.ws_urls.get(&rpc_provider).unwrap();
    println!("rpc_url {:?}", rpc_url);
    let provider_ws = Provider::<Ws>::connect(rpc_url).await.expect("ws connect error");
    // let provider_ws = Provider::<Http>::connect(&rpc_info.httpUrls[0]).await;
    let provider_ws = provider_ws.interval(Duration::from_millis(200));
    let provider_ws = Arc::new(provider_ws);

    let private_key = std::fs::read_to_string("/tmp/pk").unwrap().trim().to_string();
    let wallet: LocalWallet =
        private_key.parse::<LocalWallet>().unwrap().with_chain_id(rpc_info.chain_id);
    let wallet_address = wallet.address();
    let wallet = SignerMiddleware::new(provider_ws.clone(), wallet);
    let wallet = NonceManagerMiddleware::new(wallet, wallet_address);
    let wallet = Arc::new(wallet);

    let swap_router_contract_info = get_dex_address(dex, network, swap_router_v2).unwrap();
    println!("router_address {:?}", swap_router_contract_info.address);

    let dex_service = DexService::new(wallet.clone(), network, dex);
    println!("dex_service.factory_creation_block {:?}", dex_service.factory_creation_block);

    let base_token_info = to_token_info(arb_token_info, network, arb);
    let quote_token_info = to_token_info(usdc_token_info, network, usdc);
    let ret = dex_service
        .submit_order(
            base_token_info,
            quote_token_info,
            Decimal::from_f64(-1.2).unwrap(),
            V3_FEE,
            wallet_address,
        )
        .await;
    println!("ret {:?}", ret);
}
