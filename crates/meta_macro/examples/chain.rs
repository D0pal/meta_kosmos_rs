use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use dashmap::DashMap;
use ethers::prelude::{Address, ConnectionDetails, JsonRpcClient, Middleware, NonceManagerMiddleware, Provider, Signer, SignerMiddleware};
use url::Url;
use washington_abi::*;
use meta_macro::chain;
use washington_ethers::middleware::TransactionSubscriptionMiddleware;
use washington_ethers::rpc::concurrent::MixProvider;
use washington_utils::{strings, urls, ws_conns};
use crate::bundle::BundleManager;
use crate::transaction::start_auto_cancel_task;
use crate::error::ChainError;
use crate::TryConvert;

#[chain(
http(
    "http://sg-eth-prod-full-node-d01.timeresearch.biz:8545",
    "http://tky-eth-prod-full-node-d01.timeresearch.biz:8545",
    "https://eth-mainnet.alchemyapi.io/v2/x2uacKNi25cUmxkbycIrFpt8vCu17mxJ",
    "https://eth-mainnet.alchemyapi.io/v2/2rUJlqH2usmi83Nh-nTI1nnKhzegWn1j",
    "https://eth-mainnet.alchemyapi.io/v2/4uDu9oQSLRxL-CaSNj-41lRnX5IcuBFJ",
),
ws(
    "ws://sg-eth-prod-full-node-d01.timeresearch.biz:8546",
    "ws://tky-eth-prod-full-node-d01.timeresearch.biz:8546",
    "wss://eth-mainnet.alchemyapi.io/v2/x2uacKNi25cUmxkbycIrFpt8vCu17mxJ",
    "wss://eth-mainnet.alchemyapi.io/v2/2rUJlqH2usmi83Nh-nTI1nnKhzegWn1j",
    "wss://eth-mainnet.alchemyapi.io/v2/4uDu9oQSLRxL-CaSNj-41lRnX5IcuBFJ"
),
flashbots(
    "https://rpc.beaverbuild.org",
    "https://rsync-builder.xyz",
    "https://builder0x69.io",
    "https://relay.flashbots.net",
    "https://rpc.titanbuilder.xyz"
)

contract(
    type = common::Erc20,
    instance(name = usdt, address = "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
    instance(name = usdc, address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
    instance(name = weth, address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
    cache(
        name() -> String,
        symbol() -> String,
        decimals() -> u8,
    )
),
contract(
    type = uni_v_2::SwapRouter,
    instance(name = uniswap, address = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D")
),
contract(
    type = uni_v_3::Quoter,
    instance(name = uniswap, address = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6")
),
contract(
    type = bostan::Vault,
    instance(name = default, address = "0x41acf8682322487A9f101Dc73daaD816a3822d5a")
),
contract(
    type = bostan::UniV2Strategy,
    instance(name = default, address = "0xEDa53383c954Afa2aC7BDe8BB2Da879B3081e615"),
    delegate = delegate::UniV2Strategy
),
contract(
    type = bostan::UniV3Strategy,
    instance(name = default, address = "0xBAF2CFc01C3185bdf9A7B82ef4E9A379F5da9713"),
    delegate = delegate::UniV3Strategy
),
contract(
    type = bostan::CurveStrategy,
    instance(name = default, address = "0x0117EE5F8C88983c3a2C150dE67E26aD50b10F2E"),
    delegate = delegate::CurveStrategy
),
delegate(
    name = bostan_vault_strategies,
    caller = bostan::Vault,
    callee(delegate::UniV2Strategy, delegate::UniV3Strategy, delegate::CurveStrategy),
    output=()
)
)]
pub struct Mainnet;

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn test_contract_enums() {
        assert_eq!("usdc", CommonErc20::Usdc.to_string());
        assert_eq!(
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            format!("{:?}", CommonErc20::Usdc.address())
        );
        assert_eq!(CommonErc20::Usdc, CommonErc20::from_str("Usdc").unwrap());
        assert_eq!(CommonErc20::Usdc, CommonErc20::from_str("usdc").unwrap());
        assert_eq!(CommonErc20::Usdc, CommonErc20::from_str("USDC").unwrap());
        assert_eq!(
            CommonErc20::Usdc,
            CommonErc20::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap()
        );
        assert_eq!(
            CommonErc20::Usdc,
            CommonErc20::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap()
        );
        assert_eq!(
            "unknown contract: token",
            CommonErc20::from_str("token").err().unwrap().to_string()
        );
        assert_eq!(
            "unknown contract: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB76",
            CommonErc20::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB76").err().unwrap().to_string()
        );
        assert_eq!(
            "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            format!("{:?}", CommonErc20::Weth.address())
        );
        assert_eq!(CommonErc20::Weth, CommonErc20::from_str("weth").unwrap());
        assert_eq!(
            CommonErc20::Weth,
            CommonErc20::from_str("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap()
        );
        assert_eq!(
            "0x7a250d5630b4cf539739df2c5dacb4c659f2488d",
            format!("{:?}", UniV2SwapRouter::Uniswap.address())
        );
        assert_eq!(
            "0x41acf8682322487a9f101dc73daad816a3822d5a",
            format!("{:?}", BostanVault::Default.address())
        );
        assert_eq!(
            "0xeda53383c954afa2ac7bde8bb2da879b3081e615",
            format!("{:?}", BostanUniV2Strategy::Default.address())
        );
        assert_eq!(
            "0xbaf2cfc01c3185bdf9a7b82ef4e9a379f5da9713",
            format!("{:?}", BostanUniV3Strategy::Default.address())
        );
        assert_eq!(
            "0x0117ee5f8c88983c3a2c150de67e26ad50b10f2e",
            format!("{:?}", BostanCurveStrategy::Default.address())
        );
        assert_eq!(
            "0xeda53383c954afa2ac7bde8bb2da879b3081e615",
            format!("{:?}", DelegateUniV2Strategy::Default.address())
        );
        assert_eq!(
            "0xbaf2cfc01c3185bdf9a7b82ef4e9a379f5da9713",
            format!("{:?}", DelegateUniV3Strategy::Default.address())
        );
        assert_eq!(
            "0x0117ee5f8c88983c3a2c150de67e26ad50b10f2e",
            format!("{:?}", DelegateCurveStrategy::Default.address())
        );
    }
}
