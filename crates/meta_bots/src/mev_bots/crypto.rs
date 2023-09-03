use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};

/// Sign eip1559 transactions
pub async fn sign_eip1559(
    tx: Eip1559TransactionRequest,
    signer_wallet: &LocalWallet,
) -> Result<Bytes, WalletError> {
    let tx_typed = TypedTransaction::Eip1559(tx);
    let signed_frontrun_tx_sig = match signer_wallet.sign_transaction(&tx_typed).await {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    Ok(tx_typed.rlp_signed(&signed_frontrun_tx_sig))
}
