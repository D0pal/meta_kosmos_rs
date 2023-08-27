use std::marker::PhantomData;
use std::sync::Arc;
use ethers::prelude::{Address, Bytes, ContractCall, Middleware, U256};
use washington_derive::delegate;
use crate::bostan::{self, DecreaseLiquidityParams, ExactInputSingleWithoutRecipientParams, ExactInputWithoutRecipientParams, ExactOutputSingleWithoutRecipientParams, ExactOutputWithoutRecipientParams, IncreaseLiquidityParams, MintWithoutRecipientParams};

pub trait Delegator<M: Middleware, T>: Clone {

    fn call(&self, delegated: Address, param: Bytes) -> ContractCall<M, T>;

    fn middleware(&self) -> Arc<M>;
}

impl<M: Middleware> Delegator<M, ()> for bostan::Vault<M> {
    fn call(&self, delegated: Address, param: Bytes) -> ContractCall<M, ()> {
        self.execute(delegated, param)
    }

    fn middleware(&self) -> Arc<M> {
        self.client()
    }
}

#[delegate(contract=bostan::UniV2Strategy)]
pub trait UniV2StrategyDelegator<M, T> {
    fn add_liquidity(
        &self,
        token_a: Address,
        token_b: Address,
        amount_a_desired: U256,
        amount_b_desired: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        deadline: U256,
    ) -> ContractCall<M, T>;

    fn remove_liquidity(
        &self,
        liquidity_token: Address,
        token_a: Address,
        token_b: Address,
        liquidity: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        deadline: U256,
    ) -> ContractCall<M, T>;

    fn swap_exact_tokens_for_tokens(
        &self,
        amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
        deadline: U256,
    ) -> ContractCall<M, T>;

    fn swap_tokens_for_exact_tokens(
        &self,
        amount_out: U256,
        amount_in_max: U256,
        path: Vec<Address>,
        deadline: U256,
    ) -> ContractCall<M, T>;
}

#[delegate(contract=bostan::UniV3Strategy)]
pub trait UniV3StrategyDelegator<M, T> {

    fn exact_input_single(
        &self,
        params: ExactInputSingleWithoutRecipientParams
    ) -> ContractCall<M, T>;

    fn exact_input(
        &self,
        params: ExactInputWithoutRecipientParams
    ) -> ContractCall<M, T>;

    fn exact_output_single(
        &self,
        params: ExactOutputSingleWithoutRecipientParams
    ) -> ContractCall<M, T>;

    fn exact_output(
        &self,
        params: ExactOutputWithoutRecipientParams
    ) -> ContractCall<M, T>;

    fn mint(
        &self,
        params: MintWithoutRecipientParams
    ) -> ContractCall<M, T>;

    fn increase_liquidity(
        &self,
        params: IncreaseLiquidityParams
    ) -> ContractCall<M, T>;

    fn decrease_liquidity(
        &self,
        params: DecreaseLiquidityParams
    ) -> ContractCall<M, T>;
}

#[delegate(contract=bostan::CurveStrategy)]
pub trait CurveStrategyDelegator<M, T> {
    fn exchange_v1(
        &self,
        pool: Address,
        i: i128,
        j: i128,
        dx: U256,
        min_dy: U256
    ) -> ContractCall<M, T>;

    fn exchange_v2(
        &self,
        pool: Address,
        i: U256,
        j: U256,
        dx: U256,
        min_dy: U256,
    ) -> ContractCall<M, T>;
}
