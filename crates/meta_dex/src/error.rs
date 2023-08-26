use std::fmt;

use ethers::prelude::{AbiError, ContractError};
use ethers::providers::{Provider, ProviderError, Ws};
use ethers::signers::WalletError;
use ethers::types::H160;
use thiserror::Error;
use tokio::task::JoinError;

// use crate::prelude::is_sando_safu::OpCode;
// use crate::prelude::DatabaseError;

#[derive(Error, Debug)]
pub enum PairSyncError {
    #[error("Provider error")]
    ProviderError(#[from] ProviderError),
    #[error("Contract error")]
    ContractError(#[from] ContractError<Provider<Ws>>),
    #[error("ABI error")]
    ABIError(#[from] AbiError),
    #[error("Join error")]
    JoinError(#[from] JoinError),
    #[error("Pair for ${0}/${1} does not exist in provided dexes")]
    PairDoesNotExistInDexes(H160, H160),
}