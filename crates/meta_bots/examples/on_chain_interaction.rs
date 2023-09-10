use ethers::prelude::*;
use foundry_evm::decode::decode_revert;


use gumdrop::Options;
use meta_address::{
    get_dex_address, get_rpc_info, get_token_info,
    ContractInfo, Token, TokenInfo,
};


use meta_common::{
    enums::{ContractType, DexExchange, Network, RpcProvider},
};
use meta_contracts::{
    bindings::{
        erc20::ERC20,
        swap_router::SwapRouter,
        ExactInputSingleParams, ExactOutputSingleParams, WETH9,
    },
};

use meta_util::{
    defi::{get_swap_price_limit},
    ether::{decimal_to_wei},
    time::get_current_ts,
};
use rust_decimal::{
    prelude::{FromPrimitive},
    Decimal,
};

use std::{
    sync::{
        Arc,
    },
    time::{Duration},
};



#[tokio::main]
async fn main() {
    let network = Network::ARBI;
    let rpc_provider = RpcProvider::Quick;
    let dex = DexExchange::UniswapV3;
    let usdc = Token::USDC;
    let arb = Token::ARB;
    let weth = Token::WETH;
    let usdc_token_info = get_token_info(usdc, network).unwrap();
    let _arb_token_info = get_token_info(arb, network).unwrap();
    let _weth_token_info = get_token_info(weth, network).unwrap();

    let swap_router_v2 = ContractType::UniV3SwapRouterV2;

    let rpc_info = get_rpc_info(network).unwrap();

    let _V3_FEE = 500;

    // let base_token_info = get_token_info(base_token, config.network).unwrap();
    // let quote_token_info = get_token_info(quote_token, config.network).unwrap();

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

    // let start = tokio::time::Instant::now();
    // let elapsed = tokio::time::Instant::now().duration_since(start).as_millis();
    // println!("tx {:?}, total spent {:?} ms", tx, elapsed);

    approve_token(
        usdc_token_info,
        wallet.clone(),
        swap_router_contract_info.address,
        Decimal::from_f64(10_000_000_000f64).unwrap(),
    )
    .await;

    // swap_exact_in_single(
    //     swap_router_contract_info,
    //     arb_token_info,
    //     usdc_token_info,
    //     500,
    //     Decimal::from_f64(12.0).unwrap(),
    //     wallet_address,
    //     wallet.clone(),
    // )
    // .await;

    // swap_exact_out_single(
    //     swap_router_contract_info,
    //     usdc_token_info,
    //     arb_token_info,
    //     500,
    //     Decimal::from_f64(12.0).unwrap(),
    //     wallet_address,
    //     wallet.clone(),
    // )
    // .await;
}

async fn swap_exact_in_single(
    swap_router_contract_info: ContractInfo,
    token_in: TokenInfo,
    token_out: TokenInfo,
    fee: u32,
    amount: Decimal,
    recipient: Address,
    wallet: Arc<NonceManagerMiddleware<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>>,
) {
    let ddl = get_current_ts().as_secs() + 1000000;
    let amount_in_wei = decimal_to_wei(amount, token_in.decimals.into());
    let param_output = ExactInputSingleParams {
        token_in: token_in.address,
        token_out: token_out.address,
        fee,
        recipient,
        deadline: ddl.into(),
        amount_in: amount_in_wei,
        amount_out_minimum: U256::zero(),
        sqrt_price_limit_x96: get_swap_price_limit(
            token_in.address,
            token_out.address,
            token_in.address,
        ),
    };
    println!("swap params {:?}", param_output);
    let swap_router = SwapRouter::new(swap_router_contract_info.address, wallet.clone());
    let call = swap_router.exact_input_single(param_output).gas(2_000_000).gas_price(300_000_000);
    let ret = call.send().await;
    match ret {
        Ok(ref tx) => println!("send v3 exact input transaction {:?}", tx),
        Err(e) => {
            eprintln!("e {:?}", e);
            let revert_bytes = e.as_revert().unwrap();
            let msg = decode_revert(revert_bytes, None, None);
            eprintln!("error in send tx {:?}", msg);
        }
    }
}
async fn swap_exact_out_single(
    swap_router_contract_info: ContractInfo,
    token_in: TokenInfo,
    token_out: TokenInfo,
    fee: u32,
    amount: Decimal,
    recipient: Address,
    wallet: Arc<NonceManagerMiddleware<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>>,
) {
    let ddl = get_current_ts().as_secs() + 1000000;
    let amount_in_wei = decimal_to_wei(amount, token_out.decimals.into());
    let param_output = ExactOutputSingleParams {
        token_in: token_in.address,
        token_out: token_out.address,
        fee,
        recipient,
        deadline: ddl.into(),
        amount_out: amount_in_wei,
        amount_in_maximum: decimal_to_wei(
            amount.checked_mul(Decimal::from_i32(2000).unwrap()).unwrap(),
            token_in.decimals.into(),
        ),
        sqrt_price_limit_x96: get_swap_price_limit(
            token_in.address,
            token_out.address,
            token_in.address,
        ),
    };

    let swap_router = SwapRouter::new(swap_router_contract_info.address, wallet.clone());
    println!("swap params {:?}", param_output);
    let call = swap_router.exact_output_single(param_output).gas(1_800_000).gas_price(300_000_000);
    let ret = call.send().await;
    match ret {
        Ok(ref tx) => println!("send v3 exact out single transaction {:?}", tx),
        Err(e) => {
            eprintln!("e {:?}", e);
            let revert_bytes = e.as_revert().unwrap();
            let msg = decode_revert(revert_bytes, None, None);
            eprintln!("error in send tx {:?}", msg);
        }
    }
}

async fn wrap(
    weth_token_info: TokenInfo,
    wallet: Arc<NonceManagerMiddleware<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>>,
) {
    let weth_token = WETH9::new(weth_token_info.address, wallet.clone());
    let deposit_call = weth_token
        .deposit()
        .value(decimal_to_wei(Decimal::from_f64(0.1).unwrap(), weth_token_info.decimals.into()));
    let tx = deposit_call.send().await;
    println!("tx {:?}", tx);
}

async fn approve_token(
    token_info: TokenInfo,
    wallet: Arc<NonceManagerMiddleware<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>>,
    spender: Address,
    amount: Decimal,
) {
    // approve
    let token = ERC20::new(token_info.address, wallet.clone());
    let approve_call = token.approve(spender, decimal_to_wei(amount, token_info.decimals.into()));
    let tx = approve_call.send().await;
    println!("tx {:?}", tx);
}
