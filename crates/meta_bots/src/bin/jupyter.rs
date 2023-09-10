//! sandwidtch mev bot

use ethers::prelude::{*};

use gumdrop::Options;

use std::{
    path::PathBuf,
};



use meta_bots::{JupyterConfig};
use meta_common::enums::{Network};


use meta_tracing::init_tracing;
use meta_util::{enums::dexs_from_str};

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

async fn run(_config: JupyterConfig) -> anyhow::Result<()> {
    // info!("run jupyter app with config: {:?}", config);
    // let provider = config.chain.provider.expect("provider required");
    // let rpc_info = get_rpc_info(config.chain.network.unwrap()).unwrap();
    // debug!("rpc info {:?}", rpc_info);

    // let provider_ws = Provider::<Ws>::connect(rpc_info.ws_urls.get(&provider).unwrap().clone())
    //     .await
    //     .expect("ws connect error");
    // // let provider_ws = Provider::<Http>::connect(&rpc_info.httpUrls[0]).await;
    // let provider_ws =
    //     provider_ws.interval(Duration::from_millis(config.provider.ws_interval_milli.unwrap()));
    // let provider_ws = Arc::new(provider_ws);

    // let private_key = std::fs::read_to_string(config.accounts.private_key_path.unwrap())
    //     .unwrap()
    //     .trim()
    //     .to_string();
    // let wallet_local: Arc<LocalWallet> =
    //     Arc::new(private_key.parse::<LocalWallet>().unwrap().with_chain_id(rpc_info.chain_id));
    // // let wallet_local = wallet_local;
    // let searcher_address = wallet_local.address();
    // // let wallet = SignerMiddleware::new(provider_ws.clone(), wallet_local.clone());
    // // let wallet = NonceManagerMiddleware::new(wallet, searcher_address);
    // // let wallet = Arc::new(wallet);

    // info!("profits will be sent to {:?}", searcher_address);

    // let network = config.chain.network.unwrap();
    // let dexes = config
    //     .chain
    //     .dexs
    //     .unwrap()
    //     .into_iter()
    //     .map(|d| Arc::new(DexService::new(provider_ws.clone(), network, d)))
    //     .collect::<Vec<_>>();

    // let current_block = provider_ws.get_block_number().await.unwrap();
    // let pools = sync_dex(
    //     dexes.clone(),
    //     Some(BlockNumber::Number(current_block - 1000)),
    //     BlockNumber::Number(current_block),
    // )
    // .await
    // .unwrap();

    // info!("total pools num: {:?}", pools.len());
    // let sandwitdh_contract_info = get_bot_contract_info(BotType::SANDWIDTH_HUFF, network).unwrap();

    // let weth_address = match network {
    //     Network::BSC => {
    //         let info = get_token_info(Token::WBNB, Network::BSC).unwrap();
    //         info.address
    //     }
    //     _ => {
    //         let info = get_token_info(Token::WETH, network).unwrap();
    //         info.address
    //     }
    // };

    // // Execution loop (reconnect bot if it dies)
    // // loop {
    // //     // let client = utils::create_websocket_client().await.unwrap();
    // let mut bot = BotSandwidth::new(
    //     network,
    //     sandwitdh_contract_info.address,
    //     sandwitdh_contract_info.created_blk_num.into(),
    //     weth_address,
    //     dexes.clone(),
    //     pools,
    //     provider_ws.clone(),
    //     wallet_local.clone(),
    // )
    // .await
    // .unwrap();
    // //         .await
    // //         .unwrap();

    // bot.run().await.unwrap();
    // //     // log::error!("Websocket disconnected");
    // // }
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
    let _guard = init_tracing(app_config.log.clone().into());

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
