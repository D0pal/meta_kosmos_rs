use ethers::prelude::*;
use meta_util::ether::{h160_to_b160, h256_to_u256_be, u256_to_ru256};
use revm::primitives::TransactTo;

/// Configures the env for the transaction
pub fn configure_tx_env(env: &mut revm::primitives::Env, tx: &Transaction) {
    env.tx.caller = h160_to_b160(tx.from);
    env.tx.gas_limit = tx.gas.as_u64();
    env.tx.gas_price = tx.gas_price.unwrap_or_default().into();
    env.tx.gas_priority_fee = tx.max_priority_fee_per_gas.map(Into::into);
    env.tx.nonce = Some(tx.nonce.as_u64());
    env.tx.access_list = tx
        .access_list
        .clone()
        .unwrap_or_default()
        .0
        .into_iter()
        .map(|item| {
            (
                h160_to_b160(item.address),
                item.storage_keys.into_iter().map(h256_to_u256_be).map(u256_to_ru256).collect(),
            )
        })
        .collect();
    env.tx.value = tx.value.into();
    env.tx.data = tx.input.0.clone();
    env.tx.transact_to =
        tx.to.map(h160_to_b160).map(TransactTo::Call).unwrap_or_else(TransactTo::create)
}
