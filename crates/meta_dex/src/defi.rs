use ethers::prelude::*;
use std::{cell::RefCell, collections::HashMap, sync::Arc};

use meta_address::get_dex_address;
use meta_common::enums::{ContractType, DexExchange, Network, PoolVariant};
use meta_contracts::bindings::{
    quoter_v2::QuoterV2, swap_router::SwapRouter, uniswap_v3_factory::UniswapV3Factory,
    uniswap_v3_pool::UniswapV3Pool,
};

#[derive(Debug, Clone)]
pub struct UniV3Contracts<M> {
    pub factory: UniswapV3Factory<M>,
    pub quoter_v2: RefCell<QuoterV2<M>>,
    pub swap_router: RefCell<SwapRouter<M>>,
    pub pools: HashMap<Address, HashMap<Address, RefCell<UniswapV3Pool<M>>>>,
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
                    get_dex_address(dex_exchange, network, ContractType::UniV3SwapRouterV2).unwrap();
                let v3_factory =
                    UniswapV3Factory::new(factory_contract_info.address, client.clone());
                let v3_quoter_v2 = QuoterV2::new(quoter_v2_contract_info.address, client.clone());
                let swap_router = SwapRouter::new(swap_router_info.address, client.clone());
                (
                    PoolVariant::UniswapV3,
                    UniV3Contracts {
                        factory: v3_factory,
                        quoter_v2: RefCell::new(v3_quoter_v2),
                        swap_router: RefCell::new(swap_router),
                        pools: HashMap::new(),
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
        let v3_factory = UniswapV3Factory::new(factory_address, self.client.clone());
        v3_factory
    }

    pub async fn get_v3_pool(
        &self,
        token_0: Address,
        token_1: Address,
        fee: u32,
    ) -> anyhow::Result<&UniswapV3Pool<M>> {
        let _this = self as *const Self as *mut Self;
        unsafe { _this.as_mut().unwrap().get_v3_pool_mut(token_0, token_1, fee).await }
    }

    async fn get_v3_pool_mut(
        &mut self,
        token_0: Address,
        token_1: Address,
        fee: u32,
    ) -> anyhow::Result<&UniswapV3Pool<M>> {
        let pool = self.v3_contracts.as_ref().map_or(None, |contracts| {
            contracts
                .pools
                .get(&token_0)
                .map_or(None, |inner| inner.get_key_value(&token_1).map(|x| x.1))
        });

        if pool.is_some() {
            let ret = unsafe { pool.unwrap().as_ptr().as_ref() };
            return Ok(ret.unwrap());
        }
        if let Some(ref mut v3_contract) = self.v3_contracts {
            let address = v3_contract.factory.get_pool(token_0, token_1, fee).call().await;
            match address {
                Ok(addr) => {
                    let pool = RefCell::new(UniswapV3Pool::new(addr, self.client.clone()));
                    let ret = v3_contract
                        .pools
                        .entry(token_0)
                        .or_insert(HashMap::new())
                        .entry(token_1)
                        .or_insert(pool);
                    let ret = unsafe { ret.as_ptr().as_ref().unwrap() };
                    return Ok(ret);
                }
                Err(_e) => {
                    todo!()
                }
            }
        } else {
            todo!()
        }
    }

    pub fn get_v3_quoter(&self) -> anyhow::Result<&QuoterV2<M>> {
        let quoter = self.v3_contracts.as_ref().map(|c| &c.quoter_v2);
        if quoter.is_some() {
            let ret = unsafe { quoter.unwrap().as_ptr().as_ref() };
            return Ok(ret.unwrap());
        }
        let quoter_address = get_dex_address(self.dex, self.network, ContractType::UniV3QuoterV2)
            .expect("quoter address not found");
        let quoter = QuoterV2::new(quoter_address.address, self.client.clone());
        self.v3_contracts.as_ref().map(|x| {
            let borrowed = &mut (*x.quoter_v2.borrow_mut());
            unsafe { *(borrowed as *mut QuoterV2<M>) = quoter };
        });
        let ret =
            unsafe { self.v3_contracts.as_ref().unwrap().quoter_v2.as_ptr().as_ref().unwrap() };
        return Ok(ret);
    }

    pub fn get_v3_swap_router(&self) -> anyhow::Result<&SwapRouter<M>> {
        let swap_router = self.v3_contracts.as_ref().map(|c| &c.swap_router);
        if swap_router.is_some() {
            let ret = unsafe { swap_router.unwrap().as_ptr().as_ref() };
            return Ok(ret.unwrap());
        }

        let swap_router_address =
            get_dex_address(self.dex, self.network, ContractType::UniV3SwapRouterV2)
                .expect("swap router address not found");
        let router = SwapRouter::new(swap_router_address.address, self.client.clone());
        self.v3_contracts.as_ref().map(|x| {
            let borrowed = &mut (*x.swap_router.borrow_mut());
            unsafe { *(borrowed as *mut SwapRouter<M>) = router };
        });
        let ret =
            unsafe { self.v3_contracts.as_ref().unwrap().swap_router.as_ptr().as_ref().unwrap() };
        return Ok(ret);
    }

    // pub fn quote_v3_exact_input_single() {
    //     let params = QuoteExactInputSingleParams {
    //         token_in: weth,
    //         token_out: usdc,
    //         amount_in: decimal_to_wei(Decimal::from_f64(0.1f64).unwrap(), 18),
    //         fee: 500,
    //         sqrt_price_limit_x96: U256::from(0),
    //     };
    // }
}
