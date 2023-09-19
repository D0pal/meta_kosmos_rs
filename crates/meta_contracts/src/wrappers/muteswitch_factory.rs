use crate::bindings::muteswitchfactory::MuteSwitchFactory;
use ethers::prelude::*;
use meta_common::enums::{DexExchange, Network};
use std::sync::Arc;

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

    pub async fn get_pair_addr(&self, token_a: Address, token_b: Address) -> Address {
        (self.factory_contract.get_pair(token_a, token_b, false).call().await).unwrap()
    }
}
