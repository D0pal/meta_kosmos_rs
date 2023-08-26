pub mod v2;
// pub mod v3;

use std::sync::Arc;

use ethers::prelude::{
    k256::{ecdsa::SigningKey, Secp256k1},
    *,
};
use tokio::sync::RwLock;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref WETH_ENCODE_DIVISOR: U256 = U256::from(100_000);
}


#[derive(Debug, Clone)]
pub struct SandwichMaker {
    pub v2: v2::SandwichLogicV2,
    // pub v3: v3::SandwichLogicV3,
    pub sandwich_address: Address,
    pub searcher_wallet: Arc<LocalWallet>,
    pub nonce: Arc<RwLock<U256>>,
}

impl SandwichMaker {
    // Create a new `SandwichMaker` instance
    pub async fn new(
        weth_address: Address,
        sandwich_contract_address: Address,
        searcher_wallet: Arc<LocalWallet>,
        provider: Arc<Provider<Ws>>,
    ) -> Self {
        let nonce =
            if let Ok(n) = provider.get_transaction_count(searcher_wallet.address(), None).await {
                n
            } else {
                panic!("Failed to get searcher wallet nonce...");
            };

        let nonce = Arc::new(RwLock::new(nonce));

        Self {
            v2: v2::SandwichLogicV2::new(weth_address),
            // v3: v3::SandwichLogicV3::new(),
            sandwich_address: sandwich_contract_address,
            searcher_wallet,
            nonce,
        }
    }
}

