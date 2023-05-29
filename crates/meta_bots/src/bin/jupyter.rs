//! sandwidtch mev bot
//! 
//! use ethers::prelude::*;
use ethers::prelude::k256::pkcs8::der::oid::Error;
use futures::future::join_all;
use gumdrop::Options;
use serde::{Deserialize, __private::de};
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

use meta_address::{get_dex_address, get_token_address, get_rpc_info, get_bot_address};
use meta_bots::JupyterConfig;
use meta_common::enums::{ContractType, Dex, Network, Token, Bot};
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
use meta_util::{address_from_str, enums::dexs_from_str};

#[derive(Debug, Clone, Options)]
struct Opts {
    help: bool,

    #[options(help = "blockchain network, such as ETH, BSC")]
    network: Option<Network>,

    #[options(help = "comma separated dexs, such as PANCAKE,UNISWAP_V2")]
    dexs: String,

    #[options(help = "path to your private key", default = "/tmp/pk")]
    private_key_path: PathBuf,
}

async fn run(
    config: JupyterConfig,
) -> anyhow::Result<()> {
    info!("run jupyter app with config: {:?}", config);
    let rpc_info = get_rpc_info(config.chain.network.unwrap()).unwrap();
    // info!(
    //     "run bot with arguments, chain: {} base_token: {}, quote_token: {},  ws provider url: {:?}",
    //     opts.network, opts.base_token, opts.quote_token, rpc_info.wsUrls[0]
    // );

    // let provider = Provider::<Ws>::connect(rpc_info.wsUrls[0].clone())
    //     .await
    //     .expect("ws connect error");
    // let provider = provider.interval(Duration::from_millis(opts.interval));

    // info!("privatekey path {:?}", opts.private_key_path); // {:?} is explained in https://doc.rust-lang.org/std/fmt/index.html
    // let private_key = std::fs::read_to_string(opts.private_key_path)
    //     .unwrap()
    //     .trim()
    //     .to_string();
    // let wallet: LocalWallet = private_key.parse().unwrap();

    // let wallet = wallet.with_chain_id(rpc_info.chainId);
    // let executor_address = wallet.address();
    // let client = SignerMiddleware::new(provider, wallet);
    // let client = NonceManagerMiddleware::new(client, executor_address);
    // let client = Arc::new(client);
    // info!("profits will be sent to {:?}", executor_address);
   Ok(())
}

async fn main_impl() -> anyhow::Result<()> {
    let opts = Opts::parse_args_default_or_exit();
    println!("opts: {:?}", opts);
    if opts.network.is_none() {
        panic!("must provide network");
    }
    let dex = dexs_from_str(opts.dexs.clone());
    if dex.is_empty() {
        panic!("must provide dex list");
    }
    let mut app_config = JupyterConfig::try_new().expect("parsing config error");
    app_config.chain.network = opts.network;
    app_config.chain.dexs = Some(dex);
    let guard = init_tracing(app_config.log.clone().into());

    run(app_config).await
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
