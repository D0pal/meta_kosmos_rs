//! sandwidtch mev bot

use ethers::prelude::k256::pkcs8::der::oid::Error;
use ethers::prelude::*;
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

use meta_address::{get_bot_contract_info, get_dex_address, get_rpc_info, get_token_address,Token};
use meta_bots::{mev_bots::sandwidth::BotSandwidth, JupyterConfig};
use meta_common::enums::{BotType, ContractType, DexExchange, Network, };
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
use meta_dex::{sync_dex, Dex};
use meta_tracing::init_tracing;
use meta_util::{address_from_str, enums::dexs_from_str};

#[derive(Debug, Clone, Options)]
struct Opts {
    help: bool,

    #[options(help = "blockchain network, such as ETH, BSC")]
    network: Option<Network>,

    #[options(help = "comma separated dexs, such as PANCAKE,UNISWAP_V2")]
    dexs: String,

    #[options(help = "path to your private key", default = "/tmp/pk/jupyter")]
    private_key_path: PathBuf,
}

async fn run(config: JupyterConfig) -> anyhow::Result<()> {
    info!("run jupyter app with config: {:?}", config);
    let rpc_info = get_rpc_info(config.chain.network.unwrap()).unwrap();
    debug!("rpc info {:?}", rpc_info);

    let provider_ws =
        Provider::<Ws>::connect(rpc_info.wsUrls[0].clone()).await.expect("ws connect error");
    // let provider_ws = Provider::<Http>::connect(&rpc_info.httpUrls[0]).await;
    let provider_ws =
        provider_ws.interval(Duration::from_millis(config.provider.ws_interval_milli.unwrap()));
    let provider_ws = Arc::new(provider_ws);

    let private_key = std::fs::read_to_string(config.accounts.private_key_path.unwrap())
        .unwrap()
        .trim()
        .to_string();
    let wallet_local: Arc<LocalWallet> =
        Arc::new(private_key.parse::<LocalWallet>().unwrap().with_chain_id(rpc_info.chainId));
    // let wallet_local = wallet_local;
    let searcher_address = wallet_local.address();
    // let wallet = SignerMiddleware::new(provider_ws.clone(), wallet_local.clone());
    // let wallet = NonceManagerMiddleware::new(wallet, searcher_address);
    // let wallet = Arc::new(wallet);

    info!("profits will be sent to {:?}", searcher_address);

    let network = config.chain.network.unwrap();
    let dexes = config
        .chain
        .dexs
        .unwrap()
        .into_iter()
        .map(|d| Arc::new(Dex::new(provider_ws.clone(), network, d)))
        .collect::<Vec<_>>();

    let current_block = provider_ws.get_block_number().await.unwrap();
    let pools = sync_dex(
        dexes.clone(),
        Some(BlockNumber::Number(current_block - 1000)),
        BlockNumber::Number(current_block),
    )
    .await
    .unwrap();

    info!("total pools num: {:?}", pools.len());
    let sandwitdh_contract_info = get_bot_contract_info(BotType::SANDWIDTH_HUFF, network).unwrap();

    let weth_address = match network {
        Network::BSC => get_token_address(Token::WBNB, Network::BSC),
        _ => get_token_address(Token::WETH, network),
    };

    // Execution loop (reconnect bot if it dies)
    // loop {
    //     // let client = utils::create_websocket_client().await.unwrap();
    let mut bot = BotSandwidth::new(
        network,
        sandwitdh_contract_info.address,
        sandwitdh_contract_info.created_blk_num.into(),
        weth_address.unwrap(),
        dexes.clone(),
        pools,
        provider_ws.clone(),
        wallet_local.clone(),
    )
    .await
    .unwrap();
    //         .await
    //         .unwrap();

    bot.run().await.unwrap();
    //     // log::error!("Websocket disconnected");
    // }
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
    if app_config.accounts.private_key_path.is_none() {
        app_config.accounts.private_key_path = Some(opts.private_key_path);
    }
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
