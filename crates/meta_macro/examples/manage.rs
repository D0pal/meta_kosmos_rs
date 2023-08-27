use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use ethers::prelude::{NonceManagerMiddleware, JsonRpcClient, Provider, Signer, SignerMiddleware};
use washington_abi::*;
use washington_derive::manage;
use washington_ethers::middleware::TransactionSubscriptionMiddleware;
use washington_ethers::rpc::concurrent::MixProvider;
use crate::error::ChainError;

pub mod ethereum;
pub mod mantle;

pub trait TryConvert<T> {
    type Error;

    fn try_convert(self) -> Result<T, Self::Error>;
}

#[manage(
    ethereum::mainnet,
    mantle::mainnet
)]
pub struct ChainManager;

#[cfg(test)]
mod test {
    use crate::error::ChainError;
    use super::*;

    #[test]
    fn test_convert() {

        let eth_usdt = ethereum::mainnet::CommonErc20::Usdt;
        let res: Result<mantle::mainnet::CommonErc20, ChainError> = eth_usdt.try_convert();
        assert_eq!("Ok(Usdt)", format!("{res:?}"));
    }
}


