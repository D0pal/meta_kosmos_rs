use ethers::signers::WalletError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum SendBundleError {
    #[error("Failed to sign transaction")]
    SigningError(#[from] WalletError),
    #[error("Max fee is less than next base fee")]
    MaxFeeLessThanNextBaseFee(),
    #[error("Negative miner tip")]
    NegativeMinerTip(),
    #[error("Failed to create bundle")]
    FailedToCreateBundle(),
    #[error("Failed to send bundle")]
    FailedToSendBundle(),
    #[error("Revenue does not cover frontrun gas fees")]
    FrontrunGasFeesNotCovered(),
}
