use dashmap::DashMap;
use ethers::prelude::*;
use eyre::Result;
use hashbrown::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use meta_common::{constants::ERC20_TRANSFER_EVENT_SIG, enums::Network};
use meta_contracts::wrappers::Erc20Wrapper;
// use crate::prelude::fork_factory::ForkFactory;
// use crate::prelude::sandwich_types::RawIngredients;
// use crate::prelude::{make_sandwich, Dex, Pool};
// use crate::rpc_extensions;
// use crate::types::BlockOracle;
// use crate::utils;
// use crate::utils::tx_builder::SandwichMaker;
// use colored::Colorize;

// mod oracles;
// mod state;
// mod bundle_sender;
// use bundle_sender::*;

#[derive(Clone, Debug)]
/// Holds the state of the bot
pub struct BotState {
    pub network: Network,
    pub sandwidth_contract_address: Address,
    pub weth_address: Address,
    pub token_dust: Arc<RwLock<Vec<Address>>>,
    pub weth_balance: Arc<RwLock<U256>>,
}

impl BotState {
    // Create a new instance of the bot state
    //
    // Arguments:
    // * `sandwich_inception_block`: block number sandwich was deployed
    // * `client`: websocket provider to use for fetching data
    //
    // Returns:
    // Ok(BotState) if successful
    // Err(eyre::Error) if failed to create instance
    pub async fn new(
        network: Network,
        sandwidth_contract_address: Address,
        sandwich_inception_block: U64,
        weth_address: Address,
        client: &Arc<Provider<Ws>>,
    ) -> Result<Self> {
        let token_dust = Self::find_all_dust(
            network,
            sandwidth_contract_address,
            sandwich_inception_block,
            client,
        )
        .await?;
        let token_dust = Arc::new(RwLock::new(token_dust));

        // let weth_contract =
        //     utils::contracts::get_erc20_contract(&utils::constants::get_weth_address(), client);
        // let weth_balance = weth_contract
        //     .balance_of(utils::dotenv::get_sandwich_contract_address())
        //     .call()
        //     .await?;
        // let weth_balance = Arc::new(RwLock::new(weth_balance));

        Ok(BotState {
            network,
            sandwidth_contract_address,
            weth_address,
            token_dust,
            weth_balance: Arc::new(RwLock::new(U256::from(0))), // weth_balance,
        })
    }

    // Check if contract has dust for specific token
    //
    // Arguments:
    // * `&self`: refernce to `BotState` instance
    // * `token`: token to check dust for
    //
    // Returns:
    // bool: true if contract has dust for token, false otherwise
    pub async fn has_dust(&self, token: &Address) -> bool {
        self.token_dust.read().await.contains(token)
    }

    // Add dust to contract
    //
    // Arguments:
    // * `&self`: reference to `BotState` instance
    // * `token`: token to add dust for
    pub async fn add_dust(&self, token: Address) {
        let mut dust = self.token_dust.write().await;
        dust.push(token);
    }

    // Update the WETH balance of the contract
    //
    // Arguments:
    // * `&self`: reference to `BotState` instance
    //
    // Returns: nothing
    pub async fn update_weth_balance(&self, value_to_add: U256) {
        let mut lock = self.weth_balance.write().await;
        *lock += value_to_add;
    }

    // Find dust that bot has collected from a specific block onwards
    //
    // Arguments:
    // * `start_block`: block to start searching for dust
    // * `client`: websocket provider to use for fetching data
    //
    // Returns:
    // `Ok(Vec<Address>)`: address of token dust collected by bot
    // `Err(eyre::Error)`: failed to find dust
    async fn find_all_dust(
        network: Network,
        sandwidth_contract_address: Address,
        start_block: U64,
        client: &Arc<Provider<Ws>>,
    ) -> Result<Vec<Address>> {
        // Define the step for searching a range of block logs for transfer events
        let step = 1_000;

        // Find dust upto this block
        let current_block = match client.get_block_number().await {
            Ok(block) => block.as_u64(),
            Err(e) => {
                error!("Failed to get current_block {:?}", e);
                eyre::bail!("todo error msg here");
            }
        };

        let start_block = start_block.as_u64();

        // holds erc20 and associated balance
        let mut address_interacted_with = HashSet::new();

        // for each block within the range, get all transfer events asynchronously
        for from_block in (start_block..=current_block).step_by(step) {
            let to_block = from_block + step as u64;

            // check for all incoming and outgoing txs within step range
            let transfer_logs = client
                .get_logs(
                    &Filter::new()
                        .topic0(*ERC20_TRANSFER_EVENT_SIG)
                        .topic1(sandwidth_contract_address)
                        .from_block(BlockNumber::Number(U64([from_block])))
                        .to_block(BlockNumber::Number(U64([to_block]))),
                )
                .await?;

            let receive_logs = client
                .get_logs(
                    &Filter::new()
                        .topic0(*ERC20_TRANSFER_EVENT_SIG)
                        .topic2(sandwidth_contract_address)
                        .from_block(BlockNumber::Number(U64([from_block])))
                        .to_block(BlockNumber::Number(U64([to_block]))),
                )
                .await?;

            // combine all logs
            for log in transfer_logs {
                address_interacted_with.insert(log.address);
            }
            for log in receive_logs {
                address_interacted_with.insert(log.address);
            }
        }

        let mut token_dust = vec![];

        // doing calls to remove false positives
        for touched_addr in address_interacted_with {
            let erc20 = Erc20Wrapper::new(network, touched_addr, client.clone()).await;
            let balance: U256 = erc20.token_contract.balance_of(sandwidth_contract_address).await?;

            if !balance.is_zero() {
                debug!("balance of {:?} is {:?}", touched_addr, balance);
                token_dust.push(touched_addr);
            }
        }

        info!("Found {:?} tokens worth of dust", token_dust.len());

        Ok(token_dust)
    }
}

pub struct BotSandwidth {
    sandwich_state: Arc<BotState>,
    // latest_block_oracle: Arc<RwLock<BlockOracle>>,
    // client: Arc<Provider<Ws>>,
    // all_pools: Arc<DashMap<Address, Pool>>,
    // sandwich_maker: Arc<SandwichMaker>,
    // bundle_sender: Arc<RwLock<BundleSender>>,
    // dexes: Vec<Dex>,
}

impl BotSandwidth {
    // Create new bot instance
    //
    // Arguments:
    // * `client`: websocket provider used to make calls
    // * `pool_vec`: vector of pools that the bot will monitor
    //
    // Returns:
    // * Ok(Bot) if successful
    // * Err(eyre::Error) if not successful
    pub async fn new(
        network: Network,
        sandwidth_contract_address: Address,
        sandwich_inception_block: U64,
        weth_address: Address,
        client: Arc<Provider<Ws>>,
        // pool_vec: Vec<Pool>,
        // dexes: Vec<Dex>,
    ) -> Result<Self> {
        // create hashmap from our vec of pools (faster access when doing lookups)
        // let all_pools: DashMap<Address, Pool> = DashMap::new();
        // for pool in pool_vec {
        //     all_pools.insert(pool.address, pool);
        // }

        // let all_pools = Arc::new(all_pools);

        // let sandwich_inception_block = utils::dotenv::get_sandwich_inception_block();
        let sandwich_state = BotState::new(
            network,
            sandwidth_contract_address,
            sandwich_inception_block,
            weth_address,
            &client,
        )
        .await?;
        let sandwich_state = Arc::new(sandwich_state);

        // let sandwich_maker = Arc::new(SandwichMaker::new().await);

        // let latest_block_oracle = BlockOracle::new(&client).await?;
        // let latest_block_oracle = Arc::new(RwLock::new(latest_block_oracle));

        // let bundle_sender = Arc::new(RwLock::new(BundleSender::new().await));

        Ok(BotSandwidth {
            // client,
            // all_pools,
            // latest_block_oracle,
            sandwich_state,
            // sandwich_maker,
            // bundle_sender,
            // dexes,
        })
    }

    // Run the bot by starting a new mempool stream and filtering txs for opportunities
    //
    // Arguments:
    // * `&mut self`: reference to mutable self
    //
    // Returns:
    // Ok(()) if successful
    // Err(eyre::Error) if encounters error during execution
    // pub async fn run(&mut self) -> Result<()> {
    //     log::info!("Starting bot");

    //     oracles::start_add_new_pools(&mut self.all_pools, self.dexes.clone());
    //     oracles::start_block_oracle(&mut self.latest_block_oracle);
    //     oracles::start_mega_sandwich_oracle(
    //         self.bundle_sender.clone(),
    //         self.sandwich_state.clone(),
    //         self.sandwich_maker.clone(),
    //     );

    //     let mut mempool_stream = if let Ok(stream) =
    //         rpc_extensions::subscribe_pending_txs_with_body(&self.client).await
    //     {
    //         stream
    //     } else {
    //         panic!("Failed to create mempool stream");
    //     };

    //     while let Some(mut victim_tx) = mempool_stream.next().await {
    //         let client = utils::create_websocket_client().await?;
    //         let block_oracle = {
    //             let read_lock = self.latest_block_oracle.read().await;
    //             (*read_lock).clone()
    //         };
    //         let all_pools = &self.all_pools;
    //         let sandwich_balance = {
    //             let read_lock = self.sandwich_state.weth_balance.read().await;
    //             (*read_lock).clone()
    //         };
    //         // ignore txs that we can't include in next block
    //         // enhancement: simulate all txs, store result, and use result when tx can included
    //         if victim_tx.max_fee_per_gas.unwrap_or(U256::zero()) < block_oracle.next_block.base_fee
    //         {
    //             log::info!("{}", format!("{:?} mf<nbf", victim_tx.hash).cyan());
    //             continue;
    //         }

    //         // recover from field from vrs (ECDSA)
    //         // enhancement: expensive operation, can avoid by modding rpc to share `from` field
    //         if let Ok(from) = victim_tx.recover_from() {
    //             victim_tx.from = from;
    //         } else {
    //             log::error!("{}", format!("{:?} ecdsa recovery failed", victim_tx.hash).red());
    //             continue;
    //         };

    //         // get all state diffs that this tx produces
    //         let state_diffs = if let Some(sd) = utils::state_diff::get_from_txs(
    //             &self.client,
    //             &vec![victim_tx.clone()],
    //             BlockNumber::Number(block_oracle.latest_block.number),
    //         )
    //         .await
    //         {
    //             sd
    //         } else {
    //             log::info!("{:?}", victim_tx.hash);
    //             continue;
    //         };

    //         // if tx has statediff on pool addr then record it in `sandwichable_pools`
    //         let sandwichable_pools =
    //             if let Some(sp) = utils::state_diff::extract_pools(&state_diffs, &all_pools) {
    //                 sp
    //             } else {
    //                 log::info!("{:?}", victim_tx.hash);
    //                 continue;
    //             };

    //         let fork_block =
    //             Some(BlockId::Number(BlockNumber::Number(block_oracle.next_block.number)));

    //         // create evm simulation handler by setting up `fork_factory`
    //         let initial_db = utils::state_diff::to_cache_db(&state_diffs, fork_block, &self.client)
    //             .await
    //             .unwrap();
    //         let fork_factory =
    //             ForkFactory::new_sandbox_factory(client.clone(), initial_db, fork_block);

    //         // search for opportunities in all pools that the tx touches (concurrently)
    //         for sandwichable_pool in sandwichable_pools {
    //             if !sandwichable_pool.is_weth_input {
    //                 // enhancement: increase opportunities by handling swaps in pools with stables
    //                 log::info!("{:?} [weth_is_output]", victim_tx.hash);
    //                 continue;
    //             } else {
    //                 log::info!("{}", format!("{:?} [weth_is_input]", victim_tx.hash).green());
    //             }

    //             // prepare variables for new thread
    //             let victim_tx = victim_tx.clone();
    //             let sandwichable_pool = sandwichable_pool.clone();
    //             let mut fork_factory = fork_factory.clone();
    //             let block_oracle = block_oracle.clone();
    //             let sandwich_state = self.sandwich_state.clone();
    //             let sandwich_maker = self.sandwich_maker.clone();
    //             let bundle_sender = self.bundle_sender.clone();
    //             let state_diffs = state_diffs.clone();

    //             tokio::spawn(async move {
    //                 // enhancement: increase opportunities by handling swaps in pools with stables
    //                 let input_token = utils::constants::get_weth_address();
    //                 let victim_hash = victim_tx.hash;

    //                 // variables used when searching for opportunity
    //                 let raw_ingredients = if let Ok(data) = RawIngredients::new(
    //                     &sandwichable_pool.pool,
    //                     vec![victim_tx],
    //                     input_token,
    //                     state_diffs,
    //                 )
    //                 .await
    //                 {
    //                     data
    //                 } else {
    //                     log::error!("Failed to create raw ingredients for: {:?}", &victim_hash);
    //                     return;
    //                 };

    //                 // find optimal input to sandwich tx
    //                 let mut optimal_sandwich = match make_sandwich::create_optimal_sandwich(
    //                     &raw_ingredients,
    //                     sandwich_balance,
    //                     &block_oracle.next_block,
    //                     &mut fork_factory,
    //                     &sandwich_maker,
    //                 )
    //                 .await
    //                 {
    //                     Ok(optimal) => optimal,
    //                     Err(e) => {
    //                         log::info!(
    //                             "{}",
    //                             format!("{:?} sim failed due to {:?}", &victim_hash, e).yellow()
    //                         );
    //                         return;
    //                     }
    //                 };

    //                 // check if has dust
    //                 let other_token = if optimal_sandwich.target_pool.token_0
    //                     != utils::constants::get_weth_address()
    //                 {
    //                     optimal_sandwich.target_pool.token_0
    //                 } else {
    //                     optimal_sandwich.target_pool.token_1
    //                 };

    //                 if sandwich_state.has_dust(&other_token).await {
    //                     optimal_sandwich.has_dust = true;
    //                 }

    //                 // spawn thread to send tx to builders
    //                 let optimal_sandwich = optimal_sandwich.clone();
    //                 let optimal_sandwich_two = optimal_sandwich.clone();
    //                 let sandwich_maker = sandwich_maker.clone();
    //                 let sandwich_state = sandwich_state.clone();

    //                 if optimal_sandwich.revenue > U256::zero() {
    //                     tokio::spawn(async move {
    //                         match bundle_sender::send_bundle(
    //                             &optimal_sandwich,
    //                             block_oracle.next_block,
    //                             sandwich_maker,
    //                             sandwich_state.clone(),
    //                         )
    //                         .await
    //                         {
    //                             Ok(_) => { /* all reporting already done inside of send_bundle */ }
    //                             Err(e) => {
    //                                 log::info!(
    //                                     "{}",
    //                                     format!(
    //                                         "{:?} failed to send bundle, due to {:?}",
    //                                         optimal_sandwich.print_meats(),
    //                                         e
    //                                     )
    //                                     .bright_magenta()
    //                                 );
    //                             }
    //                         };
    //                     });
    //                 }

    //                 // spawn thread to add tx for mega sandwich calculation
    //                 let bundle_sender = bundle_sender.clone();
    //                 tokio::spawn(async move {
    //                     bundle_sender.write().await.add_recipe(optimal_sandwich_two).await;
    //                 });
    //             });
    //         }
    //     }
    //     Ok(())
    // }
}
