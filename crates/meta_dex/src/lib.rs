#![allow(unused_imports)]

pub mod defi;
pub mod error;
pub mod oracle;
pub mod pool;
pub mod sandwidth;
pub mod prelude {
    pub use super::{error::*, oracle::*, pool::*, sandwidth::*};
}
use crate::prelude::Pool;
use defi::DexWrapper;
use error::OrderError;
use ethers::prelude::*;
use ethers::prelude::*;
use eyre::Result;
use futures::future::try_join_all;
use futures_util::{SinkExt, TryStreamExt};
use hashbrown::HashMap;
use meta_address::TokenInfo;
use meta_address::{get_dex_address, Token};
use meta_common::{
    enums::{ContractType, DexExchange, Network, PoolVariant},
    models::{CurrentSpread, MarcketChange},
};
use meta_contracts::bindings::{
    quoterv2::{QuoteExactInputSingleParams, QuoteExactOutputSingleParams, QuoterV2},
    swaprouter::{ExactInputSingleParams, ExactOutputSingleParams, SwapRouter},
    uniswapv2factory::UniswapV2Factory,
    uniswapv3pool::{SwapFilter, UniswapV3Pool},
};
use meta_util::{
    defi::{get_swap_price_limit, get_token0_and_token1},
    ether::{decimal_from_wei, decimal_from_wei_i256, decimal_to_wei},
    time::get_current_ts,
};
use rust_decimal::{
    prelude::{FromPrimitive, Signed, ToPrimitive},
    Decimal,
};
use std::sync::{mpsc::SyncSender, Arc};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug)]
pub struct DexService<M> {
    pub client: Arc<M>,
    pub network: Network,
    pub dex_exchange: DexExchange,
    pub pool_variant: PoolVariant,
    pub dex_contracts: DexWrapper<M>,
}

#[derive(Debug)]
pub struct FeeInfo {
    pub fee_token: Token,
    pub amount: Decimal,
}

#[derive(Debug)]
pub struct TradeBalanceDiff {
    pub trade: HashMap<Token, Decimal>,
    pub fee: FeeInfo,
}

impl<M: Middleware> DexService<M> {
    /// # Description
    /// Creates a new dex instance
    pub fn new(
        client: Arc<M>,
        provider: Arc<Provider<Ws>>,
        network: Network,
        dex_exchange: DexExchange,
        base_token: TokenInfo,
        base_token_quote_amt: Decimal,
        quote_token: TokenInfo,
        v3_fee: u32,
        sender_market_change: SyncSender<MarcketChange>,
    ) -> (Self, DexBackend<Provider<Ws>>) {
        let pool_variant: PoolVariant = match dex_exchange {
            DexExchange::UniswapV3 => PoolVariant::UniswapV3,
            _ => PoolVariant::UniswapV2,
        };

        let contracts = DexWrapper::new(client.clone(), network, dex_exchange);

        let backend = DexBackend::<Provider<Ws>>::new(
            provider,
            network,
            sender_market_change,
            base_token,
            base_token_quote_amt,
            quote_token,
            v3_fee,
        );
        (
            DexService {
                client: client.clone(),
                network,
                dex_exchange,
                pool_variant,
                dex_contracts: contracts,
            },
            backend,
        )
    }

    pub async fn get_pool_address(&mut self, _token_0: Address, _token_1: Address) -> Address {
        Address::default()
    }

    pub async fn analyze_v3_tx(
        &self,
        hash: TxHash,
        base_token_info: TokenInfo,
        quote_token_info: TokenInfo,
        fee: u32,
    ) -> Result<TradeBalanceDiff, OrderError<M>> {
        let receipt = self.client.get_transaction_receipt(hash).await;
        match receipt {
            Ok(r) => match r {
                Some(_r) => {
                    let (token_0, token_1) =
                        get_token0_and_token1(&base_token_info.address, &quote_token_info.address);
                    let (mut token_0_info, mut token_1_info) =
                        (base_token_info.clone(), quote_token_info.clone());
                    if token_0.eq(&quote_token_info.address) {
                        (token_0_info, token_1_info) = (quote_token_info, base_token_info);
                    }
                    let gas_used = _r.gas_used.unwrap();
                    let gas_price = _r.effective_gas_price.unwrap();
                    let gas_fee = gas_used.checked_mul(gas_price).unwrap();
                    let gas_fee = decimal_from_wei(gas_fee, 18);
                    let pool = self.dex_contracts.get_v3_pool(token_0, token_1, fee).await.unwrap();
                    let mut diff = HashMap::new();
                    for log in _r.logs {
                        if log.address.eq(&pool.address()) {
                            let decoded: SwapFilter = pool
                                .decode_event(SwapFilter::name().as_ref(), log.topics, log.data)
                                .unwrap();

                            diff.insert(
                                token_0_info.token,
                                decimal_from_wei_i256(
                                    decoded.amount_0.saturating_neg(),
                                    token_0_info.decimals.into(),
                                ),
                            );
                            diff.insert(
                                token_1_info.token,
                                decimal_from_wei_i256(
                                    decoded.amount_1.saturating_neg(),
                                    token_1_info.decimals.into(),
                                ),
                            );
                        }
                    }
                    Ok(TradeBalanceDiff {
                        trade: diff,
                        fee: FeeInfo {
                            fee_token: Token::ETH,
                            amount: gas_fee.saturating_mul(Decimal::NEGATIVE_ONE),
                        },
                    })
                }
                None => Err(OrderError::UnableFetchTxReceiptError),
            },
            Err(_e) => Err(OrderError::UnableFetchTxReceiptError),
        }
    }

    pub async fn submit_order(
        &self,
        base: TokenInfo,
        quote: TokenInfo,
        amount: Decimal,
        fee: u32,
        recipient: Address,
    ) -> Result<TxHash, OrderError<M>> {
        let ddl = get_current_ts().as_secs() + 1000000;

        match self.dex_exchange {
            DexExchange::UniswapV3 => {
                if amount.is_sign_negative() {
                    // sell base
                    let (token_in, token_out) = (base, quote);
                    let amount_in_wei = decimal_to_wei(amount.abs(), token_in.decimals.into());
                    let swap_param = ExactInputSingleParams {
                        token_in: token_in.address,
                        token_out: token_out.address,
                        fee,
                        recipient,
                        deadline: ddl.into(),
                        amount_in: amount_in_wei,
                        amount_out_minimum: U256::zero(),
                        sqrt_price_limit_x96: get_swap_price_limit(
                            token_in.address,
                            token_out.address,
                            token_in.address,
                        ),
                    };
                    info!("swap params {:?}", swap_param);

                    let call = self
                        .dex_contracts
                        .get_v3_swap_router()
                        .await
                        .as_ref()
                        .unwrap()
                        .exact_input_single(swap_param)
                        .gas(2_000_000)
                        .gas_price(300_000_000);
                    let ret = call.send().await;
                    match ret {
                        Ok(ref tx) => {
                            info!("send v3 exact input transaction {:?}", tx);
                            Ok(tx.tx_hash())
                        }
                        Err(e) => Err(OrderError::ContractError(e)),
                    }
                } else {
                    // buy base
                    let (token_in, token_out) = (quote, base);
                    let amount_out_wei = decimal_to_wei(amount, token_out.decimals.into());
                    let param_output = ExactOutputSingleParams {
                        token_in: token_in.address,
                        token_out: token_out.address,
                        fee,
                        recipient,
                        deadline: ddl.into(),
                        amount_out: amount_out_wei,
                        // TODO: to determin amount_in_maximum, assume no asset is more expensive than 200_000 per quote
                        amount_in_maximum: decimal_to_wei(
                            amount.checked_mul(Decimal::from_i32(200_000).unwrap()).unwrap(),
                            token_in.decimals.into(),
                        ),
                        sqrt_price_limit_x96: get_swap_price_limit(
                            token_in.address,
                            token_out.address,
                            token_in.address,
                        ),
                    };
                    let call = self
                        .dex_contracts
                        .get_v3_swap_router()
                        .await
                        .as_ref()
                        .unwrap()
                        .exact_output_single(param_output)
                        .gas(1_800_000)
                        .gas_price(300_000_000);
                    let ret = call.send().await;
                    match ret {
                        Ok(ref tx) => {
                            info!("send v3 exact out single transaction {:?}", tx);
                            Ok(tx.tx_hash())
                        }
                        Err(e) => Err(OrderError::ContractError(e)),
                    }
                }
            }
            _ => unimplemented!(),
        }
    }

    //     /// # Description
    //     /// fetch pool created events from factory contract
    //     ///
    //     /// Returns vector of pools created
    //     ///
    //     /// # Arguments
    //     /// * `from_block` - start block number
    //     /// * `to_block`   - end block number
    //     pub async fn fetch_pair_created_event(
    //         &self,
    //         from_block: u64,
    //         to_block: u64,
    //     ) -> Result<Vec<Pool>> {
    //         debug!("start fetch within range {:?}, {:?}", from_block, to_block);
    //         match self.pool_variant {
    //             PoolVariant::UniswapV2 => {
    //                 let pools = self
    //                     .v2_factory
    //                     .as_ref()
    //                     .unwrap()
    //                     .pair_created_filter()
    //                     .from_block(BlockNumber::Number(from_block.into()))
    //                     .to_block(BlockNumber::Number(to_block.into()))
    //                     .query()
    //                     .await
    //                     .unwrap()
    //                     .into_iter()
    //                     .map(|p| Pool {
    //                         token_0: p.token_0,
    //                         token_1: p.token_1,
    //                         address: p.pair,
    //                         swap_fee: p.p3,
    //                         pool_variant: PoolVariant::UniswapV2,
    //                     })
    //                     .collect::<Vec<Pool>>();

    //                 debug!(
    //                     "{:?} number of pools created within block range from {:?} to {:?}",
    //                     pools.len(),
    //                     from_block,
    //                     to_block
    //                 );
    //                 Ok(pools)
    //             }
    //             PoolVariant::UniswapV3 => {
    //                 unimplemented!();
    //             }
    //         }
    //     }
    // }

    // # Description
    // get all pairs for a given dex between `start_block` and `current_block`
    // # Args
    // * `dexes` - .
    // * `start_block` - if None, will set to the contract creation block
    // * `end_block` - end block to sync
    // pub async fn sync_dex<M: Middleware + 'static>(
    //     dexes: Vec<Arc<Dex<M>>>,
    //     start_block: Option<BlockNumber>,
    //     end_block: BlockNumber,
    // ) -> Result<Vec<Pool>, PairSyncError> {
    //     // initialize multi progress bar
    //     let multi_progress_bar = MultiProgress::new();

    //     let mut handles = vec![];

    //     // for each dex supplied, get all pair created events
    //     for dex in dexes {
    //         let progress_bar = multi_progress_bar.add(ProgressBar::new(0));

    //         handles.push(tokio::spawn({
    //             let dex = dex.clone();
    //             async move {
    //                 progress_bar.set_style(
    //                     ProgressStyle::with_template(
    //                         "{msg} {bar:40.green/grey} {pos:>7}/{len:7} Blocks",
    //                     )
    //                     .unwrap()
    //                     .progress_chars("##-"),
    //                 );

    //                 let pools =
    //                     get_all_pools(dex, start_block, end_block, progress_bar.clone()).await?;

    //                 progress_bar.reset();
    //                 progress_bar.set_style(
    //                     ProgressStyle::with_template(
    //                         "{msg} {bar:40.green/grey} {pos:>7}/{len:7} Pairs",
    //                     )
    //                     .unwrap()
    //                     .progress_chars("##-"),
    //                 );

    //                 Ok::<Vec<Pool>, PairSyncError>(pools)
    //             }
    //         }));
    //     }

    //     // aggregate the populated pools from each thread
    //     let mut aggregated_pools: Vec<Pool> = vec![];

    //     for handle in handles {
    //         match handle.await {
    //             Ok(sync_result) => aggregated_pools.extend(sync_result?),
    //             Err(join_error) => return Err(PairSyncError::JoinError(join_error)),
    //         }
    //     }

    //     // return the populated aggregated pools vec
    //     Ok(aggregated_pools)
    // }

    // function to get all pair created events for a given Dex factory address
    // async fn get_all_pools<M: Middleware + 'static>(
    //     dex: Arc<Dex<M>>,
    //     start_block: Option<BlockNumber>,
    //     end_block: BlockNumber,
    //     progress_bar: ProgressBar,
    // ) -> Result<Vec<Pool>, PairSyncError> {
    //     // define the step for searching a range of blocks for pair created events
    //     let step = 1000;

    //     // get start block
    //     let resolved_start_block = if let Some(block) = start_block {
    //         block.as_number().unwrap().as_u64()
    //     } else {
    //         dex.factory_creation_block.unwrap().as_number().unwrap().as_u64()
    //     };

    //     let current_block = end_block.as_number().unwrap().as_u64();

    //     // initialize the progress bar message
    //     progress_bar.set_length(current_block - resolved_start_block);
    //     // progress_bar.set_message(format!("Getting all pools from: {}", dex.factory_address));

    //     // init a new vec to keep track of tasks
    //     let mut handles = vec![];

    //     // for each block within the range, get all pairs asynchronously
    //     for from_block in (resolved_start_block..=current_block).step_by(step) {
    //         let progress_bar = progress_bar.clone();

    //         //Spawn a new task to get pair created events from the block range
    //         handles.push(tokio::spawn({
    //             let dex = dex.clone();
    //             async move {
    //                 let mut pools_all = vec![];

    //                 //Get pair created event logs within the block range
    //                 let to_block = from_block + step as u64;

    //                 let mut pools = dex
    //                     .fetch_pair_created_event(from_block, to_block)
    //                     .await
    //                     .expect("unable to fetch");

    //                 // increment the progres bar by the step
    //                 progress_bar.inc(step as u64);
    //                 pools_all.append(&mut pools);

    //                 Ok::<Vec<Pool>, ProviderError>(pools_all)
    //             }
    //         }));
    //     }

    //     // wait for each thread to finish and aggregate the pairs from each Dex into a single aggregated pairs vec
    //     let mut aggregated_pairs: Vec<Pool> = vec![];
    //     for handle in handles {
    //         match handle.await {
    //             Ok(sync_result) => aggregated_pairs.extend(sync_result?),
    //             Err(join_error) => return Err(PairSyncError::JoinError(join_error)),
    //         }
    //     }
    //     Ok(aggregated_pairs)
    // }
}

pub struct DexBackend<M> {
    client: Arc<M>,
    network: Network,
    sender_market_change: SyncSender<MarcketChange>,
    base_token: TokenInfo,
    base_token_quote_amt: Decimal,
    quote_token: TokenInfo,
    v3_fee: u32,
}

impl DexBackend<Provider<Ws>> {
    pub fn new(
        client: Arc<Provider<Ws>>,
        network: Network,
        sender_market_change: SyncSender<MarcketChange>,
        base_token: TokenInfo,
        base_token_quote_amt: Decimal,
        quote_token: TokenInfo,
        v3_fee: u32,
    ) -> Self {
        Self {
            client,
            network,
            sender_market_change,
            base_token,
            base_token_quote_amt,
            quote_token,
            v3_fee,
        }
    }

    pub async fn event_loop(&mut self) -> anyhow::Result<()> {
        // TODO: currently it's hardcoded to v3
        let quoter_address =
            get_dex_address(DexExchange::UniswapV3, self.network, ContractType::UniV3QuoterV2)
                .unwrap();

        let quoter = QuoterV2::new(quoter_address.address, Arc::clone(&self.client));

        let mut new_block_stream = self.client.subscribe_blocks().await.unwrap();
        let mut last_block: u64 = 0;

        let (mut last_dex_sell_price, mut last_dex_buy_price) = (Decimal::ZERO, Decimal::ZERO);

        loop {
            match new_block_stream.next().await {
                Some(block) => {
                    if let Some(block_number) = block.number {
                        let new_block = block_number.as_u64();
                        if new_block > last_block {
                            last_block = new_block;

                            // quote for 500, 3000, all pools
                            let base_quote_amt_in_wei = decimal_to_wei(
                                self.base_token_quote_amt,
                                self.base_token.decimals.into(),
                            );
                            debug!("amt_in_wei: {:?}, base_token_address: {:?}, quote_token_address: {:?}", base_quote_amt_in_wei, self.base_token.address, self.quote_token.address);
                            let rets = try_join_all([
                                quoter
                                    .quote_exact_input_single(QuoteExactInputSingleParams {
                                        token_in: self.base_token.address,
                                        token_out: self.quote_token.address,
                                        amount_in: base_quote_amt_in_wei,
                                        fee: self.v3_fee,
                                        sqrt_price_limit_x96: 0.into(),
                                    })
                                    .call(),
                                quoter
                                    .quote_exact_output_single(QuoteExactOutputSingleParams {
                                        token_in: self.quote_token.address,
                                        token_out: self.base_token.address,
                                        amount: base_quote_amt_in_wei,
                                        fee: self.v3_fee,
                                        sqrt_price_limit_x96: U256::from_str_radix(
                                            "1461446703485210103287273052203988822378723970341",
                                            10,
                                        )
                                        .unwrap(),
                                    })
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
                                        self.quote_token.decimals.into(),
                                    )
                                    .checked_div(self.base_token_quote_amt)
                                    .unwrap();
                                    let (amount_in, _, _, _) = buy;
                                    let buy_price = decimal_from_wei(
                                        amount_in,
                                        self.quote_token.decimals.into(),
                                    )
                                    .checked_div(self.base_token_quote_amt)
                                    .unwrap();

                                    // println!("dex price, sell {:?}, buy {:?}", sell_price, buy_price);
                                    if !sell_price.eq(&last_dex_sell_price)
                                        || !buy_price.eq(&last_dex_buy_price)
                                    {
                                        last_dex_sell_price = sell_price;
                                        last_dex_buy_price = buy_price;

                                        debug!("send dex price change, block number {:?}, sell price: {:?}, buy price: {:?} ",block.number, sell_price, buy_price);
                                        let ret =
                                            self.sender_market_change.clone().send(MarcketChange {
                                                cex: None,
                                                dex: Some(CurrentSpread {
                                                    best_bid: sell_price,
                                                    best_ask: buy_price,
                                                }),
                                            });
                                        match ret {
                                            Ok(()) => debug!("send price chagne success"),
                                            Err(e) => {
                                                error!("error in send dex price change, {:?}", e);
                                                panic!("error in send dex price change, {:?}", e);
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
    }
}
