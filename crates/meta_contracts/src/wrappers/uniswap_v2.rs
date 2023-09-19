use super::Erc20Wrapper;
use crate::bindings::{
    flashbotsrouter::UniswapWethParams, muteswitchfactory::MuteSwitchFactory,
    uniswapv2factory::UniswapV2Factory, uniswapv2pair::UniswapV2Pair,
    uniswapv2router02::UniswapV2Router02, erc20::ERC20,
};
use ethers::prelude::*;
use futures::future::join_all;
use meta_common::enums::{DexExchange, Network};
use meta_util::ether::address_to_str;

use std::{
    cell::RefCell,
    ops::{Div, Mul},
    rc::Rc,
    sync::Arc,
};
use tracing::{debug, error, info};

pub enum UniswapV2FactoryEnum<M> {
    UniswapV2(UniswapV2Factory<M>),
    MuteSwitch(MuteSwitchFactory<M>),
}

pub struct UniswapV2<M> {
    // pub config_contract: Config<M>,
    pub network: Network,
    pub dex: DexExchange,
    pub factory_contract: UniswapV2FactoryEnum<M>,
    // pub swap_router_contract: UniswapV2Router02<M>,
    pub client: Arc<M>,
}

impl<M: Middleware> UniswapV2<M> {
    pub fn new(
        network: Network,
        dex: DexExchange,
        factory_address: Address,
        swap_router_addr: Address,
        client: Arc<M>,
    ) -> Self {
        // let config_contract = Config::new(config_address, client.clone());
        debug!(
            "init UniswapV2 with network {} dex {} factory_address {}",
            network, dex, factory_address
        );
        let factory_contract = match dex {
            DexExchange::MuteSwitch => UniswapV2FactoryEnum::MuteSwitch(MuteSwitchFactory::new(
                factory_address,
                client.clone(),
            )),
            _ => UniswapV2FactoryEnum::UniswapV2(UniswapV2Factory::new(
                factory_address,
                client.clone(),
            )),
        };
        let _swap_router_contract = UniswapV2Router02::new(swap_router_addr, client.clone());
        UniswapV2 {
            network,
            dex,
            factory_contract,
            // swap_router_contract,
            client,
        }
    }

    pub fn get_factory_address(&self) -> Address {
        match &self.factory_contract {
            UniswapV2FactoryEnum::MuteSwitch(factory) => factory.address(),
            UniswapV2FactoryEnum::UniswapV2(factory) => factory.address(),
        }
    }

    /// get uniswap v2 pair contract address given by token0, token1. token0 < token1
    pub async fn get_pair_addr(&self, token0: Address, token1: Address) -> Address {
        match &self.factory_contract {
            UniswapV2FactoryEnum::MuteSwitch(factory) => {
                (factory).get_pair(token0, token1, false).call().await.unwrap()
            }
            UniswapV2FactoryEnum::UniswapV2(factory) => {
                (factory).get_pair(token0, token1).call().await.unwrap()
            }
        }
    }

    pub async fn get_pair_contract_wrapper(
        &self,
        token_a: Address,
        token_b: Address,
    ) -> UniswapV2PairWrapper<M> {
        debug!(
            "get_pair_contract_wrapper, dex {}, factory_address {} token_a {}, token_b: {} ",
            self.dex,
            self.get_factory_address(),
            address_to_str(&token_a),
            address_to_str(&token_b)
        );
        let contract_addr = self.get_pair_addr(token_a, token_b).await;
        debug!("got pair address: {:?}", contract_addr);
        let pair_contract = UniswapV2Pair::new(contract_addr, self.client.clone());
        // let (token_0, token_1) = get_token_0_and_token_1(token_a, token_b);
        UniswapV2PairWrapper::new(self.network, self.dex, pair_contract).await
    }

    // TODO: add cache to token addrs
    pub async fn get_pool_token_pair_addr(
        &self,
        pair_contract: &UniswapV2Pair<M>,
    ) -> (Address, Address) {
        let rets = join_all([pair_contract.token_0().call(), pair_contract.token_1().call()]).await;

        let token_0 = rets[0].as_ref().unwrap().to_owned();
        let token_1 = rets[1].as_ref().unwrap().to_owned();
        (token_0, token_1)
    }
}

#[derive(Clone, Debug)]
pub struct UniswapV2Reserves {
    pub reserve_0: U128,
    pub reserve_1: U128,
    pub block_ts_last: u32,
}

#[derive(Clone, Debug)]
pub struct Erc20Info {
    pub address: Address,
    pub decimals: u8,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct UniswapV2PairState {
    pub token_0: Erc20Info,
    pub token_1: Erc20Info,
    pub reserves: UniswapV2Reserves,
}

#[derive(Clone)]
pub struct UniswapV2PairWrapper<M> {
    pub network: Network,
    pub dex: DexExchange,
    pub state: UniswapV2PairState,
    pub pair_contract: UniswapV2Pair<M>,
    // pub client: Arc<M>,
}

impl<M: Middleware> UniswapV2PairWrapper<M> {
    async fn new(network: Network, dex: DexExchange, pair_contract: UniswapV2Pair<M>) -> Self {
        let tokens_rets =
            join_all([pair_contract.token_0().call(), pair_contract.token_1().call()]).await;
        let token_0_addr = tokens_rets[0].as_ref().unwrap().to_owned();
        let token_1_addr = tokens_rets[1].as_ref().unwrap().to_owned();
        let token_0_erc20 = ERC20::new(token_0_addr, pair_contract.client());
        let token_1_erc20 = ERC20::new(token_1_addr, pair_contract.client());

        let (reserve_0, reserve_1, block_ts_last) =
            pair_contract.get_reserves().call().await.expect("unable to fetch reserves");

        let name_rets = join_all([token_0_erc20.name().call(), token_1_erc20.name().call()]).await;

        let decimals_rets =
            join_all([token_0_erc20.decimals().call(), token_1_erc20.decimals().call()]).await;
        let state = UniswapV2PairState {
            token_0: Erc20Info {
                address: token_0_addr,
                decimals: decimals_rets[0].as_ref().unwrap().to_owned(),
                name: name_rets[0].as_ref().unwrap().to_owned(),
            },
            token_1: Erc20Info {
                address: token_1_addr,
                decimals: decimals_rets[1].as_ref().unwrap().to_owned(),
                name: name_rets[1].as_ref().unwrap().to_owned(),
            },
            reserves: UniswapV2Reserves {
                reserve_0: U128::from(reserve_0),
                reserve_1: U128::from(reserve_1),
                block_ts_last,
            },
        };
        UniswapV2PairWrapper { network, dex, state, pair_contract }
    }

    pub async fn get_price(&self, block_num: u64) -> Option<f64> {
        let ret = self.pair_contract.get_reserves().block(U64::from(block_num)).call().await;
        match ret {
            Ok((reserve_0, reserve_1, _ts)) => {
                debug!(
                    "reserve0: {:?}, reserve1: {:?}, token0: {:?}, token1: {:?}",
                    reserve_0, reserve_1, self.state.token_0, self.state.token_1
                );
                let price = reserve_1.mul(u128::pow(
                    10,
                    (self.state.token_0.decimals - self.state.token_1.decimals).into(),
                )) as f64
                    / reserve_0 as f64;
                debug!("got price: {:?}", price);
                Some(price)
            }
            Err(_) => None,
        }
    }

    pub async fn update_state_and_return_price(&mut self, block_num: &BlockId) -> Option<f64> {
        debug!("start get price for block number {:?}, state {:?}", block_num, self.state);
        let ret = self
            .pair_contract
            .get_reserves()
            // .block(*block_num)
            .call()
            .await;

        match ret {
            Ok((reserve_0, reserve_1, ts)) => {
                debug!("got reserve0: {:?}, reserve_1: {:?}", reserve_0, reserve_1);
                self.state.reserves = UniswapV2Reserves {
                    reserve_0: U128::from(reserve_0),
                    reserve_1: U128::from(reserve_1),
                    block_ts_last: ts,
                };
                let price = reserve_1 as f64 / reserve_0 as f64;
                debug!("got price: {:?}", price);
                Some(price)
            }
            Err(err) => {
                error!("error in fetching reserve {}", err);
                None
            }
        }
    }

    pub async fn update_reserves(&mut self) {
        let (reserve_0, reserve_1, block_ts_last) =
            self.pair_contract.get_reserves().call().await.expect("unable to fetch reserves");
        let new_reserves: UniswapV2Reserves = UniswapV2Reserves {
            reserve_0: U128::from(reserve_0),
            reserve_1: U128::from(reserve_1),
            block_ts_last,
        };
        self.state.reserves = new_reserves;
    }
    pub async fn get_storage_at(&self, from: Address, location: u64) -> Result<H256, M::Error> {
        self.pair_contract
            .client()
            .get_storage_at(from, H256::from_low_u64_le(location), None)
            .await
    }
    /// get swap tx data
    pub async fn get_swap_tx_data(
        &self,
        // pair_contract_wrapper: &UniswapV2PairWrapper<M>,
        token_in: Address,
        amount_in: U128,
        recipient: Address,
    ) -> Bytes {
        let amt_out = get_uni_v2_amt_out(&self.state, &token_in, amount_in);
        let mut call =
            self.pair_contract.swap(U256::from(0), U256::from(amt_out), recipient, vec![].into());

        if token_in.eq(&self.state.token_1.address) {
            call = self.pair_contract.swap(
                U256::from(amt_out),
                U256::from(0),
                recipient,
                vec![].into(),
            );
        }
        let data = call.tx.data();

        data.unwrap().to_owned()
    }
}

/// calculate amt out based on token in and amt in
pub fn get_uni_v2_amt_out(state: &UniswapV2PairState, token_in: &Address, amount_in: U128) -> U128 {
    let amt_in_with_fee = U256::from(amount_in) * U256::from(997);
    let mut numerator = amt_in_with_fee * U256::from(state.reserves.reserve_1);
    let mut denominator = U256::from(state.reserves.reserve_0) * U256::from(1000) + amt_in_with_fee;
    if token_in.eq(&state.token_1.address) {
        numerator = amt_in_with_fee * U256::from(state.reserves.reserve_0);
        denominator = U256::from(state.reserves.reserve_1) * U256::from(1000) + amt_in_with_fee;
    }
    U128::from((numerator / denominator).as_u128())
}

fn get_token_0_and_token_1(token_a: Address, token_b: Address) -> (Address, Address) {
    if token_a.lt(&token_b) {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    }
}

/**
 * (market_a_price - market_b_price)/ market_b_price * 1000
 * relative price diff in BP
 */
pub async fn calculate_price_diff<M: Middleware, T: Into<BlockId> + Clone>(
    market_a_contract_wrapper: Rc<RefCell<UniswapV2PairWrapper<M>>>,
    market_b_contract_wrapper: Rc<RefCell<UniswapV2PairWrapper<M>>>,
    block_num: T,
) -> f64 {
    // diff in BP
    let block_id: BlockId = block_num.to_owned().into();
    let rets = join_all([
        market_a_contract_wrapper.as_ref().borrow_mut().update_state_and_return_price(&block_id),
        market_b_contract_wrapper.as_ref().borrow_mut().update_state_and_return_price(&block_id),
    ])
    .await;

    let market_a_price = rets[0].as_ref();
    let market_b_price = rets[1].as_ref();
    debug!("market_a_price {:?}, market_b_price {:?}", market_a_price, market_b_price);

    if let (Some(cake_p), Some(swap_p)) = (market_a_price, market_b_price) {
        let price_diff_in_bp = (cake_p - swap_p) / swap_p * 10000f64;
        debug!(
            "block: {:?}, pancake_price: {}, biswap_price: {:?}, price_diff: {:?}",
            block_num.into(),
            cake_p,
            swap_p,
            price_diff_in_bp
        );
        return price_diff_in_bp;
    }
    0f64
}

/**
 * price_diff: token1/token0 in market_a - token1/token0 in market_b
 */
pub async fn get_atomic_arb_call_params<M: Middleware>(
    market_a: Rc<RefCell<UniswapV2PairWrapper<M>>>,
    market_b: Rc<RefCell<UniswapV2PairWrapper<M>>>,
    price_diff: f64,
    quote_asset: &Erc20Wrapper<M>,
    quote_amt_in: u128,
    base_asset: &Erc20Wrapper<M>,
    receiver: Address,
) -> UniswapWethParams {
    let mut market_first = market_a.clone();
    let mut market_second = market_b.clone();

    // token0 = quote, token1 = base, and quote asset is cheaper in market a
    if quote_asset
        .token_contract
        .address()
        .lt(&base_asset.token_contract.address())
        && price_diff.is_sign_negative()
        ||
        // token0 = base, token1 = quote, and quote asset is cheaper in market a 
        quote_asset
            .token_contract
            .address()
            .gt(&base_asset.token_contract.address())
            && price_diff.is_sign_positive()
    {
        market_first = market_b.clone();
        market_second = market_a.clone();
    }

    let market_first_addr = market_first.as_ref().borrow().pair_contract.address();
    let market_second_addr = market_second.as_ref().borrow().pair_contract.address();

    info!(
        "sell {:?} at {:?} (pool addr {:?}) for {:?}, and resell at {:?} (pool addr {:?}) ",
        quote_asset.name,
        market_first.as_ref().borrow().dex,
        market_first_addr,
        base_asset.name,
        market_second.as_ref().borrow().dex,
        market_second_addr
    );
    let mut market_first_base_amt_out = get_uni_v2_amt_out(
        &market_first.as_ref().borrow().state,
        &quote_asset.token_contract.address(),
        U128::from(quote_amt_in),
    );
    market_first_base_amt_out = U128::from(market_first_base_amt_out.as_u128().mul(95).div(100));
    debug!(
        "{:?} amount out after first market swap : {:?}",
        base_asset.name, market_first_base_amt_out
    );

    // base asset is token 1
    let mut amt_0_out = U256::from(0);
    let mut amt_1_out = U256::from(market_first_base_amt_out);

    // base asset is token 0
    if base_asset.token_contract.address().eq(&market_first.as_ref().borrow().state.token_0.address)
    {
        amt_0_out = U256::from(market_first_base_amt_out);
        amt_1_out = U256::from(0);
    }
    debug!("market_first amt_0_out {:?}, amt_1_out {:?} ", amt_0_out, amt_1_out);
    let market_first_swap_call = market_first
        .as_ref()
        .borrow()
        .pair_contract
        .swap(
            amt_0_out,
            amt_1_out,
            market_second.as_ref().borrow().pair_contract.address(),
            vec![].into(),
        )
        .tx
        .data()
        .unwrap()
        .to_owned();

    let mut market_second_quote_amt_out = get_uni_v2_amt_out(
        &market_second.as_ref().borrow().state,
        &base_asset.token_contract.address(),
        market_first_base_amt_out,
    );
    market_second_quote_amt_out =
        U128::from(market_second_quote_amt_out.as_u128().mul(95).div(100));

    debug!(
        "{:?} amount out after send market swap : {:?}",
        quote_asset.name, market_second_quote_amt_out
    );

    // quote asset is token 0
    let mut market_second_amt_0_out = U256::from(market_second_quote_amt_out);
    let mut market_second_amt_1_out = U256::from(0);

    // market second quote asset is token 1
    if quote_asset.token_contract.address().eq(&market_second
        .as_ref()
        .borrow()
        .state
        .token_1
        .address)
    {
        market_second_amt_0_out = U256::from(0);
        market_second_amt_1_out = U256::from(market_second_quote_amt_out);
    }
    debug!(
        "market_second amt_0_out {}, market_second_amt_1_out {}",
        market_second_amt_0_out, market_second_amt_1_out
    );
    let market_second_swap_call = market_second
        .as_ref()
        .borrow()
        .pair_contract
        .swap(market_second_amt_0_out, market_second_amt_1_out, receiver, vec![].into())
        .tx
        .data()
        .unwrap()
        .to_owned();

    UniswapWethParams {
        weth_amount_to_first_market: U256::from(quote_amt_in),
        eth_amount_to_coinbase: U256::from(0),
        targets: vec![market_first_addr, market_second_addr],
        payloads: vec![market_first_swap_call, market_second_swap_call],
    }
}
#[cfg(test)]
mod test {
    use super::{get_uni_v2_amt_out, UniswapV2PairState, UniswapV2Reserves};
    // use crate::{common::ETHER, dex_wrappers::Erc20Info};
    use super::Erc20Info;
    use ethers::prelude::*;
    use meta_common::constants::ETHER;
    use mockall::mock;
    mock!(UniswapV2Pair {});

    #[test]
    fn test_get_uni_v2_amt_out() {
        let token_0 = Erc20Info {
            address: "aB1a4d4f1D656d2450692D237fdD6C7f9146e814".parse().unwrap(),
            name: "token0".to_string(),
            decimals: 18,
        };
        let token_1 = Erc20Info {
            address: "ae13d989daC2f0dEbFf460aC112a837C89BAa7cd".parse().unwrap(),
            name: "token1".to_string(),
            decimals: 18,
        };

        let state = UniswapV2PairState {
            token_0,
            token_1,
            reserves: UniswapV2Reserves {
                reserve_0: ETHER.checked_mul(U128::from(1000)).unwrap(),
                reserve_1: ETHER.checked_mul(U128::from(2000)).unwrap(),
                block_ts_last: 0,
            },
        };
        let ret1 = get_uni_v2_amt_out(
            &state,
            &state.token_0.address,
            ETHER.checked_mul(U128::from(10)).unwrap(),
        );
        assert_eq!(ret1, U128::from(19743160687941225977u128));

        let ret2 = get_uni_v2_amt_out(
            &state,
            &state.token_1.address,
            ETHER.checked_mul(U128::from(10)).unwrap(),
        );
        assert_eq!(ret2, U128::from(4960273038901078125u128));
    }
}
