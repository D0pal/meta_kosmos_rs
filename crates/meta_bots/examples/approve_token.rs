use ethers::prelude::*;
use futures::future::join_all;
use futures_util::future::try_join_all;
use gumdrop::Options;
use meta_address::enums::Asset;
use meta_address::get_rpc_info;
use meta_address::{get_bot_contract_info, get_dex_address, get_token_info, Token};
use meta_bots::{AppConfig, VenusConfig};
use meta_cefi::cefi_service::CefiService;
use meta_common::enums::{Network, RpcProvider};
use meta_common::{
    enums::{BotType, CexExchange, ContractType, DexExchange},
    models::{CurrentSpread, MarcketChange},
};
use meta_contracts::bindings::{
    ExactInputSingleParams, ExactOutputParams, ExactOutputSingleParams,
};
use meta_contracts::{
    bindings::{
        flash_bots_router::{FlashBotsRouter, UniswapWethParams},
        quoter_v2::QuoterV2,
        swap_router::SwapRouter,
        uniswap_v2_pair::{SwapFilter, UniswapV2PairEvents},
        QuoteExactInputSingleParams, QuoteExactOutputSingleParams,
        erc20::ERC20
    },
    wrappers::{
        calculate_price_diff, get_atomic_arb_call_params, Erc20Wrapper, UniswapV2,
        UniswapV2PairWrapper,
    },
};
use meta_dex::enums::TokenInfo;
use meta_tracing::init_tracing;
use meta_util::defi::get_token0_and_token1;
use meta_util::ether::{address_from_str, decimal_from_wei, decimal_to_wei};
use meta_util::get_price_delta_in_bp;
use meta_util::time::get_current_ts;
use rust_decimal::{
    prelude::{FromPrimitive, Signed},
    Decimal,
};
use serde::Deserialize;
use std::ops::Sub;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    collections::{BinaryHeap, HashMap},
    io::BufReader,
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
    let token = Token::USDC;
    let reeiver = ContractType::UniV3SwapRouterV2;

    let rpc_info = get_rpc_info(network).unwrap();
    let token_info = get_token_info(token, network).unwrap();

    println!("token_info {:?}", token_info);

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

    let router = get_dex_address(dex, network, reeiver).unwrap();
    println!("router_address {:?}", router.address);
    let token = ERC20::new(token_info.address, wallet.clone());

    let call = token.approve(router.address, decimal_to_wei(Decimal::from_str_radix("10000", 10).unwrap(), token_info.decimals.into()));

    let start = tokio::time::Instant::now();
    let tx = call.send().await;
    let elapsed = tokio::time::Instant::now().duration_since(start).as_millis();
    println!("tx {:?}, total spent {:?} ms", tx, elapsed);
}
