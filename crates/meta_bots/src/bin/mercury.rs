use ethers::prelude::*;
use futures::future::join_all;
use gumdrop::Options;
use serde::Deserialize;
use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{BinaryHeap, HashMap},
    io::BufReader,
    path::PathBuf,
    rc::Rc,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tracing::{debug, info, instrument::WithSubscriber, warn, Level};

use meta_address::{get_dex_address, get_token_address};
use meta_bots::AppConfig;
use meta_common::enums::{ContractType, Dex, Network, Token};
use meta_contracts::{
    bindings::{
        flash_bots_router::{FlashBotsRouter, UniswapWethParams},
        uniswap_v2_pair::{SwapFilter, UniswapV2PairEvents},
    },
    wrappers::{
        calculate_price_diff, get_atomic_arb_call_params, Erc20Wrapper, UniswapV2,
        UniswapV2PairWrapper,
    },
};
use meta_tracing::init_tracing;
use meta_util::address_from_str;

#[derive(Debug, Clone, Options)]
struct Opts {
    help: bool,

    #[options(help = "blockchain network, such as ETH, BSC")]
    network: Network,

    #[options(help = "base token, such as WETH, WBNB")]
    base_token: Token,

    #[options(help = "quote token, such as USDC, BUSD")]
    quote_token: Token,

    #[options(help = "dex a, such as PANCAKE")]
    dex_a: Dex,

    #[options(help = "dex a, such as BISWAP")]
    dex_b: Dex,

    #[options(help = "the Ethereum node endpoint (HTTP or WS)")]
    url: Option<String>,

    #[options(help = "path to your private key", default = "/tmp/pk")]
    private_key_path: PathBuf,

    #[options(help = "polling interval (ms)", default = "200")]
    interval: u64,

    #[options(default = "false", help = "use json as log format")]
    json_log: bool,

    #[options(help = "Instance name (used for logging)", default = "cliff")]
    instance_name: String,

    #[options(help = "only run one iteration and exit", default = "false")]
    one_shot: bool,
}

async fn run<P: JsonRpcClient + 'static + PubsubClient>(
    opts: Opts,
    provider: Provider<P>,
) -> anyhow::Result<()> {
    let provider = provider.interval(Duration::from_millis(opts.interval));

    info!("privatekey path {:?}", opts.private_key_path); // {:?} is explained in https://doc.rust-lang.org/std/fmt/index.html
    let private_key = std::fs::read_to_string(opts.private_key_path)
        .unwrap()
        .trim()
        .to_string();

    let wallet: LocalWallet = private_key.parse().unwrap();

    let wallet = wallet.with_chain_id(Chain::BinanceSmartChain);
    let liquidator_address = wallet.address();
    let client = SignerMiddleware::new(provider, wallet);
    let client = NonceManagerMiddleware::new(client, liquidator_address);
    let client = Arc::new(client);
    info!("profits will be sent to {:?}", liquidator_address);
    let quote_amt_in = u128::pow(10, 17);

    let quote_addr = get_token_address(opts.quote_token)
        .unwrap()
        .address(opts.network)
        .unwrap();
    let base_addr = get_token_address(opts.base_token)
        .unwrap()
        .address(opts.network)
        .unwrap();
    debug!("quote_addr: {}, base_addr: {} ", quote_addr, base_addr);

    let quote_asset = Erc20Wrapper::new(opts.network, quote_addr, client.clone()).await;
    let base_asset = Erc20Wrapper::new(opts.network, base_addr, client.clone()).await;

    let flashbots_router = FlashBotsRouter::new(
        address_from_str("0x53d19Fd4A403dCE6C8e4a1D0F3d0D12245813275"),
        client.clone(),
    );

    let market_a_factory_addr = get_dex_address(opts.dex_a.clone(), opts.network)
        .unwrap()
        .address(ContractType::UNI_V2_FACTORY)
        .unwrap();
    let market_a_swap_router_addr = get_dex_address(opts.dex_a.clone(), opts.network)
        .unwrap()
        .address(ContractType::UNI_V2_ROUTER)
        .unwrap();

    let market_a = UniswapV2::new(
        opts.network,
        opts.dex_a,
        market_a_factory_addr,
        market_a_swap_router_addr,
        client.clone(),
    );

    let market_b_factory_addr = get_dex_address(opts.dex_b.clone(), opts.network)
        .unwrap()
        .address(ContractType::UNI_V2_FACTORY)
        .unwrap();
    let market_b_swap_router_addr = get_dex_address(opts.dex_b.clone(), opts.network)
        .unwrap()
        .address(ContractType::UNI_V2_ROUTER)
        .unwrap();
    let biswap = UniswapV2::new(
        opts.network,
        opts.dex_b,
        market_b_factory_addr,
        market_b_swap_router_addr,
        client.clone(),
    );

    let market_a_pool_contract_wrapper = market_a
        .get_pair_contract_wrapper(
            quote_addr, // WBNB
            base_addr,  // BUSD
        )
        .await;
    let market_a_pool_contract_wrapper = Rc::new(RefCell::<
        UniswapV2PairWrapper<NonceManagerMiddleware<SignerMiddleware<Provider<P>, LocalWallet>>>,
    >::new(market_a_pool_contract_wrapper));

    let market_b_pool_contract_wrapper = biswap
        .get_pair_contract_wrapper(
            quote_addr, // WBNB
            base_addr,  // BUSD
        )
        .await;
    let market_b_pool_contract_wrapper = Rc::new(RefCell::<UniswapV2PairWrapper<_>>::new(
        market_b_pool_contract_wrapper,
    ));

    let last_block = client
        .get_block(BlockNumber::Latest)
        .await
        .expect("unable to get latest block")
        .unwrap()
        .number
        .unwrap();
    // let cake_swap_event_filter = pancake_pool_contract_wrapper.as_ref();
    let market_a_swap_event_filter = unsafe {
        market_a_pool_contract_wrapper
            .try_borrow_unguarded()
            .expect("borrow error")
            .pair_contract
            .event::<SwapFilter>()
            .from_block(last_block - 2)
    };

    let mut swap_events = market_a_swap_event_filter
        .subscribe()
        .await
        .unwrap()
        .with_meta();

    let mut last_block = 0;

    let mut last_ts = 0;
    loop {
        let current_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        debug!("current_ts {:?}", current_ts);
        if current_ts - last_ts > 2 {
            debug!("ping keep connection");
            client.client_version().await?;
            last_ts = current_ts;
        }

        match swap_events.next().await {
            Some(log) => {
                let (swap_log, meta) = log.unwrap() as (SwapFilter, LogMeta);
                // swap_log.

                debug!(
                    "block: {:?}, hash: {:?}, address: {:?}",
                    meta.block_number, meta.transaction_hash, meta.address
                );
                if meta.block_number.as_u64() > last_block {
                    last_block = meta.block_number.as_u64();
                    let price_diff = calculate_price_diff(
                        market_a_pool_contract_wrapper.clone(),
                        market_b_pool_contract_wrapper.clone(),
                        last_block,
                    )
                    .await;
                    info!("{:?} {:?}", last_block, price_diff);
                    if price_diff.abs() > 40f64 {
                        let params = get_atomic_arb_call_params(
                            market_a_pool_contract_wrapper.clone(),
                            market_b_pool_contract_wrapper.clone(),
                            price_diff,
                            &quote_asset,
                            quote_amt_in,
                            &base_asset,
                            flashbots_router.address(),
                        )
                        .await;
                        let contract_call = flashbots_router.uniswap_weth(params, false);
                        let tx = contract_call.send().await;
                        match tx {
                            Ok(tx) => {
                                println!("success: {:?}", tx)
                            }
                            Err(err) => {
                                println!("err: {:?}", err)
                            }
                        }
                    }
                }
            }
            None => {}
        }
    }
}

async fn main_impl() -> anyhow::Result<()> {
    let opts = Opts::parse_args_default_or_exit();
    let app_config = AppConfig::try_new().expect("parsing config error");

    let guard = init_tracing(app_config.log.into());

    info!(
        "run bot with arguments, chain: {} base_token: {}, quote_token: {},  ws provider url: {:?}",
        opts.network, opts.base_token, opts.quote_token, opts.url
    );

    if opts.url == None {
        panic!("Must provider url");
    }
    let provider = Provider::<Ws>::connect(opts.url.clone().unwrap())
        .await
        .expect("ws connect error");

    run(opts, provider).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    match main_impl().await {
        Ok(_) => {
            std::process::exit(exitcode::OK);
        }
        Err(e) => {
            eprintln!("run Error: {}", e);
            std::process::exit(exitcode::DATAERR);
        }
    }
}
