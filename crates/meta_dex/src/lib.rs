pub mod error;
pub mod pool;
pub mod sandwidth;
pub mod oracle;
pub mod enums;

pub mod prelude {
    pub use super::{error::*, pool::*, sandwidth::*, oracle::*};
}


use std::sync::Arc;

use ethers::{prelude::*, utils::__serde_json::de};
use eyre::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use meta_address::get_dex_address;
use meta_common::enums::{ContractType, DexExchange, Network, PoolVariant};
use meta_contracts::bindings::uniswap_v2_factory::UniswapV2Factory;
use tracing::{debug, debug_span, info, instrument, trace, warn};


use crate::prelude::{PairSyncError, Pool};

#[derive(Clone, Debug)]
pub struct Dex<M> {
    pub client: Arc<M>,
    pub network: Network,
    pub dex_exchange: DexExchange,
    pub pool_variant: PoolVariant,
    pub factory_address: Address,
    pub factory_creation_block: BlockNumber,
}

impl<M: Middleware> Dex<M> {
    /// # Description
    /// Creates a new dex instance
    pub fn new(
        client: Arc<M>,
        network: Network,
        dex_exchange: DexExchange,
    ) -> Self {
        
        let pool_variant: PoolVariant = match dex_exchange {
            DexExchange::UniswapV3 => PoolVariant::UniswapV3,
            _ => PoolVariant::UniswapV2,
        };
        let contract_type = match pool_variant {
            PoolVariant::UniswapV2 => ContractType::UniV2Factory,
            PoolVariant::UniswapV3 => ContractType::UniV3Factory,
        };
        let factory_contract_info = get_dex_address(dex_exchange, network, contract_type).unwrap();
        Dex {
            client: client.clone(),
            network,
            dex_exchange,
            pool_variant,
            factory_address: factory_contract_info.address,
            factory_creation_block: BlockNumber::Number(
                factory_contract_info.created_blk_num.into(),
            ),
        }
    }

    /// # Description
    /// fetch pool created events from factory contract
    ///
    /// Returns vector of pools created
    ///
    /// # Arguments
    /// * `from_block` - start block number
    /// * `to_block`   - end block number
    pub async fn fetch_pair_created_event(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<Pool>> {
        debug!("start fetch within range {:?}, {:?}", from_block, to_block);
        match self.pool_variant {
            PoolVariant::UniswapV2 => {
                let uniswap_v2_factory = UniswapV2Factory::new(self.factory_address, self.client.clone());
                let pools = uniswap_v2_factory
                    .pair_created_filter()
                    .from_block(BlockNumber::Number(from_block.into()))
                    .to_block(BlockNumber::Number(to_block.into()))
                    .query()
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|p| Pool {
                        token_0: p.token_0,
                        token_1: p.token_1,
                        address: p.pair,
                        swap_fee: p.p3,
                        pool_variant: PoolVariant::UniswapV2,
                    })
                    .collect::<Vec<Pool>>();

                debug!(
                    "{:?} number of pools created within block range from {:?} to {:?}",
                    pools.len(),
                    from_block,
                    to_block
                );
                Ok(pools)
            }
            PoolVariant::UniswapV3 => {
                unimplemented!();
            }
        }
    }
}

/// # Description
/// get all pairs for a given dex between `start_block` and `current_block`
/// # Args
/// * `dexes` - .
/// * `start_block` - if None, will set to the contract creation block
/// * `end_block` - end block to sync
pub async fn sync_dex<M: Middleware + 'static>(
    dexes: Vec<Arc<Dex<M>>>,
    start_block: Option<BlockNumber>,
    end_block: BlockNumber,
) -> Result<Vec<Pool>, PairSyncError> {
    // initialize multi progress bar
    let multi_progress_bar = MultiProgress::new();

    let mut handles = vec![];

    // for each dex supplied, get all pair created events
    for dex in dexes {
        let progress_bar = multi_progress_bar.add(ProgressBar::new(0));

        handles.push(tokio::spawn({
            let dex = dex.clone();
            async move {
                progress_bar.set_style(
                    ProgressStyle::with_template("{msg} {bar:40.green/grey} {pos:>7}/{len:7} Blocks")
                        .unwrap()
                        .progress_chars("##-"),
                );
    
                let pools = get_all_pools(dex, start_block, end_block, progress_bar.clone()).await?;
    
                progress_bar.reset();
                progress_bar.set_style(
                    ProgressStyle::with_template("{msg} {bar:40.green/grey} {pos:>7}/{len:7} Pairs")
                        .unwrap()
                        .progress_chars("##-"),
                );
    
                Ok::<Vec<Pool>, PairSyncError>(pools)
            }
        }));
    }

    // aggregate the populated pools from each thread
    let mut aggregated_pools: Vec<Pool> = vec![];

    for handle in handles {
        match handle.await {
            Ok(sync_result) => aggregated_pools.extend(sync_result?),
            Err(join_error) => return Err(PairSyncError::JoinError(join_error)),
        }
    }

    // return the populated aggregated pools vec
    Ok(aggregated_pools)
}

/// function to get all pair created events for a given Dex factory address
async fn get_all_pools<M: Middleware + 'static>(
    dex: Arc<Dex<M>>,
    start_block: Option<BlockNumber>,
    end_block: BlockNumber,
    progress_bar: ProgressBar,
) -> Result<Vec<Pool>, PairSyncError> {

    // define the step for searching a range of blocks for pair created events
    let step = 1000;

    // get start block
    let resolved_start_block = if let Some(block) = start_block {
        block.as_number().unwrap().as_u64()
    } else {
        dex.factory_creation_block.as_number().unwrap().as_u64()
    };

    let current_block = end_block.as_number().unwrap().as_u64();

    // initialize the progress bar message
    progress_bar.set_length(current_block - resolved_start_block);
    progress_bar.set_message(format!("Getting all pools from: {}", dex.factory_address));

    // init a new vec to keep track of tasks
    let mut handles = vec![];

    // for each block within the range, get all pairs asynchronously
    for from_block in (resolved_start_block..=current_block).step_by(step) {
        let progress_bar = progress_bar.clone();

        //Spawn a new task to get pair created events from the block range
        handles.push(tokio::spawn({
            let dex = dex.clone();
            async move {
                let mut pools_all = vec![];
    
                //Get pair created event logs within the block range
                let to_block = from_block + step as u64;
    
                let mut pools = dex.fetch_pair_created_event(from_block, to_block).await.expect("unable to fetch");
    
                // increment the progres bar by the step
                progress_bar.inc(step as u64);
                pools_all.append(&mut pools);
    
                Ok::<Vec<Pool>, ProviderError>(pools_all)
            }
        }));
    }

    // wait for each thread to finish and aggregate the pairs from each Dex into a single aggregated pairs vec
    let mut aggregated_pairs: Vec<Pool> = vec![];
    for handle in handles {
        match handle.await {
            Ok(sync_result) => aggregated_pairs.extend(sync_result?),
            Err(join_error) => return Err(PairSyncError::JoinError(join_error)),
        }
    }
    Ok(aggregated_pairs)
}
