pub mod errors;
pub mod evm;
pub mod flashbots;
pub mod fork_db;
pub mod inspectors;
pub mod oracle;
pub mod pool;
pub mod storage;

pub use flashbots::*;

use dashmap::DashMap;
use errors::SimulationError;
use ethers::{core::types::Bytes, prelude::*, types::transaction::eip2718::TypedTransaction};
use evm::configure_tx_env;
use fork_db::fork_factory::ForkFactory;
use foundry_evm::decode::decode_revert;
use futures::future::join_all;
use lazy_static::lazy_static;
use meta_common::constants::ARBITRUM_SENDER;
// use ethers::core::types::trace::geth::DiffMode;
use bytes::Bytes as StdBytes;

use meta_util::{
    defi::get_tick_from_slot_value,
    ether::{address_from_str, h256_to_b256},
};
use oracle::BlockInfo;
use pool::{BalanceChange, DefiStorage, SimPool};
use revm::{
    db::{CacheDB, EmptyDB},
    inspectors::NoOpInspector,
    primitives::{bytes, ExecutionResult, Output, TransactTo, U256 as rU256},
};
use std::{
    collections::{BTreeMap},
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{mpsc::Sender, Mutex, RwLock},
    task::JoinError,
};

use crate::pool::{get_v3_pool_slot_0, prepare_pool};

pub struct EvmSimulator {
    pub ws_provider: Arc<Provider<Ws>>,
    // pub weth_address: Address,
    pub target_pools: Option<Arc<DashMap<Address, DefiStorage>>>,
    pub latest_block_info: Arc<RwLock<BlockInfo>>,
    pub fork_factory: Option<Arc<Mutex<ForkFactory>>>,
}

lazy_static! {
    pub static ref SIMULATION_CALLER: Address =
        address_from_str("0x9a6dcf5e566fa65c67a5f82aa98d03207e065726");
}

#[derive(Debug)]
pub enum ReplayTransactionResult {
    Success { gas_used: u64, gas_refunded: u64, output: StdBytes },
    Revert { gas_used: u64, message: String },
}

pub struct EvmFork {
    pub fork_factory: Arc<RwLock<ForkFactory>>,
}

impl EvmFork {
    pub fn new(fork_factory: Arc<RwLock<ForkFactory>>) -> Self {
        Self { fork_factory }
    }
    pub async fn multi_call_contract(
        &self,
        quoter_address: Address,
        data: Vec<Bytes>,
    ) -> anyhow::Result<Vec<Result<Bytes, JoinError>>> {
        let mut all_handlers: Vec<tokio::task::JoinHandle<Bytes>> = vec![];

        for quote_data in data {
            let ff_clone = self.fork_factory.clone();
            let handler: tokio::task::JoinHandle<Bytes> = tokio::task::spawn(async move {
                let g = ff_clone.read().await;
                let mut evm = revm::EVM::new();
                let fork_db = g.new_sandbox_fork();
                evm.database(fork_db);
                evm.env.tx.transact_to = TransactTo::Call(quoter_address.clone().0.into());
                evm.env.tx.data = quote_data.clone().0.into();
                evm.env.tx.value = rU256::ZERO;

                let _ret = evm.transact_ref().unwrap();

                let output: Bytes = match _ret.result {
                    ExecutionResult::Success { output, .. } => match output {
                        Output::Call(o) => o.into(),
                        Output::Create(o, _) => o.into(),
                    },
                    ExecutionResult::Revert { gas_used: _, output } => {
                        println!("reverted with output: {:?}", output);
                        panic!("failed");
                    }
                    ExecutionResult::Halt { reason, gas_used: _ } => {
                        println!("halt with reason: {:?}", reason);
                        panic!("failed");
                    }
                };

                output
            });
            all_handlers.push(handler);
        }

        let rets = futures::future::join_all(all_handlers).await;
        Ok(rets)
    }
}

impl EvmSimulator {
    pub async fn new(
        ws_url: &str,
        target_pools: Option<Arc<DashMap<Address, DefiStorage>>>,
    ) -> Self {
        let ws = Ws::connect(ws_url).await.unwrap(); //     ws://localhost:8545
        let ws_provider = Provider::new(ws).interval(Duration::from_millis(100));
        let block_info = BlockInfo::default();
        let block_info = Arc::new(RwLock::new(block_info));

        EvmSimulator {
            ws_provider: Arc::new(ws_provider),
            target_pools,
            latest_block_info: block_info,
            fork_factory: None,
        }
    }

    // pub async fn fork_at(
    //     &self,
    //     block_number: u64,
    // ) -> Result<EvmFork, SimulationError<Provider<Ws>>> {
    //     match self.fork_factory {
    //         Some(ref f) => {
    //             let g = f.lock().await;
    //             if g.fork_block.as_u64() != block_number {
    //                 return Err(SimulationError::BlockNumberUnmatch(
    //                     g.fork_block.as_u64(),
    //                     block_number,
    //                 ));
    //             }
    //             let evm_fork = EvmFork::new(Arc::clone(f));
    //             Ok(evm_fork)
    //         }
    //         None => Err(SimulationError::ForkfactoryNotReady),
    //     }
    // }

    // Update latest block variable whenever we recieve a new block;
    // sync pool state
    //
    // Arguments:
    // * `block_info`: oracle to update
    pub async fn start_state_sync(&mut self, tx: Sender<EvmFork>) {
        let latest_block_clone = self.latest_block_info.clone();
        {
            let provider = self.ws_provider.clone();
            let target_pools = self.target_pools.clone();

            tokio::spawn(async move {
                loop {
                    let mut block_stream = if let Ok(stream) = provider.subscribe_blocks().await {
                        stream
                    } else {
                        panic!("Failed to create new block stream");
                    };

                    while let Some(block) = block_stream.next().await {
                        // lock the RwLock for write access and update the variable
                        {
                            let mut lock = latest_block_clone.write().await;
                            lock.number = block.number.unwrap();
                            lock.timestamp = block.timestamp;
                            // println!(
                            //     "new block number {}, timestamp {}",
                            //     lock.number, lock.timestamp
                            // );
                            if let Some(ref pools) = target_pools {
                                let cache_db = CacheDB::new(EmptyDB::default());
                                let fork_factory = ForkFactory::new_sandbox_factory(
                                    provider.clone(),
                                    cache_db,
                                    lock.number,
                                );
                                let fork_factory = Arc::new(RwLock::new(fork_factory));
                                let block_id = BlockId::Number(BlockNumber::Number(lock.number));
                                for entry in pools.iter() {
                                    let defi_storage = entry.value();

                                    let slot = get_v3_pool_slot_0(
                                        provider.clone(),
                                        defi_storage.pool_address,
                                        block_id,
                                    )
                                    .await
                                    .unwrap();
                                    let tick = get_tick_from_slot_value(slot);
                                    let _ = prepare_pool(
                                        provider.clone(),
                                        defi_storage,
                                        fork_factory.clone(),
                                        tick,
                                        Some(block_id),
                                    )
                                    .await;
                                }
                                let evm_fork = EvmFork::new(fork_factory);
                                let _ = tx.send(evm_fork).await;
                            }
                        } // remove write lock due to being out of scope here
                    }
                }
            });
        }
    }

    pub async fn replay_transaction(
        &self,
        tx_hash: TxHash,
    ) -> Result<ReplayTransactionResult, SimulationError<Provider<Ws>>> {
        let tx = self
            .ws_provider
            .get_transaction(tx_hash)
            .await?
            .ok_or(SimulationError::TransactionNotFound(tx_hash))?;

        if tx.block_number.is_none() {
            return Err(SimulationError::TransactionBlkNumberNotFound);
        }

        let tx_block_number = tx.block_number.unwrap().as_u64();

        let cache_db = CacheDB::new(EmptyDB::default());
        let fork_factory = ForkFactory::new_sandbox_factory(
            self.ws_provider.clone(),
            cache_db,
            (tx_block_number - 1).into(),
        );

        let mut evm = revm::EVM::new();
        let fork_db = fork_factory.new_sandbox_fork();
        evm.database(fork_db);

        evm.env.block.number = rU256::from(tx_block_number);
        let block = self.ws_provider.get_block_with_txs(tx_block_number).await?;

        if let Some(ref block) = block {
            evm.env.block.timestamp = block.timestamp.into();
            evm.env.block.coinbase = block.author.unwrap_or_default().into();
            evm.env.block.difficulty = block.difficulty.into();
            evm.env.block.prevrandao = block.mix_hash.map(h256_to_b256);
            evm.env.block.basefee = block.base_fee_per_gas.unwrap_or_default().into();
            evm.env.block.gas_limit = block.gas_limit.into();
        }

        // execure front txs
        if let Some(block) = block {
            for (_, tx) in block.transactions.into_iter().enumerate() {
                // arbitrum L1 transaction at the start of every block that has gas price 0
                // and gas limit 0 which causes reverts, so we skip it
                if tx.from == ARBITRUM_SENDER {
                    continue;
                }
                if tx.hash().eq(&tx_hash) {
                    break;
                }

                configure_tx_env(&mut evm.env, &tx);
                let inspector = NoOpInspector {};
                let _run_result = match evm.inspect_commit(inspector) {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("simulate error for other tx {:?},{:?}", tx.hash, e);
                        return Err(SimulationError::SimulationEvmOtherTxError(format!(
                            "{:?},{:?}",
                            tx.hash, e
                        )));
                    }
                };
            }
        }

        // execure target tx
        configure_tx_env(&mut evm.env, &tx);
        if let Some(_to) = tx.to {
            // println!("executing call transaction");
            let inspector = NoOpInspector {};
            let run_result = match evm.inspect_commit(inspector) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("simulate error for target tx {:?},{:?}", tx.hash, e);
                    return Err(SimulationError::SimulationEvmError(format!(
                        "{:?},{:?}",
                        tx.hash, e
                    )));
                }
            };
            // println!("result: {:?}", run_result);
            let replay_ret = match run_result {
                ExecutionResult::Success { gas_used, gas_refunded, output, .. } => match output {
                    Output::Call(o) => ReplayTransactionResult::Success {
                        gas_used,
                        gas_refunded,
                        output: o.clone(),
                    },
                    Output::Create(_o, _) => unimplemented!(),
                },
                ExecutionResult::Revert { gas_used, output } => {
                    // println!("reverted with output: {:?}", output);
                    let ret = decode_revert(&output, None, None);
                    match ret {
                        Ok(r) => ReplayTransactionResult::Revert { gas_used, message: r },
                        Err(e) => {
                            eprintln!("error: {:?}", e);
                            return Err(SimulationError::DecodeRevertMsgError);
                        }
                    }
                }
                ExecutionResult::Halt { reason: _, gas_used: _ } => unimplemented!(),
            };
            return Ok(replay_ret);
        }
        Err(SimulationError::UnableToReplay(tx_hash))
    }

    /// check whether target tx interatcted with watched pools
    /// # Args
    /// @param(victim_hash): txhash of the transaction to be traced
    ///
    /// # Return
    /// @return(DiffMode): state diff of applying the victim tx
    ///
    /// # Panic
    /// panic if there is trace error, or could not find the state diff
    // pub async fn analyze_target_tx_with_geth(
    //     &self,
    //     victim_tx: Transaction,
    //     block_id: BlockId,
    // ) -> Result<Option<Vec<SimPool>>, AnalyzeError> {
    //     if self.target_pools.is_none() {
    //         return Err(AnalyzeError::MustProvidePoolError);
    //     }

    //     let ret = self.debug_trace_call(victim_tx, block_id).await;
    //     match ret {
    //         Ok(state_diff) => {
    //             let sim = extract_pools_for_geth(
    //                 &state_diff,
    //                 self.weth_address,
    //                 &self.target_pools.clone().unwrap(),
    //             );
    //             Ok(sim)
    //         }
    //         Err(e) => Err(AnalyzeError::SimulateError(e)),
    //     }
    // }

    // pub async fn analyze_target_tx_with_erigon(&self, victim: Transaction) -> Option<Vec<SimPool>> {
    //     if let Some(pools) = self.target_pools.clone() {
    //         // get all state diffs that this tx produces
    //         if let Some(sd) = self.trace_call_many(vec![victim.clone()], BlockNumber::Latest).await
    //         {
    //             let sim = extract_pools(&sd, self.weth_address, &pools);
    //             return sim;
    //         }
    //     }
    //     panic!("unable to identify");
    // }

    // pub async fn debug_trace_transaction_call_tracer(
    //     &self,
    //     victim_hash: TxHash,
    // ) -> anyhow::Result<CallFrame> {
    //     let mut option = GethDebugTracingOptions::default();
    //     option.tracer =
    //         Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::CallTracer));
    //     option.tracer_config =
    //         Some(GethDebugTracerConfig::BuiltInTracer(GethDebugBuiltInTracerConfig::CallTracer(
    //             CallConfig { only_top_call: Some(false), with_log: Some(true) },
    //         )));
    //     let ret = self.ws_provider.debug_trace_transaction(victim_hash, option).await;
    //     match ret {
    //         Ok(trace) => {
    //             if let GethTrace::Known(GethTraceFrame::CallTracer(frame)) = trace {
    //                 return Ok(frame);
    //             };
    //             Err(SimulationError::NodeTracingNotGetStateDiffError(victim_hash.to_string()))
    //         }
    //         Err(err) => Err(SimulationError::NodeTracingProviderError(err)),
    //     }
    // }

    /// trace transaction with geth
    /// # Args
    /// @param(victim_hash): txhash of the transaction to be traced
    ///
    /// # Return
    /// @return(DiffMode): state diff of applying the victim tx
    // pub async fn debug_trace_transaction_state_tracer(
    //     &self,
    //     victim_hash: TxHash,
    // ) -> anyhow::Result<DiffMode> {
    //     let mut option = GethDebugTracingOptions::default();
    //     option.tracer =
    //         Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::PreStateTracer));
    //     option.tracer_config = Some(GethDebugTracerConfig::BuiltInTracer(
    //         GethDebugBuiltInTracerConfig::PreStateTracer(PreStateConfig { diff_mode: Some(true) }),
    //     ));
    //     let ret = self.ws_provider.debug_trace_transaction(victim_hash, option).await;
    //     match ret {
    //         Ok(trace) => {
    //             if let GethTrace::Known(GethTraceFrame::PreStateTracer(PreStateFrame::Diff(diff))) =
    //                 trace
    //             {
    //                 return Ok(diff);
    //             };
    //             Err(SimulationError::NodeTracingNotGetStateDiffError(victim_hash.to_string()))
    //         }
    //         Err(err) => Err(SimulationError::NodeTracingProviderError(err)),
    //     }
    // }

    // pub async fn debug_trace_call_storage(
    //     &self,
    //     target_tx: TypedTransaction,
    //     block_id: BlockId,
    // ) -> anyhow::Result<DefaultFrame> {
    //     // let hash = target_tx.hash;
    //     // let victim_typed_tx = to_typed_transaction(target_tx);
    //     let mut tracing_option = GethDebugTracingOptions::default();
    //     // tracing_option.tracer = None,
    //     tracing_option.disable_storage = Some(false);
    //     // tracing_option.tracer_config = Some(GethDebugTracerConfig::BuiltInTracer(
    //     //     GethDebugBuiltInTracerConfig::PreStateTracer(PreStateConfig {
    //     //         diff_mode: Some(true),
    //     //     }),
    //     // ));

    //     let mut option = GethDebugTracingCallOptions::default();
    //     option.tracing_options = tracing_option;
    //     let ret = self.ws_provider.debug_trace_call(target_tx, Some(block_id), option).await;
    //     match ret {
    //         Ok(trace) => {
    //             if let GethTrace::Known(GethTraceFrame::Default(frame)) = trace {
    //                 return Ok(frame);
    //             };
    //             Err(SimulationError::NodeTracingNotGetStateDiffError("HELLO".to_string()))
    //         }
    //         Err(err) => Err(SimulationError::NodeTracingProviderError(err)),
    //     }
    // }
    // pub async fn debug_trace_call(
    //     &self,
    //     victim_tx: Transaction,
    //     block_id: BlockId,
    // ) -> anyhow::Result<DiffMode> {
    //     let hash = victim_tx.hash;
    //     let victim_typed_tx = to_typed_transaction(victim_tx);
    //     let mut tracing_option = GethDebugTracingOptions::default();
    //     tracing_option.tracer =
    //         Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::PreStateTracer));
    //     tracing_option.tracer_config = Some(GethDebugTracerConfig::BuiltInTracer(
    //         GethDebugBuiltInTracerConfig::PreStateTracer(PreStateConfig { diff_mode: Some(true) }),
    //     ));

    //     let mut option = GethDebugTracingCallOptions::default();
    //     option.tracing_options = tracing_option;
    //     let ret = self.ws_provider.debug_trace_call(victim_typed_tx, Some(block_id), option).await;
    //     match ret {
    //         Ok(trace) => {
    //             if let GethTrace::Known(GethTraceFrame::PreStateTracer(PreStateFrame::Diff(diff))) =
    //                 trace
    //             {
    //                 return Ok(diff);
    //             };
    //             Err(SimulationError::NodeTracingNotGetStateDiffError(hash.to_owned().to_string()))
    //         }
    //         Err(err) => Err(SimulationError::NodeTracingProviderError(err)),
    //     }
    // }

    // pub async fn trace_call_many_txhash(
    //     &self,
    //     victim_hashs: Vec<TxHash>,
    //     block_number: BlockNumber,
    // ) -> Option<BTreeMap<Address, AccountDiff>> {
    //     let victim_txs = self.get_txs(victim_hashs).await;
    //     return self.trace_call_many(victim_txs, block_number).await;
    // }

    // pub async fn trace_call_many(
    //     &self,
    //     victim_txs: Vec<Transaction>,
    //     block_number: BlockNumber,
    // ) -> Option<BTreeMap<Address, AccountDiff>> {
    //     let req = victim_txs.iter().map(|tx| (tx, vec![TraceType::StateDiff])).collect();
    //     let block_traces = match self.ws_provider.trace_call_many(req, Some(block_number)).await {
    //         Ok(x) => x,
    //         Err(err) => panic!("client trace call error {:?}", err),
    //     };

    //     let mut merged_state_diffs = BTreeMap::new();

    //     block_traces
    //         .into_iter()
    //         .flat_map(|bt| bt.state_diff.map(|sd| sd.0.into_iter()))
    //         .flatten()
    //         .for_each(|(address, account_diff)| {
    //             match merged_state_diffs.entry(address) {
    //                 Entry::Vacant(entry) => {
    //                     entry.insert(account_diff);
    //                 }
    //                 Entry::Occupied(_) => {
    //                     // Do nothing if the key already exists
    //                     // we only care abt the starting state
    //                 }
    //             }
    //         });
    //     return Some(merged_state_diffs);
    // }

    async fn get_txs(&self, victim_tx_hash: Vec<TxHash>) -> Vec<Transaction> {
        let mut tx_futures = vec![];
        for hash in victim_tx_hash {
            tx_futures.push(self.ws_provider.get_transaction(hash));
        }

        let txs = join_all(tx_futures)
            .await
            .iter()
            .map(|x| match x {
                Ok(ref tx) => tx.clone().unwrap(),
                Err(err) => panic!("error in get tx {:?}", err),
            })
            .collect::<Vec<_>>();
        txs
    }
}

fn to_typed_transaction(tx: Transaction) -> TypedTransaction {
    let tx_type = tx.transaction_type.unwrap().as_u64();
    if tx_type == 2u64 {
        // let typed_tx: Eip1559TransactionRequest = ;
        return TypedTransaction::Eip1559((&tx).into());
    }

    if tx_type == 0u64 {
        return TypedTransaction::Legacy((&tx).into());
    }
    panic!("to be supported: {:?}", tx_type);
}

// #[allow(dead_code)]
// fn extract_pools_for_geth(
//     state_diffs: &DiffMode,
//     weth_address: Address,
//     all_pools: &DashMap<Address, DefiStorage>,
// ) -> Option<Vec<SimPool>> {
//     let touched_pools: Vec<DefiStorage> = state_diffs
//         .pre
//         .keys()
//         .filter_map(|e| all_pools.get(e).map(|p| (*p.value()).clone()))
//         .collect();

//     let mut sandwichable_pools: Vec<SimPool> = vec![];

//     let weth_state_pre = state_diffs.pre.get(&weth_address);
//     let weth_state_post = state_diffs.post.get(&weth_address);

//     // find storage mapping index for each pool
//     for pool in touched_pools {
//         // find mapping storage location
//         let storage_key = TxHash::from(ethers::utils::keccak256(abi::encode(&[
//             abi::Token::Address(pool.pool_address),
//             abi::Token::Uint(U256::from(3)),
//         ])));
//         let mut weth_change = None;

//         let weth_storage_pre = weth_state_pre.map_or(None, |account_state| {
//             account_state.storage.as_ref().map_or(None, |storage| {
//                 storage.get(&storage_key).map_or(None, |balance| Some(balance))
//             })
//         });

//         let weth_storage_post = weth_state_post.map_or(None, |account_state| {
//             account_state.storage.as_ref().map_or(None, |storage| {
//                 storage.get(&storage_key).map_or(None, |balance| Some(balance))
//             })
//         });

//         if weth_state_pre.is_some() {
//             let pre = U256::from(weth_storage_pre.unwrap().to_fixed_bytes());
//             let post = weth_storage_post.map_or(U256::from(0), |x| U256::from(x.to_fixed_bytes()));
//             weth_change = Some(BalanceChange { pre, post })
//         }

//         sandwichable_pools.push(SimPool::new(pool, weth_change));
//     }

//     Some(sandwichable_pools)
// }

#[allow(dead_code)]
fn extract_pools(
    state_diffs: &BTreeMap<Address, AccountDiff>,
    weth_address: Address,
    all_pools: &DashMap<Address, DefiStorage>,
) -> Option<Vec<SimPool>> {
    // capture all addresses that have a state change and are also a pool
    let touched_pools: Vec<DefiStorage> =
        state_diffs.keys().filter_map(|e| all_pools.get(e).map(|p| (*p.value()).clone())).collect();

    // find direction of swap based on state diff (does weth have state changes?)
    let weth_state = &state_diffs.get(&weth_address);

    let mut sandwichable_pools: Vec<SimPool> = vec![];

    // find storage mapping index for each pool
    for pool in touched_pools {
        // find mapping storage location
        let storage_key = TxHash::from(ethers::utils::keccak256(abi::encode(&[
            abi::Token::Address(pool.pool_address),
            abi::Token::Uint(U256::from(3)),
        ])));
        let mut weth_change = None;
        if let Some(weth_state_diff) = weth_state {
            weth_change = match weth_state_diff.storage.get(&storage_key)? {
                Diff::Changed(c) => {
                    let from = U256::from(c.from.to_fixed_bytes());
                    let to = U256::from(c.to.to_fixed_bytes());
                    Some(BalanceChange { pre: from, post: to })
                }
                _ => continue,
            };
        }

        sandwichable_pools.push(SimPool::new(pool, weth_change));
    }

    Some(sandwichable_pools)
}
