use ethers::prelude::*;
use meta_address::{get_dex_address, get_rpc_info, get_token_info, Token};
use meta_common::enums::{ContractType, DexExchange, Network, RpcProvider};
use meta_dex::DexService;
use meta_util::ether::tx_hash_from_str;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    let network = Network::ARBI;
    let rpc_provider = RpcProvider::Quick;
    let dex = DexExchange::UniswapV3;
    let usdc = Token::USDC;
    let arb = Token::ARB;
    let weth = Token::WETH;
    let usdc_token_info = get_token_info(usdc, network).unwrap();
    let arb_token_info = get_token_info(arb, network).unwrap();
    let _weth_token_info = get_token_info(weth, network).unwrap();

    let swap_router_v2 = ContractType::UniV3SwapRouterV2;

    let rpc_info = get_rpc_info(network).unwrap();

    let V3_FEE = 500;

    println!("token_info {:?}", usdc_token_info);

    let rpc_url = rpc_info.ws_urls.get(&rpc_provider).unwrap();
    println!("rpc_url {:?}", rpc_url);
    let provider_ws = Provider::<Ws>::connect(rpc_url).await.expect("ws connect error");
    // let provider_ws = Provider::<Http>::connect(&rpc_info.httpUrls[0]).await;
    let provider_ws = provider_ws.interval(Duration::from_millis(200));
    let provider_ws = Arc::new(provider_ws);

    let private_key = std::fs::read_to_string("/tmp/pk").unwrap().trim().to_string();
    let wallet: LocalWallet =
        private_key.parse::<LocalWallet>().unwrap().with_chain_id(rpc_info.chain_id);
    let wallet_address = wallet.address();
    let wallet = SignerMiddleware::new(provider_ws.clone(), wallet);
    let wallet = NonceManagerMiddleware::new(wallet, wallet_address);
    let wallet = Arc::new(wallet);

    let swap_router_contract_info = get_dex_address(dex, network, swap_router_v2).unwrap();
    println!("router_address {:?}", swap_router_contract_info.address);

    let dex_service = DexService::new(wallet.clone(), network, dex);
    // println!("dex_service.factory_creation_block {:?}", dex_service.factory_creation_block);


    let ret = dex_service
        .submit_order(
            arb_token_info,
            usdc_token_info,
            Decimal::from_f64(3.0).unwrap(),
            V3_FEE,
            wallet_address,
        )
        .await;
    println!("ret {:?}", ret);
    // let hash =
    //     tx_hash_from_str("0xcba0d4fc27a32aaddece248d469beb430e29c1e6fecdd5db3383e1c8b212cdeb");
    // let ret = dex_service.analyze_v3_tx(hash, arb_token_info, usdc_token_info, V3_FEE).await;
    // println!("ret {:?}", ret);
}
