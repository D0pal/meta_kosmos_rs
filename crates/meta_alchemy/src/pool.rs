use crate::{
    // error::SimulationError,
    fork_db::fork_factory::ForkFactory,
    storage::{
        code::{
            get_arbitrum_extended_weth_code_and_hash, get_arbitrum_fiat_token_code_and_hash,
            get_circle_proxy_code_and_hash, ARB_V3_POOL_CODE_MAP, ETH_V3_POOL_CODE_MAP,
        },
        slots::{
            erc20::{erc20_balance_of_storage_slot, ETH_ERC20_USDC_PREFETCH_SLOTS},
            uni_v3::{uni_v3_slot_0_storage_slot, COMMON_SLOTS, TICK_MAP_SLOTS, TICK_SLOTS},
        },
    },
};
use ethers::prelude::*;
use futures::future::join_all;
use meta_address::{
    get_addressed_token_info, get_dex_address, get_token_info, Token as Erc20Token,
};
use meta_common::{
    enums::{ContractType, DexExchange, Network, PoolVariant},
    traits::ContractCode,
};
use revm::primitives::{AccountInfo, Bytecode, B160 as rAddress, B256, U256 as rU256};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DefiStorage {
    pub network: Network,
    pub pool_address: Address,
    pub token0: Address,
    pub token1: Address,
    pub swap_fee: u32,
    pub pool_variant: PoolVariant,
}

impl DefiStorage {
    pub fn new(
        network: Network,
        pool_address: Address,
        token_a: Address,
        token_b: Address,
        swap_fee: u32,
        pool_variant: PoolVariant,
    ) -> Self {
        let (token0, token1) =
            if token_a < token_b { (token_a, token_b) } else { (token_b, token_a) };
        DefiStorage { network, pool_address, token0, token1, swap_fee, pool_variant }
    }

    #[allow(dead_code)]
    fn get_unique_pool_id(&self) -> String {
        format!("{:?}_{:?}_{:?}_{}", self.pool_variant, self.token0, self.token1, self.swap_fee)
    }

    /// return the contract code to be used for tx simulation
    /// # Return
    /// K: the contract address, V: tuple of byte code and code hash
    pub fn get_simulation_contract_code(&self) -> HashMap<Address, (Bytes, [u8; 32])> {
        let mut map = HashMap::new();
        match self.pool_variant {
            PoolVariant::UniswapV3 => {
                let token_0_code = self.get_token_code_and_hash(self.token0);
                map.extend(token_0_code);
                let token_1_code = self.get_token_code_and_hash(self.token1);
                map.extend(token_1_code);
                let quoter_code = self.get_v3_quoter_code_and_hash();
                map.extend(quoter_code);
                let pool_code = {
                    match self.network {
                        Network::ETH => ETH_V3_POOL_CODE_MAP.get(&self.pool_address),
                        Network::ARBI => ARB_V3_POOL_CODE_MAP.get(&self.pool_address),
                        _ => unimplemented!(),
                    }
                };
                if let Some(code) = pool_code {
                    map.insert(self.pool_address, code.clone());
                }
            }
            _ => unimplemented!(),
        }
        map
    }

    /// return the storage slots to be fetched for v3 tx simulation
    /// # Return
    /// K: the contract address, V: vector or slots
    pub fn get_simulation_prefetch_storage_slots(
        &self,
        current_tick: i32,
    ) -> HashMap<Address, Vec<H256>> {
        match self.pool_variant {
            PoolVariant::UniswapV3 => {
                let mut map = HashMap::new();
                map.insert(self.token0, self.get_v3_token_slots(&self.token0));
                map.insert(self.token1, self.get_v3_token_slots(&self.token1));
                map.insert(self.pool_address, self.get_v3_pool_slots(current_tick));
                map
            }
            _ => unimplemented!(),
        }
    }

    fn get_token_code_and_hash(
        &self,
        token_address: Address,
    ) -> HashMap<Address, (Bytes, [u8; 32])> {
        let mut map = HashMap::new();
        let token_info = get_addressed_token_info(self.network, token_address);
        if let Some(info) = token_info {
            map.insert(token_address, info.get_byte_code_and_hash());
        }

        // eth, usdc is a proxy token
        if self.network.eq(&Network::ETH) {
            let usdc_token_info = get_token_info(Erc20Token::USDC, self.network);
            match usdc_token_info {
                Some(usdc) => {
                    if usdc.address.eq(&token_address) {
                        let (address, (code, code_hash)) = get_circle_proxy_code_and_hash();
                        map.insert(address, (code, code_hash));
                    }
                }
                None => unimplemented!(),
            }
        }
        if self.network.eq(&Network::ARBI) {
            let (address, (code, code_hash)) = get_arbitrum_extended_weth_code_and_hash();
            map.insert(address, (code, code_hash));
            let usdc_token_info = get_token_info(Erc20Token::USDC, self.network);
            match usdc_token_info {
                Some(usdc) => {
                    if usdc.address.eq(&token_address) {
                        let (address, (code, code_hash)) = get_arbitrum_fiat_token_code_and_hash();
                        map.insert(address, (code, code_hash));
                    }
                }
                None => unimplemented!(),
            }
        }
        map
    }

    fn get_v3_quoter_code_and_hash(&self) -> HashMap<Address, (Bytes, [u8; 32])> {
        let mut map = HashMap::new();
        let quoter =
            get_dex_address(DexExchange::UniswapV3, self.network, ContractType::UniV3QuoterV2);
        if let Some(info) = quoter {
            let (code, hash) = info.get_byte_code_and_hash();
            map.insert(info.address, (code, hash));
        }
        map
    }

    /// Get token contract slots to be fetched for a v3 pool
    /// For non standard tokens (e.g, USDC token in eth mainnet, it is a proxy contract), specific slots are fetched also
    fn get_v3_token_slots(&self, token: &Address) -> Vec<H256> {
        let mut token_contract_slots = vec![erc20_balance_of_storage_slot(self.pool_address)];
        if self.network.eq(&Network::ETH) {
            let usdc_token_info = get_token_info(Erc20Token::USDC, self.network);
            match usdc_token_info {
                Some(usdc) => {
                    if usdc.address.eq(token) {
                        token_contract_slots
                            .extend_from_slice(&ETH_ERC20_USDC_PREFETCH_SLOTS.clone());
                    }
                }
                None => unimplemented!(),
            }
        }
        token_contract_slots
    }

    /// Get pool contracts slots to be fetched within a tick range
    /// tick range defauts to [-20, 20] centerred at current tick
    fn get_v3_pool_slots(&self, tick: i32) -> Vec<H256> {
        let mut common_slots = COMMON_SLOTS.clone();
        let tick_slots: Vec<H256> = (tick - 20..tick + 20)
            .into_iter()
            .flat_map(|i| TICK_SLOTS.get(&i).map_or(vec![], |x| x.to_vec()))
            .collect();
        let tick_map_slots: Vec<H256> = (tick - 20..tick + 20)
            .into_iter()
            .filter_map(|i| {
                if let Some(slot) =
                    TICK_MAP_SLOTS.get(&(i, uni_v3_tick_spacking_from_fee(self.swap_fee as i32)))
                {
                    Some(*slot)
                } else {
                    None
                }
            })
            .collect();
        common_slots.extend(tick_slots);
        common_slots.extend(tick_map_slots);
        common_slots
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BalanceChange {
    pub pre: U256,
    pub post: U256,
}

/// Holds pools that have the potential to be sandwiched
#[derive(Clone, Copy, Debug)]
pub struct SimPool {
    pub pool: DefiStorage,
    pub weth_change: Option<BalanceChange>,
}

impl SimPool {
    pub fn new(pool: DefiStorage, weth_change: Option<BalanceChange>) -> Self {
        Self { pool, weth_change }
    }
}

/// fee 500 -> tick spacing 10
/// fee 3000 -> 60
pub fn uni_v3_tick_spacking_from_fee(fee: i32) -> i32 {
    if fee == 500 {
        10
    } else if fee == 3000 {
        60
    } else {
        200
    }
}

pub async fn get_v3_pool_slot_0(
    ws_provider: Arc<Provider<Ws>>,
    pool_address: Address,
    block: BlockId,
) -> Option<H256> {
    let slot = uni_v3_slot_0_storage_slot();
    let val = ws_provider.get_storage_at(pool_address, slot, Some(block)).await.ok();
    val
}

pub async fn prepare_pool(
    ws_provider: Arc<Provider<Ws>>,
    storage: &DefiStorage,
    fork_factory: Arc<RwLock<ForkFactory>>,
    tick: i32,
    block: Option<BlockId>,
    // future: impl std::future::Future<Output = Result<i32, ContractError<Provider<Ws>>>>,
    // code_needed: bool,
) -> anyhow::Result<()> {
    // let start = Instant::now();
    // println!("start prepare for pool {:?}", storage.pool_address);
    // TODO: do not have to inject codes every time
    // prepare contracts code
    let code_map = storage.get_simulation_contract_code();
    for (address, (code, code_hash)) in code_map.iter() {
        let account = AccountInfo {
            nonce: 0,
            balance: rU256::from(0),
            code: Some(Bytecode::new_raw(code.clone().0)),
            code_hash: B256::from_slice(code_hash),
        };
        {
            let mut f = fork_factory.write().await;
            (*f).insert_account_info(address.0.into(), account);
        }
    }

    // prepre caler state
    // let caller_acc = AccountInfo::from_balance(
    //     decimal_to_wei(Decimal::from_f64(1000f64).unwrap(), 18).into(),
    // );
    // fork_factory.insert_account_info(SIMULATION_CALLER.0.into(), caller_acc);

    // pre fetch all pool states
    // let tick = maybe(tick, future).await?;
    let slots = storage.get_simulation_prefetch_storage_slots(tick);
    let mut all_futures = vec![];

    for (contract, contract_slots) in slots.iter() {
        for slot in contract_slots {
            all_futures.push(async {
                let val = ws_provider.get_storage_at(*contract, slot.clone(), block).await;
                ((*contract, *slot), val)
            })
        }
    }
    let slots = join_all(all_futures).await;

    // insert pool states in EVM
    for ((contract, slot), state) in slots.iter() {
        if let Ok(val) = state {
            {
                let mut f = fork_factory.write().await;
                let _ = (*f).insert_account_storage(
                    rAddress::from_slice(&contract.0),
                    rU256::from_be_bytes(slot.0),
                    rU256::from_be_bytes(val.0),
                );
            }
        }
    }
    // let end = Instant::now();
    // println!(
    //     "end prepare for pool {:?}, total time spent {:?} ms",
    //     storage.pool_address,
    //     end.duration_since(start).as_millis()
    // );
    Ok(())
}

#[cfg(test)]
mod test_pools {
    use meta_util::ether::address_from_str;

    use super::*;
    #[test]
    fn test_storage() {
        let pool_address = address_from_str("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640"); // let v3_weth_usdc_500_address =
        let token0 = address_from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"); //usdc
        let token1 = address_from_str("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"); // weth

        let _v2_weth_usdc_pool_address =
            address_from_str("0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc");
        let pool = DefiStorage::new(
            Network::ETH,
            pool_address,
            token0,
            token1,
            500,
            PoolVariant::UniswapV3,
        );

        assert_eq!(pool.get_unique_pool_id(), "UniswapV3_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48_0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2_500");
        // let ret = pool.get_simulation_prefetch_storage_slots(200545);
        // ret.iter().for_each(|(k, v)| {
        //     println!("k: {:?}, v: {:?}", k, v);
        // });

        let ret = pool.get_simulation_contract_code();
        ret.iter().for_each(|(k, v)| {
            println!("k: {:?}, v: {:?}", k, v.1);
        });
    }
}

// current tick: 200545
// tick_spacing: 10
// total slots to fetch 5
// fetch storage address: 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640, slot: 0x0000000000000000000000000000000000000000000000000000000000000000, val: 0x00010002d202d2014b030f6100000000000058605e94188dab8c940a6f385312
// fetch storage address: 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640, slot: 0x0000000000000000000000000000000000000000000000000000000000000004, val: 0x0000000000000000000000000000000000000000000000025fed9a72906660bf
// fetch storage address: 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640, slot: 0x0000000000000000000000000000000000000000000000000000000000000002, val: 0x00000000000000000000000000000d554d24606598e760bd415baf4b9d37ee2f
// fetch storage address: 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640, slot: 0xa6172f64e80168333d3bc97ef096a7cb251d24be47156dff7a40f6163a6ab42b, val: 0xdffffdfffffffffffbffffffff7ffdffffffffff7fffff7ffffffffff7ffffff
// fetch storage address: 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640, slot: 0x0000000000000000000000000000000000000000000000000000000000000153, val: 0x010000000000000001ea370764cd485ddef08098a5000c6a590e1ccb64a68337
// fetch storage address: 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640, slot: 0x0000000000000000000000000000000000000000000000000000000000000154, val: 0x010000000000000001ea368e85eebccee25ad505ec000c66f9308a3b64a56937
// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0x10d6a54a4754c8869d6886b5f5d7fbfa5b4522237ea5c60d11bc4e7a1ff9390b, val: 0x000000000000000000000000807a96288a1a408dbc13de2b1d087d10356395d2
// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0x7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3, val: 0x000000000000000000000000a2327a938febf5fec13bacfb16ae10ecbc4cbdcf
// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0x0000000000000000000000000000000000000000000000000000000000000001, val: 0x000000000000000000000000f0d160dec1749afaf5a831668093b1431f7c8527
// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0x390f6178407c9b8e95802b8659e6df8e34c1e3d4f8d6a49e6132bbcdd937b63a, val: 0x0000000000000000000000000000000000000000000000000000000000000000
// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0xe62d414f3a567cbfb91d56365331f3a3f45c3551729a8edcce7cad198e1e74c7, val: 0x0000000000000000000000000000000000000000000000000000000000000000
// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0x1f21a62c4538bacf2aabeca410f0fe63151869f172e03c0e00357ba26a341eff, val: 0x00000000000000000000000000000000000000000000000000007f398158d67f
// fetch storage address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48, slot: 0x3c53a91714fd8d27deef117c3827ce1fa74e6ad389952cc14a012bab8632e4ed, val: 0x0000000000000000000000000000000000000000000000000000000000000000
// fetch storage address: 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2, slot: 0x390f6178407c9b8e95802b8659e6df8e34c1e3d4f8d6a49e6132bbcdd937b63a, val: 0x000000000000000000000000000000000000000000000d7272384a3bad55e841
