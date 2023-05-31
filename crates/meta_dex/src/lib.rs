//! credit to 0xKitsune's cfmms-rs: https://github.com/0xKitsune/cfmms-rs/tree/main/src/dex

pub mod pool;
pub mod error;

use std::sync::Arc;

use ethers::prelude::*;
use eyre::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use meta_common::enums::{Network, DexExchange, PoolVariant, ContractType};
use meta_contracts::bindings::uniswap_v2_factory::UniswapV2Factory;
use meta_address::get_dex_address;

pub mod prelude {
    pub use super::{
        pool::*, error::*,
    };
}

use crate::{
    prelude::{Pool, PairSyncError},
};



#[derive(Clone, Copy, Debug)]
pub struct Dex {
    pub network: Network,
    pub dex_exchange: DexExchange,
    pub pool_variant: PoolVariant,
    pub factory_address: Address,
    pub creation_block: BlockNumber,
}

impl Dex {
    // Creates a new dex instance
    pub fn new(network: Network, dex_exchange: DexExchange, pool_variant: PoolVariant, ) -> Dex {
        let contract_type = match pool_variant {
            PoolVariant::UniswapV2 => ContractType::UNI_V2_FACTORY,
            PoolVariant::UniswapV3 => ContractType::UNI_V3_FACTORY
        };
        let factory_contract_info = get_dex_address(dex_exchange, network, contract_type).unwrap();
        Dex {
            network,
            dex_exchange,
            pool_variant,
            factory_address: factory_contract_info.address,
            creation_block: BlockNumber::Number(factory_contract_info.created_blk_num.into()),
        }
    }

    // Parse logs and extract pools
    // pub fn new_pool_from_event(&self, log: Log, provider: Arc<Provider<Ws>>) -> Option<Pool> {
    //     match self.pool_variant {
    //         PoolVariant::UniswapV2 => {
    //             let uniswap_v2_factory = UniswapV2Factory::new(self.factory_address, provider);
    //             let (token_0, token_1, address, _) = if let Ok(pair) = uniswap_v2_factory
    //                 .decode_event::<(Address, Address, Address, U256)>(
    //                     "PairCreated",
    //                     log.topics,
    //                     log.data,
    //                 ) {
    //                 pair
    //             } else {
    //                 return None;
    //             };

    //             // ignore pool does not have weth as one of its tokens
    //             if ![token_0, token_1].contains(&utils::constants::get_weth_address()) {
    //                 return None;
    //             }

    //             Some(Pool::new(
    //                 address,
    //                 token_0,
    //                 token_1,
    //                 U256::from(3000),
    //                 PoolVariant::UniswapV2,
    //             ))
    //         }
    //         PoolVariant::UniswapV3 => {
    //             unimplemented!();
    //             // let uniswap_v3_factory = UniswapV3Factory::new(self.factory_address, provider);

    //             // let (token_0, token_1, fee, _, address) = if let Ok(pool) = uniswap_v3_factory
    //             //     .decode_event::<(Address, Address, u32, u128, Address)>(
    //             //         "PoolCreated",
    //             //         log.topics,
    //             //         log.data,
    //             //     ) {
    //             //     pool
    //             // } else {
    //             //     return None;
    //             // };

    //             // // ignore pair does not have weth as one of its tokens
    //             // if ![token_0, token_1].contains(&utils::constants::get_weth_address()) {
    //             //     return None;
    //             // }

    //             // Some(Pool::new(
    //             //     address,
    //             //     token_0,
    //             //     token_1,
    //             //     U256::from(fee),
    //             //     PoolVariant::UniswapV3,
    //             // ))
    //         }
    //     }
    // }
}

// get all pairs for a given dex between `start_block` and `current_block`
// pub async fn sync_dex(
//     dexes: Vec<Dex>,
//     client: &Arc<Provider<Ws>>,
//     current_block: U64,
//     start_block: Option<BlockNumber>,
// ) -> Result<Vec<Pool>, PairSyncError> {
//     // initialize multi progress bar
//     let multi_progress_bar = MultiProgress::new();

//     let mut handles = vec![];

//     // for each dex supplied, get all pair created events
//     for dex in dexes {
//         let async_provider = client.clone();
//         let progress_bar = multi_progress_bar.add(ProgressBar::new(0));

//         handles.push(tokio::spawn(async move {
//             progress_bar.set_style(
//                 ProgressStyle::with_template("{msg} {bar:40.green/grey} {pos:>7}/{len:7} Blocks")
//                     .unwrap()
//                     .progress_chars("##-"),
//             );

//             let pools = get_all_pools(
//                 dex,
//                 async_provider.clone(),
//                 BlockNumber::Number(current_block),
//                 start_block,
//                 progress_bar.clone(),
//             )
//             .await?;

//             progress_bar.reset();
//             progress_bar.set_style(
//                 ProgressStyle::with_template("{msg} {bar:40.green/grey} {pos:>7}/{len:7} Pairs")
//                     .unwrap()
//                     .progress_chars("##-"),
//             );

//             Ok::<Vec<Pool>, PairSyncError>(pools)
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
// async fn get_all_pools(
//     dex: Dex,
//     provider: Arc<Provider<Ws>>,
//     current_block: BlockNumber,
//     start_block: Option<BlockNumber>,
//     progress_bar: ProgressBar,
// ) -> Result<Vec<Pool>, PairSyncError> {
//     // define the step for searching a range of blocks for pair created events
//     let step = 10_000;

//     // get start block
//     let creation_block = if let Some(block) = start_block {
//         block.as_number().unwrap().as_u64()
//     } else {
//         dex.creation_block.as_number().unwrap().as_u64()
//     };

//     let current_block = current_block.as_number().unwrap().as_u64();

//     // initialize the progress bar message
//     progress_bar.set_length(current_block - creation_block);
//     progress_bar.set_message(format!("Getting all pools from: {}", dex.factory_address));

//     // init a new vec to keep track of tasks
//     let mut handles = vec![];

//     // for each block within the range, get all pairs asynchronously
//     for from_block in (creation_block..=current_block).step_by(step) {
//         let provider = provider.clone();
//         let progress_bar = progress_bar.clone();

//         //Spawn a new task to get pair created events from the block range
//         handles.push(tokio::spawn(async move {
//             let mut pools = vec![];

//             //Get pair created event logs within the block range
//             let to_block = from_block + step as u64;

//             let logs = provider
//                 .get_logs(
//                     &Filter::new()
//                         .topic0(ValueOrArray::Value(
//                             dex.pool_variant.pool_created_event_signature(),
//                         ))
//                         .address(dex.factory_address)
//                         .from_block(BlockNumber::Number(U64([from_block])))
//                         .to_block(BlockNumber::Number(U64([to_block]))),
//                 )
//                 .await?;

//             // increment the progres bar by the step
//             progress_bar.inc(step as u64);

//             // for each pair created log, create a new Pair type and add it to the pairs vec
//             for log in logs {
//                 match dex.new_pool_from_event(log, provider.clone()) {
//                     Some(pool) => pools.push(pool),
//                     None => continue,
//                 }
//             }

//             Ok::<Vec<Pool>, ProviderError>(pools)
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
