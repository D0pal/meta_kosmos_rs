//! cex dex arbitrage bot
use chrono::prelude::*;
use ethers::prelude::*;
use futures::future::join_all;
use futures_util::future::try_join_all;
use gumdrop::Options;
use meta_address::{
    enums::Asset, get_bot_contract_info, get_dex_address, get_rpc_info, get_token_info, Token,
    TokenInfo,
};
use meta_bots::{
    venus::{
        ArbitrageInstruction, ArbitragePair, CexInstruction, CexTradeInfo, DexInstruction,
        DexTradeInfo, CID,
    },
    AppConfig, VenusConfig,
};
use meta_cefi::{
    bitfinex::wallet::{OrderUpdateEvent, TradeExecutionUpdate},
    cefi_service::{AccessKey, CefiService, CexConfig},
};
use meta_common::{
    enums::{BotType, CexExchange, ContractType, DexExchange, Network},
    models::{CurrentSpread, MarcketChange},
};
use meta_contracts::{
    bindings::{
        flash_bots_router::{FlashBotsRouter, UniswapWethParams},
        quoter_v2::QuoterV2,
        swap_router::SwapRouter,
        uniswap_v2_pair::{SwapFilter, UniswapV2PairEvents},
        ExactInputSingleParams, ExactOutputParams, ExactOutputSingleParams,
        QuoteExactInputSingleParams, QuoteExactOutputSingleParams,
    },
    wrappers::{
        calculate_price_diff, get_atomic_arb_call_params, Erc20Wrapper, UniswapV2,
        UniswapV2PairWrapper,
    },
};
use meta_dex::DexService;
use meta_integration::Lark;
use meta_model::{ArbitrageOutcome, ArbitrageSummary};
use meta_tracing::init_tracing;
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
use serde::Deserialize;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    collections::{BTreeMap, BinaryHeap, HashMap},
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
use uuid::Uuid;

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

pub const V3_FEE: u32 = 500u32;

async fn run(config: VenusConfig) -> anyhow::Result<()> {
    debug!("run venus app with config: {:?}", config);
    let rpc_info = get_rpc_info(config.network).unwrap();

    let rpc_provider = config.provider.provider.expect("need rpc provider");
    let rpc_url = rpc_info.ws_urls.get(&rpc_provider).unwrap();
    info!("rpc_url {:?}", rpc_url);
    let provider_ws = Provider::<Ws>::connect(rpc_url).await.expect("ws connect error");
    // let provider_ws = Provider::<Http>::connect(&rpc_info.httpUrls[0]).await;
    let provider_ws =
        provider_ws.interval(Duration::from_millis(config.provider.ws_interval_milli.unwrap()));
    let provider_ws = Arc::new(provider_ws);

    let private_key = std::fs::read_to_string(config.account.private_key_path.unwrap())
        .unwrap()
        .trim()
        .to_string();
    let wallet: LocalWallet =
        private_key.parse::<LocalWallet>().unwrap().with_chain_id(rpc_info.chain_id);
    let wallet_address = wallet.address();
    let wallet = SignerMiddleware::new(provider_ws.clone(), wallet);
    let wallet = NonceManagerMiddleware::new(wallet, wallet_address);
    let wallet = Arc::new(wallet);
    let dex_service = DexService::new(wallet.clone(), config.network, config.dex);

    let mut arbitrages: Arc<SyncRwLock<BTreeMap<CID, ArbitragePair>>> =
        Arc::new(SyncRwLock::new(BTreeMap::new())); // key is request id

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
                unwrap_to: None,
                byte_code: None,
                code_hash: None,
                native: false,
            };
            let quote_token: TokenInfo = TokenInfo {
                token: quote_token,
                decimals: quote_token_info.decimals,
                network: config.network,
                address: quote_token_address,
                unwrap_to: None,
                byte_code: None,
                native: false,
                code_hash: None,
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
                    // for receiving spread update
                    let (tx, mut rx) = mpsc::sync_channel::<MarcketChange>(1000);
                    let (tx_order, mut rx_order) = mpsc::sync_channel::<TradeExecutionUpdate>(100);

                    let mut map = BTreeMap::new();
                    let ak = config.bitfinex.unwrap();
                    map.insert(
                        CexExchange::BITFINEX,
                        AccessKey {
                            api_key: ak.api_key.to_string(),
                            api_secret: ak.api_secret.to_string(),
                        },
                    );
                    let cex_config = CexConfig { keys: Some(map) };
                    let mut cefi_service = CefiService::new(
                        Some(cex_config),
                        Some(tx.clone()),
                        Some(tx_order.clone()),
                    );

                    let cefi_service = &mut cefi_service as *mut CefiService;
                    let cefi_service = Arc::new(AtomicPtr::new(cefi_service));

                    // receive cex trade execution info and update to arbitrages
                    {
                        let arbitrages_clone = arbitrages.clone();
                        thread::spawn(move || loop {
                            let ou_event = rx_order.recv();
                            if let Ok(trade) = ou_event {
                                if let Ok(mut _g) = arbitrages_clone.write() {
                                    _g.entry(trade.cid.into()).and_modify(|e| {
                                        (*e).cex.trade_info = Some(trade);
                                    });
                                } else {
                                    error!("unable to get arbitrages write lock");
                                }
                            }
                        });
                    }

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
                                                                    fee: V3_FEE,
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
                                                                    fee: V3_FEE,
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

                            info!("current spread, cex_bid: {:?}, dex_ask: {:?}, dex_bid: {:?}, cex_ask {:?}", cex_bid, dex_ask, dex_bid, cex_ask);
                            if cex_bid > dex_ask {
                                let change = get_price_delta_in_bp(cex_bid, dex_ask);
                                if change > Decimal::from_u32(config.spread_diff_threshold).unwrap()
                                {
                                    info!(
                                        "found a cross, cex bid {:?}, dex ask {:?}, price change {:?}",
                                        cex_bid, dex_ask, change
                                    );
                                    let mut amount = config.base_asset_quote_amt.clone();
                                    amount.set_sign_negative(true);
                                    let instraction = ArbitrageInstruction {
                                        cex: CexInstruction {
                                            venue: CexExchange::BITFINEX,
                                            amount: amount,
                                            base_asset: config.base_asset,
                                            quote_asset: config.quote_asset,
                                        },
                                        dex: DexInstruction {
                                            network: config.network,
                                            venue: DexExchange::UniswapV3,
                                            amount: config.base_asset_quote_amt,
                                            base_token: base_token.clone(),
                                            quote_token: quote_token.clone(),
                                            recipient: wallet_address,
                                            fee: V3_FEE,
                                        },
                                    };

                                    try_arbitrage(
                                        arbitrages.clone(),
                                        instraction,
                                        cefi_service.clone().load(Ordering::Relaxed),
                                        &dex_service,
                                    )
                                    .await;
                                }
                            }

                            if dex_bid > cex_ask {
                                let change = get_price_delta_in_bp(dex_bid, cex_ask);
                                if change > Decimal::from_u32(config.spread_diff_threshold).unwrap()
                                {
                                    // sell dex, buy cex
                                    info!(
                                        "found a cross, dex bid {:?}, cex ask {:?}, price change {:?}",
                                        dex_bid, cex_ask, change
                                    );
                                    let mut amount = config.base_asset_quote_amt.clone();
                                    amount.set_sign_negative(true);
                                    let instraction = ArbitrageInstruction {
                                        cex: CexInstruction {
                                            venue: CexExchange::BITFINEX,
                                            amount: config.base_asset_quote_amt,
                                            base_asset: config.base_asset,
                                            quote_asset: config.quote_asset,
                                        },
                                        dex: DexInstruction {
                                            network: config.network,
                                            venue: DexExchange::UniswapV3,
                                            amount: amount,
                                            base_token: base_token.clone(),
                                            quote_token: quote_token.clone(),
                                            recipient: wallet_address,
                                            fee: V3_FEE,
                                        },
                                    };

                                    try_arbitrage(
                                        arbitrages.clone(),
                                        instraction,
                                        cefi_service.clone().load(Ordering::Relaxed),
                                        &dex_service,
                                    )
                                    .await;
                                }
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

async fn try_arbitrage<'a, M: Middleware>(
    arbitrages: Arc<SyncRwLock<BTreeMap<CID, ArbitragePair>>>,
    instruction: ArbitrageInstruction,
    cefi_service_ptr: *mut CefiService,
    dex_service_ref: &DexService<M>,
) {
    let client_order_id = get_current_ts().as_millis();

    info!("start arbitrage with instruction {:?}", instruction);
    info!(
        "start send cex trade, venue {:?}, base_asset {:?}, quote_asset {:?}, amount {:?}",
        instruction.cex.venue,
        instruction.cex.base_asset,
        instruction.cex.quote_asset,
        instruction.cex.amount
    );

    let mut _g = arbitrages.write().expect("unable to get write lock");

    _g.insert(
        client_order_id,
        ArbitragePair {
            datetime: chrono::Utc::now(),
            base: instruction.cex.base_asset,
            quote: instruction.cex.quote_asset,
            cex: CexTradeInfo { venue: instruction.cex.venue, trade_info: None },
            dex: DexTradeInfo {
                network: instruction.dex.network,
                venue: instruction.dex.venue,
                tx_hash: None,
                base_token_info: instruction.dex.base_token.clone(),
                quote_token_info: instruction.dex.quote_token.clone(),
                v3_fee: Some(instruction.dex.fee),
            },
        },
    );

    match instruction.cex.venue {
        CexExchange::BITFINEX => unsafe {
            (*cefi_service_ptr).submit_order(
                client_order_id,
                CexExchange::BITFINEX,
                instruction.cex.base_asset,
                instruction.cex.quote_asset,
                instruction.cex.amount,
            );
        },
        _ => unimplemented!(),
    }
    info!("end send cex trade");

    match instruction.dex.venue {
        DexExchange::UniswapV3 => {
            let ret = dex_service_ref
                .submit_order(
                    instruction.dex.base_token,
                    instruction.dex.quote_token,
                    instruction.dex.amount,
                    instruction.dex.fee,
                    instruction.dex.recipient,
                )
                .await;
            match ret {
                Ok(hash) => {
                    info!("send dex order success {:?}", hash);
                    _g.entry(client_order_id).and_modify(|e| {
                        (*e).dex.tx_hash = Some(hash);
                    });
                }
                Err(e) => error!("error in send dex order {:?}", e),
            }
        }
        _ => unimplemented!(),
    }
    info!("end send dex trade");
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

    app_config.log.file_name_prefix.push_str("_");
    app_config.log.file_name_prefix.push_str(&app_config.base_asset.to_string());
    app_config.log.file_name_prefix.push_str(&app_config.quote_asset.to_string());
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

#[cfg(test)]
mod test {
    use meta_util::ether::tx_hash_from_str;

    use super::*;

    #[test]
    fn test_check_arbitrage_status() {
        let mut map: BTreeMap<CID, ArbitragePair> = BTreeMap::new();
        map.insert(
            u128::from(100u32),
            ArbitragePair {
                base: Asset::ARB,
                quote: Asset::USD,
                cex: CexTradeInfo { venue: CexExchange::BITFINEX, trade_info: None },
                dex: DexTradeInfo {
                    network: Network::ARBI,
                    venue: DexExchange::UniswapV3,
                    tx_hash: None,
                    base_token_info: TokenInfo {
                        token: Token::ARB,
                        decimals: 18,
                        network: Network::ARBI,
                        address: address_from_str("0x89dbEA2B8c120a60C086a5A7f73cF58261Cb9c44"),
                    },
                    quote_token_info: TokenInfo {
                        token: Token::USD,
                        decimals: 6,
                        network: Network::ARBI,
                        address: address_from_str("0x89dbEA2B8c120a60C086a5A7f73cF58261Cb9c44"),
                    },
                    v3_fee: None,
                },
            },
        );
        map.insert(
            u128::from(200u32),
            ArbitragePair {
                base: Asset::ARB,
                quote: Asset::USD,
                cex: CexTradeInfo {
                    venue: CexExchange::BITFINEX,
                    trade_info: Some(TradeExecutionUpdate::default()),
                },
                dex: DexTradeInfo {
                    network: Network::ARBI,
                    venue: DexExchange::UniswapV3,
                    tx_hash: None,
                    base_token_info: TokenInfo {
                        token: Token::ARB,
                        decimals: 18,
                        network: Network::ARBI,
                        address: address_from_str("0x89dbEA2B8c120a60C086a5A7f73cF58261Cb9c44"),
                    },
                    quote_token_info: TokenInfo {
                        token: Token::USD,
                        decimals: 6,
                        network: Network::ARBI,
                        address: address_from_str("0x89dbEA2B8c120a60C086a5A7f73cF58261Cb9c44"),
                    },
                    v3_fee: None,
                },
            },
        );
        let map = Arc::new(std::sync::RwLock::new(map));
        let output = check_arbitrage_status(map.clone());
        assert!(output.is_none());

        {
            let mut _g = map.write().unwrap();
            _g.entry(u128::from(200u32)).and_modify(|e| {
                (*e).dex.tx_hash = Some(tx_hash_from_str(
                    "0xcba0d4fc27a32aaddece248d469beb430e29c1e6fecdd5db3383e1c8b212cdeb",
                ))
            });
        }
        let output = check_arbitrage_status(map.clone());
        assert!(output.is_some());
        assert_eq!(output.unwrap().0, u128::from(200u32));
    }
}
