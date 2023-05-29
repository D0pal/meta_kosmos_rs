//! sandwidtch mev bot
//! 
//! use ethers::prelude::*;
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
    network: Network,

    #[options(help = "comma separated dexs, such as PANCAKE,UNISWAP_V2")]
    dexs: String,

    #[options(help = "path to your private key", default = "/tmp/pk")]
    private_key_path: PathBuf,
}

async fn run(
    opts: Opts,
) -> anyhow::Result<()> {
    // let rpc_info = get_rpc_info(opts.network).unwrap();
   Ok(())
}

async fn main_impl() -> anyhow::Result<()> {
    let opts = Opts::parse_args_default_or_exit();
    println!("opts: {:?}", opts);
    let dex = dexs_from_str(opts.dexs.clone());
    println!("{:?}", dex);
    let app_config = JupyterConfig::try_new().expect("parsing config error");

    println!("{:?}", app_config);
    let guard = init_tracing(app_config.log.into());

    run(opts).await
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
