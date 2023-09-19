use ethers::{prelude::*};
use meta_address::{get_rpc_info, get_token_info, Token};
use meta_common::enums::{ContractType, DexExchange, Network, RpcProvider};
use meta_contracts::bindings::uniswapv3pool::SwapFilter;
use meta_dex::DexService;
use meta_util::ether::{address_from_str};

use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let network = Network::ARBI;
    let rpc_provider = RpcProvider::Quick;
    let dex = DexExchange::UniswapV3;
    let usdc = Token::USDC;
    let arb = Token::ARB;
    let weth = Token::WETH;
    let usdc_token_info = get_token_info(usdc, network).unwrap();
    let arb_token_info = get_token_info(arb, network).unwrap();
    let _weth_token_info = get_token_info(weth, network).unwrap();

    let _swap_router_v2 = ContractType::UniV3SwapRouterV2;

    let rpc_info = get_rpc_info(network).unwrap();

    let V3_FEE = 500;

    println!("token_info {:?}", usdc_token_info);

    let rpc_url = rpc_info.ws_urls.get(&rpc_provider).unwrap();
    println!("rpc_url {:?}", rpc_url);
    let provider_ws = Provider::<Ws>::connect(rpc_url).await.expect("ws connect error");
    // let provider_ws = Provider::<Http>::connect(&rpc_info.httpUrls[0]).await;
    let provider_ws = provider_ws.interval(Duration::from_millis(200));
    let provider_ws = Arc::new(provider_ws);

    let last_block = provider_ws.get_block(BlockNumber::Latest).await?.unwrap().number.unwrap();
    println!("last_block: {}", last_block);

    let receipient = address_from_str("0xCd699De44114725d274a75b9da30548382c8a7BF");
    println!("receipient {:?}", format!("{:?}", receipient));
    let dex_service = DexService::new(Arc::clone(&provider_ws), network, dex);

    let pool = dex_service
        .dex_contracts
        .get_v3_pool(arb_token_info.address, usdc_token_info.address, V3_FEE)
        .await
        .unwrap();

    println!("pool contract address: {:?}", pool.address());

    println!("hash {:?}", H256::from(receipient));

    let filter = pool
        .event::<SwapFilter>()
        .from_block(last_block)
        .topic2(ValueOrArray::Value(H256::from(receipient)));

    let mut events_stream = filter.subscribe().await.unwrap().with_meta();

    loop {
        let next = events_stream.next().await;
        if let Some(log) = next {
            let (swap_log, meta) = log.unwrap() as (SwapFilter, LogMeta);
            // swap_log.

            println!(
                "block: {:?}, hash: {:?}, address: {:?}, log {:?}",
                meta.block_number, meta.transaction_hash, meta.address, swap_log
            );
            // if meta.block_number.as_u64() > last_block {
            //     last_block = meta.block_number.as_u64();
            //     check_price_diff(
            //         &pancake_pair_contract_wrapper,
            //         &biswap_pair_contract_wrapper,
            //         last_block,
            //     )
            //     .await;
            // }
        }
    }
    Ok(())
}
