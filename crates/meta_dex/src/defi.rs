use ethers::prelude::*;
use std::{
    borrow::BorrowMut,
    cell::{RefCell, UnsafeCell},
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::RwLock;

use meta_address::get_dex_address;
use meta_common::enums::{ContractType, DexExchange, Network, PoolVariant};
use meta_contracts::bindings::{
    quoterv2::QuoterV2, swaprouter::SwapRouter, uniswapv3factory::UniswapV3Factory,
    uniswapv3pool::UniswapV3Pool,
};

#[derive(Debug, Clone)]
pub struct UniV3Contracts<M> {
    pub factory: UniswapV3Factory<M>,
    pub quoter_v2: Arc<RwLock<QuoterV2<M>>>,
    pub swap_router: Arc<RwLock<SwapRouter<M>>>,
    pub pools: Arc<RwLock<HashMap<Address, HashMap<Address, Arc<UniswapV3Pool<M>>>>>>,
}

#[derive(Clone, Debug)]
pub struct DexWrapper<M> {
    pub client: Arc<M>,
    pub network: Network,
    pub dex: DexExchange,
    pub pool_variant: PoolVariant,
    pub v3_contracts: Option<UniV3Contracts<M>>,
}

impl<M: Middleware> DexWrapper<M> {
    /// # Description
    /// Creates a new dex instance
    pub fn new(client: Arc<M>, network: Network, dex_exchange: DexExchange) -> Self {
        let (pool_variant, factory) = match dex_exchange {
            DexExchange::UniswapV3 => {
                let factory_contract_info =
                    get_dex_address(dex_exchange, network, ContractType::UniV3Factory).unwrap();
                let quoter_v2_contract_info =
                    get_dex_address(dex_exchange, network, ContractType::UniV3QuoterV2).unwrap();
                let swap_router_info =
                    get_dex_address(dex_exchange, network, ContractType::UniV3SwapRouterV2)
                        .unwrap();
                let v3_factory =
                    UniswapV3Factory::new(factory_contract_info.address, client.clone());
                let v3_quoter_v2 = QuoterV2::new(quoter_v2_contract_info.address, client.clone());
                let swap_router = SwapRouter::new(swap_router_info.address, client.clone());
                (
                    PoolVariant::UniswapV3,
                    UniV3Contracts {
                        factory: v3_factory,
                        quoter_v2: Arc::new(RwLock::new(v3_quoter_v2)),
                        swap_router: Arc::new(RwLock::new(swap_router)),
                        pools: Arc::new(RwLock::new(HashMap::new())),
                    },
                )
            }
            _ => unimplemented!(),
        };

        Self {
            client: client.clone(),
            network,
            dex: dex_exchange,
            pool_variant,
            v3_contracts: Some(factory),
        }
    }

    #[allow(dead_code)]
    fn get_factoy_address(&self) -> Address {
        let contract_type = match self.pool_variant {
            PoolVariant::UniswapV3 => ContractType::UniV3Factory,
            _ => ContractType::UniV2Factory,
        };
        let factory_contract_info = get_dex_address(self.dex, self.network, contract_type).unwrap();
        factory_contract_info.address
    }

    #[allow(dead_code)]
    fn get_v3_factory(&self) -> UniswapV3Factory<M> {
        let factory_address = self.get_factoy_address();

        UniswapV3Factory::new(factory_address, self.client.clone())
    }

    pub async fn get_v3_pool(
        &self,
        token_0: Address,
        token_1: Address,
        fee: u32,
    ) -> anyhow::Result<Arc<UniswapV3Pool<M>>> {
        if let Some(ref v3_contract) = self.v3_contracts {
            {
                let _g = v3_contract.pools.read().await;
                _g.get(&token_0).and_then(|inner| {
                    inner.get_key_value(&token_1).map(|x| {
                        x.1.clone()
                    })
                })
            };
            {
                let mut _g = v3_contract.pools.write().await;
                let address = v3_contract.factory.get_pool(token_0, token_1, fee).call().await;
                match address {
                    Ok(addr) => {
                        let pool = Arc::new(UniswapV3Pool::new(addr, self.client.clone()));
                        let ret = _g
                            .entry(token_0)
                            .or_insert(HashMap::new())
                            .entry(token_1)
                            .or_insert(pool);
                        Ok(ret.clone())
                    }
                    Err(_e) => {
                        todo!()
                    }
                }
            }
        } else {
            todo!()
        }
    }

    pub async fn get_v3_quoter(&self) -> anyhow::Result<QuoterV2<M>> {
        if let Some(ref v3_contract) = self.v3_contracts {
            {
                let _g = v3_contract.quoter_v2.read().await;
                let a = _g.clone();
                Ok(a)
            }
        } else {
            todo!()
        }
    }

    pub async fn get_v3_swap_router(&self) -> anyhow::Result<SwapRouter<M>> {
        if let Some(ref v3_contract) = self.v3_contracts {
            {
                let _g = v3_contract.swap_router.read().await;
                let a = _g.clone();
                Ok(a)
            }
        } else {
            todo!()
        }
    }
}
