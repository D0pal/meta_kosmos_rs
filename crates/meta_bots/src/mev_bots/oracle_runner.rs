use colored::Colorize;
use dashmap::DashMap;
use ethers::prelude::*;

use meta_dex::{
    oracle::{BlockOracle},
    pool::Pool,
    // sync_dex,
    DexService,
};
use std::{sync::Arc};
use tokio::sync::RwLock;
use tracing::info;



// Update latest block variable whenever we recieve a new block
//
// Arguments:
// * `oracle`: oracle to update
pub fn start_block_oracle(client: Arc<Provider<Ws>>, oracle: &mut Arc<RwLock<BlockOracle>>) {
    let next_block_clone = oracle.clone();

    tokio::spawn(async move {
        // loop so we can reconnect if the websocket connection is lost
        loop {
            let mut block_stream = if let Ok(stream) = client.subscribe_blocks().await {
                stream
            } else {
                panic!("Failed to create new block stream");
            };

            while let Some(block) = block_stream.next().await {
                // lock the RwLock for write access and update the variable
                {
                    let mut lock = next_block_clone.write().await;
                    lock.update_block_number(block.number.unwrap());
                    lock.update_block_timestamp(block.timestamp);
                    lock.update_base_fee(block);

                    let latest_block = &lock.latest_block;
                    let next_block = &lock.next_block;
                    info!(
                    "{}",
                    format!(
                        "New Block: (number:{:?}, timestamp:{:?}, basefee:{:?}), Next Block: (number:{:?}, timestamp:{:?}, basefee:{:?})",
                        latest_block.number, latest_block.timestamp, latest_block.base_fee, next_block.number, next_block.timestamp, next_block.base_fee
                    )
                    .bright_purple()
                    .on_black()
                    );
                } // remove write lock due to being out of scope here
            }
        }
    });
}

pub fn start_add_new_pools<M: Middleware + 'static>(
    _client: Arc<Provider<Ws>>, //Vec<Arc<Dex<M>>>,
    all_pools: &mut Arc<DashMap<Address, Pool>>,
    _dexes: Vec<Arc<DexService<M>>>,
) {
    let _all_pools = all_pools.clone();

    tokio::spawn(async move {
        // loop so we can reconnect if the websocket connection is lost
        loop {
            // let mut block_stream = if let Ok(stream) = client.subscribe_blocks().await {
            //     stream
            // } else {
            //     panic!("Failed to create new block stream");
            // };

            // let mut counter = 0;
            // let mut current_block_num = client.get_block_number().await.unwrap();

            // while let Some(block) = block_stream.next().await {
            //     counter += 1;

            //     // every 50 blocks fetch and new pools
            //     if counter == 50 {
            //         let latest_block_number = block.number.unwrap();
            //         let fetched_new_pools = sync_dex(
            //             dexes.clone(),
            //             Some(BlockNumber::Number(current_block_num)),
            //             BlockNumber::Number(latest_block_number),
            //         )
            //         .await
            //         .unwrap();

            //         let fetched_pools_count = fetched_new_pools.len();

            //         // turn fetched pools into hashmap
            //         for pool in fetched_new_pools {
            //             // Create hashmap from our vec
            //             all_pools.insert(pool.address, pool);
            //         }

            //         counter = 0;
            //         current_block_num = latest_block_number;
            //         info!("added {} new pools", fetched_pools_count);
            //     }
            // }
        }
    });
}

// pub fn start_mega_sandwich_oracle(
//     client: Arc<Provider<Ws>>,
//     new_block_delay_milli: u64,
//     bundle_sender: Arc<RwLock<BundleSender>>,
//     sandwich_state: Arc<BotState>,
//     sandwich_maker: Arc<SandwichMaker>,
//     network: Network,
//     weth_address: Address,
//     sandwidth_contract_address: Address,
//     searcher_address: Address,
// ) {
//     tokio::spawn(async move {
//         // loop so we can reconnect if the websocket connection is lost
//         loop {
//             let mut block_stream = if let Ok(stream) = client.subscribe_blocks().await {
//                 stream
//             } else {
//                 panic!("Failed to create new block stream");
//             };

//             while let Some(block) = block_stream.next().await {
//                 // clear all recipes
//                 // enchanement: don't do this step but keep recipes because they can be used in future
//                 {
//                     let mut bundle_sender_guard = bundle_sender.write().await;
//                     bundle_sender_guard.pending_sandwiches.clear();
//                 } // lock removed here

//                 // 10.5 seconds from when new block was detected, caluclate mega sandwich
//                 thread::sleep(Duration::from_millis(new_block_delay_milli)); // 10_500 FOR ETH
//                 let next_block_info = BlockInfo::find_next_block_info(block);
//                 {
//                     bundle_sender
//                         .write()
//                         .await
//                         .make_mega_sandwich(
//                             client.clone(),
//                             next_block_info,
//                             sandwich_state.clone(),
//                             sandwich_maker.clone(),
//                             network,
//                             weth_address,
//                             sandwidth_contract_address,
//                             searcher_address,
//                         )
//                         .await;
//                 } // lock removed here
//             }
//         }
//     });
// }
