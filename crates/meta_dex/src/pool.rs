use std::hash::{Hash, Hasher};

use ethers::prelude::*;
use meta_common::enums::PoolVariant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pool {
    pub address: Address,
    pub token_0: Address,
    pub token_1: Address,
    pub swap_fee: U256,
    pub pool_variant: PoolVariant,
}

impl Pool {
    // Creates a new pool instance
    pub fn new(
        address: Address,
        token_a: Address,
        token_b: Address,
        swap_fee: U256,
        pool_variant: PoolVariant,
    ) -> Pool {
        let (token_0, token_1) =
            if token_a < token_b { (token_a, token_b) } else { (token_b, token_a) };

        Pool { address, token_0, token_1, swap_fee, pool_variant }
    }
}

impl Hash for Pool {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}
