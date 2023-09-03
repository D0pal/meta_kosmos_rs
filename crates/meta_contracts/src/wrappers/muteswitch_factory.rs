use super::Erc20Wrapper;
use crate::bindings::{
    mute_switch_factory::MuteSwitchFactory, uniswap_v2_pair::UniswapV2Pair,
    uniswap_v2_router_02::UniswapV2Router02, ERC20,
};
// use core::num;
use ethers::prelude::*;
use futures::future::join_all;
use meta_common::{
    constants::ZERO_ADDRESS,
    enums::{DexExchange, Network},
};
use meta_util::ether::{address_from_str, address_to_str};

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    ops::{Add, Div, Mul},
    rc::Rc,
    sync::Arc,
};
use tracing::{debug, error, info};

pub struct MuteSwitchFactoryWrapper<M> {
    // pub config_contract: Config<M>,
    pub network: Network,
    pub dex: DexExchange,
    pub factory_contract: MuteSwitchFactory<M>,
    pub client: Arc<M>,
}

impl<M: Middleware> MuteSwitchFactoryWrapper<M> {
    pub fn new(
        network: Network,
        dex: DexExchange,
        factory_address: Address,
        client: Arc<M>,
    ) -> Self {
        // let config_contract = Config::new(config_address, client.clone());
        let factory_contract = MuteSwitchFactory::new(factory_address, client.clone());
        MuteSwitchFactoryWrapper { network, dex, factory_contract, client }
    }

    pub async fn get_pair_addr(&self, tokenA: Address, tokenB: Address) -> Address {
        let contract_addr =
            (self.factory_contract.get_pair(tokenA, tokenB, false).call().await).unwrap();
        contract_addr
    }
}
