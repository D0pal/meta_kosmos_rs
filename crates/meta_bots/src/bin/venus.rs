//! cex dex arbitrage bot
use ethers::prelude::*;
use futures::future::join_all;
use futures_util::future::try_join_all;
use gumdrop::Options;
use meta_address::enums::Asset;
use meta_address::{get_bot_contract_info, get_dex_address, get_rpc_info, get_token_info, Token};
use meta_bots::{AppConfig, VenusConfig};
use meta_cefi::cefi_service::CefiService;
use meta_common::{
    enums::{BotType, CexExchange, ContractType, DexExchange, Network},
    models::{CurrentSpread, MarcketChange},
};
use meta_contracts::{
    bindings::{
        flash_bots_router::{FlashBotsRouter, UniswapWethParams},
        quoter_v2::QuoterV2,
        uniswap_v2_pair::{SwapFilter, UniswapV2PairEvents},
        QuoteExactInputSingleParams, QuoteExactOutputSingleParams,
    },
    wrappers::{
        calculate_price_diff, get_atomic_arb_call_params, Erc20Wrapper, UniswapV2,
        UniswapV2PairWrapper,
    },
};
use meta_dex::enums::TokenInfo;
use meta_tracing::init_tracing;
use meta_util::ether::{address_from_str, decimal_from_wei, decimal_to_wei};
use rust_decimal::Decimal;
use serde::Deserialize;
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

#[derive(Debug, Clone, Options)]
struct Opts {
    help: bool,

    #[options(help = "base token, such as USDT")]
    base_asset: Option<Asset>,

    #[options(help = "quote token, tokenIn, such as WBNB, BUSD")]
    quote_asset: Option<Asset>,

    #[options(help = "dex a, such as PANCAKE")]
    dex: Option<DexExchange>,

    #[options(help = "blockchain network, such as ETH, ARBI")]
    network: Option<Network>,

    #[options(help = "dex a, such as BISWAP")]
    cex: Option<CexExchange>,

    #[options(help = "path to your private key")]
    private_key_path: Option<PathBuf>,
}

async fn run(config: VenusConfig) -> anyhow::Result<()> {
    info!("run venus app with config: {:?}", config);
    let rpc_info = get_rpc_info(config.network).unwrap();
    debug!("rpc info {:?}", rpc_info);
    let rpc_provider = config.provider.provider.expect("need rpc provider");
    let provider_ws = Provider::<Ws>::connect(rpc_info.ws_urls.get(&rpc_provider).unwrap().clone())
        .await
        .expect("ws connect error");
    // let provider_ws = Provider::<Http>::connect(&rpc_info.httpUrls[0]).await;
    let provider_ws =
        provider_ws.interval(Duration::from_millis(config.provider.ws_interval_milli.unwrap()));
    let provider_ws = Arc::new(provider_ws);

    let private_key = std::fs::read_to_string(config.account.private_key_path.unwrap())
        .unwrap()
        .trim()
        .to_string();
    let wallet_local: Arc<LocalWallet> =
        Arc::new(private_key.parse::<LocalWallet>().unwrap().with_chain_id(rpc_info.chain_id));

    match config.dex {
        DexExchange::UniswapV3 => {
            let (base_token, quote_token) = (config.base_asset.into(), config.quote_asset.into());
            let base_token_info = get_token_info(base_token, config.network).unwrap();
            let quote_token_info = get_token_info(quote_token, config.network).unwrap();
            let (base_token_address, quote_token_address) =
                (base_token_info.address, quote_token_info.address);
            let base_token: TokenInfo = TokenInfo {
                token: base_token,
                decimals: base_token_info.decimals,
                network: config.network,
                address: base_token_address,
            };
            let quote_token: TokenInfo = TokenInfo {
                token: quote_token,
                decimals: quote_token_info.decimals,
                network: config.network,
                address: quote_token_address,
            };

            let quoter_address = get_dex_address(
                DexExchange::UniswapV3,
                config.network,
                ContractType::UniV3QuoterV2,
            )
            .unwrap();

            let quoter = QuoterV2::new(quoter_address.address, provider_ws.clone());

            match config.cex {
                CexExchange::BITFINEX => {
                    let (tx, mut rx) = mpsc::sync_channel::<MarcketChange>(1000);

                    let mut cefi_service = CefiService::new(None, Some(tx.clone()));

                    let cefi_service = &mut cefi_service as *mut CefiService;
                    let cefi_service = Arc::new(AtomicPtr::new(cefi_service));

                    {
                        let cefi_service = cefi_service.clone();
                        thread::spawn(move || {
                            let a = cefi_service.load(Ordering::Relaxed);
                            unsafe {
                                (*a).subscribe_book(
                                    CexExchange::BITFINEX,
                                    config.base_asset,
                                    config.quote_asset,
                                );
                            }
                        });
                    }

                    let (last_dex_sell_price, last_dex_buy_price) = (
                        Arc::new(RwLock::new(Decimal::default())),
                        Arc::new(RwLock::new(Decimal::default())),
                    );

                    {
                        let last_dex_sell_price = last_dex_sell_price.clone();
                        let last_dex_buy_price = last_dex_buy_price.clone();
                        tokio::spawn(async move {
                            let mut new_block_stream =
                                provider_ws.subscribe_blocks().await.unwrap();
                            let mut last_block: u64 = 0;

                            loop {
                                match new_block_stream.next().await {
                                    Some(block) => {
                                        if let Some(block_number) = block.number {
                                            if block_number.as_u64() > last_block {
                                                last_block = block_number.as_u64();

                                                // quote for 500, 3000, all pools
                                                let base_quote_amt_in_wei = decimal_to_wei(
                                                    config.base_asset_quote_amt,
                                                    base_token.decimals.into(),
                                                );
                                                debug!("amt_in_wei: {:?}, base_token_address: {:?}, quote_token_address: {:?}", base_quote_amt_in_wei, base_token_address, quote_token_address);
                                                let rets = try_join_all([
                                                        quoter
                                                            .quote_exact_input_single(
                                                                QuoteExactInputSingleParams {
                                                                    token_in: base_token.address,
                                                                    token_out: quote_token.address,
                                                                    amount_in: base_quote_amt_in_wei,
                                                                    fee: 500,
                                                                    sqrt_price_limit_x96: 0.into(),
                                                                },
                                                            )
                                                            .call(),
                                                        quoter
                                                            .quote_exact_output_single(
                                                                QuoteExactOutputSingleParams {
                                                                    token_in: quote_token.address,
                                                                    token_out: base_token.address,
                                                                    amount: base_quote_amt_in_wei,
                                                                    fee: 500,
                                                                    sqrt_price_limit_x96: U256::from_str_radix("1461446703485210103287273052203988822378723970341", 10).unwrap(),
                                                                },
                                                            )
                                                            .call(),
                                                    ])
                                                    .await;

                                                match rets {
                                                    Ok(ret) => {
                                                        let sell = ret[0];
                                                        let buy = ret[1];
                                                        let (amount_out, _, _, _) = sell;
                                                        let sell_price = decimal_from_wei(
                                                            amount_out,
                                                            quote_token.decimals.into(),
                                                        )
                                                        .checked_div(config.base_asset_quote_amt)
                                                        .unwrap();
                                                        let (amount_in, _, _, _) = buy;
                                                        let buy_price = decimal_from_wei(
                                                            amount_in,
                                                            quote_token.decimals.into(),
                                                        )
                                                        .checked_div(config.base_asset_quote_amt)
                                                        .unwrap();

                                                        if !sell_price.eq(&(*(last_dex_sell_price
                                                            .read()
                                                            .await)))
                                                            || !buy_price.eq(
                                                                &(*(last_dex_buy_price
                                                                    .read()
                                                                    .await)),
                                                            )
                                                        {
                                                            *(last_dex_sell_price.write().await) =
                                                                sell_price;
                                                            *(last_dex_buy_price.write().await) =
                                                                buy_price;

                                                            debug!("send dex price change, block number {:?}, sell price: {:?}, buy price: {:?} ",block.number, sell_price, buy_price);
                                                            let ret =
                                                                tx.clone().send(MarcketChange {
                                                                    cex: None,
                                                                    dex: Some(CurrentSpread {
                                                                        best_bid: sell_price,
                                                                        best_ask: buy_price,
                                                                    }),
                                                                });
                                                            match ret {
                                                                Ok(()) => debug!(
                                                                    "send price chagne success"
                                                                ),
                                                                Err(e) => {
                                                                    eprintln!("error in send dex price change, {:?}",e);
                                                                    panic!("error in send dex price change, {:?}",e);
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        error!("error in quote {:?}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    None => {
                                        debug!("no block info");
                                    }
                                }
                            }
                        });
                    }

                    loop {
                        if let Ok(change) = rx.recv() {
                            let (mut cex_bid, mut cex_ask, mut dex_bid, mut dex_ask) = (
                                Decimal::default(),
                                Decimal::default(),
                                Decimal::default(),
                                Decimal::default(),
                            );
                            if let Some(cex_spread) = change.cex {
                                let (current_dex_bid, current_dex_ask) = (
                                    *last_dex_sell_price.read().await,
                                    *last_dex_buy_price.read().await,
                                );
                                (cex_bid, cex_ask, dex_bid, dex_ask) = (
                                    cex_spread.best_bid,
                                    cex_spread.best_ask,
                                    current_dex_bid,
                                    current_dex_ask,
                                );
                            }
                            if let Some(dex_spread) = change.dex {
                                let a = cefi_service.clone().load(Ordering::Relaxed);
                                let ret = unsafe {
                                    (*a).get_spread(
                                        CexExchange::BITFINEX,
                                        config.base_asset,
                                        config.quote_asset,
                                    )
                                };
                                match ret {
                                    Some(cex_spread) => {
                                        (cex_bid, cex_ask, dex_bid, dex_ask) = (
                                            cex_spread.best_bid,
                                            cex_spread.best_ask,
                                            dex_spread.best_bid,
                                            dex_spread.best_ask,
                                        );
                                    }
                                    None => {}
                                }
                            }

                            if cex_bid > dex_ask {
                                debug!(
                                    "found a cross, cex bid {:?}, dex ask {:?}",
                                    cex_bid, dex_ask
                                );
                            }

                            if dex_bid > cex_bid {
                                debug!(
                                    "found a cross, dex bid {:?}, cex ask {:?}",
                                    dex_bid, cex_ask
                                );
                            }
                        }
                    }
                }
            }
        }
        _ => {
            todo!()
        }
    }
}

async fn main_impl() -> anyhow::Result<()> {
    let opts = Opts::parse_args_default_or_exit();
    println!("opts: {:?}", opts);

    let mut app_config = VenusConfig::try_new().expect("parsing config error");
    if let Some(network) = opts.network {
        app_config.network = network;
    }
    if let Some(dex) = opts.dex {
        app_config.dex = dex;
    }
    if let Some(cex) = opts.cex {
        app_config.cex = cex;
    }
    if let Some(asset) = opts.base_asset {
        app_config.base_asset = asset;
    }
    if let Some(asset) = opts.quote_asset {
        app_config.quote_asset = asset;
    }

    if let Some(pk_path) = opts.private_key_path {
        app_config.account.private_key_path = Some(pk_path);
    }
    let guard = init_tracing(app_config.log.clone().into());

    debug!("venus config: {:?}", app_config);
    run(app_config).await;
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
