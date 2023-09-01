use ethers::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimulationError<M: Middleware> {
    #[error("{0}")]
    SimulationEvmOtherTxError(String),

    #[error("{0}")]
    SimulationEvmError(String),

    #[error("node tracing provider error")]
    NodeTracingProviderError(#[from] ProviderError),

    #[error("unable to get state diff for tx: `{0}`")]
    NodeTracingNotGetStateDiffError(String),

    #[error("tx not found: `{0}`")]
    TransactionNotFound(TxHash),

    #[error("tx block number is none")]
    TransactionBlkNumberNotFound,

    #[error("unable to replay: `{0}`")]
    UnableToReplay(TxHash),

    #[error("decode revert msg error")]
    DecodeRevertMsgError,

    #[error(transparent)]
    ContractError(#[from] ContractError<M>),

    #[error("block number does not match current {0}, required {1}")]
    BlockNumberUnmatch(u64, u64),

    #[error("fork factory not ready")]
    ForkfactoryNotReady,
}

#[derive(Debug, Error)]
pub enum AnalyzeError<M: Middleware> {
    #[error("must provider pool lists to anlayze")]
    MustProvidePoolError,

    #[error("simulate error")]
    SimulateError(#[from] SimulationError<M>),
}

#[derive(Debug, Error)]
pub enum FlashbotsError {
    #[error("reqwest error {0}")]
    FlashbotsReqwestResponseError(#[from] reqwest::Error),
}

