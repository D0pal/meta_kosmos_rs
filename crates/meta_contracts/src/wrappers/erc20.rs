use crate::{
    bindings::{
        uniswap_v2_factory::UniswapV2Factory,
        uniswap_v2_pair::UniswapV2Pair,
        ERC20
    }, 
    // chain_book::{Dex}
};
use ethers::prelude::*;
use std::sync::Arc;
use futures::future::join_all;
use meta_common::enums::{Network};

pub struct Erc20Wrapper<M> {
    // pub config_contract: Config<M>,
    pub network: Network,
    // pub token_address: UniswapV2Factory<M>,
    pub token_contract: ERC20<M>,
    pub client: Arc<M>,
    pub decimals: Option<u8>,
    pub name: String,
}

impl<M: Middleware> Erc20Wrapper<M> {
    pub async fn new(network: Network, token_address: Address, client: Arc<M>) -> Self {
        // let config_contract = Config::new(config_address, client.clone());
        let token_contract = ERC20::new(token_address, client.clone());
        let mut erc20= Erc20Wrapper { network, token_contract, client, decimals: None, name: String::new()};
        erc20.get_or_fetch_name().await;
        erc20.get_or_fetch_decimals().await;
        erc20
    }

    pub async fn get_or_fetch_decimals(&mut self) -> u8 {
        match(self.decimals) {
            Some(num) => {
                num
            },
            None => {
               let decimal_num = self.token_contract.decimals().call().await.unwrap();
               self.decimals = Some(decimal_num);
               decimal_num
            }
        }
    }

    pub async fn get_or_fetch_name(&mut self) -> String {
        if self.name.is_empty() {
            let name = self.token_contract.name().call().await.unwrap();
            self.name = name;
        }
        self.name.to_owned()
    }
}