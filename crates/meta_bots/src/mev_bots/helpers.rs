use ethers::{
    abi::{self, parse_abi, ParamType},
    prelude::BaseContract,
    types::{
        transaction::eip2930::{AccessList, AccessListItem},
        Address, BigEndianHash, Bytes, H256, U256,
    },
    utils::parse_ether,
};
use meta_address::get_bot_contract_info;
use meta_common::{
    enums::{BotType, Network},
    traits::ContractCode,
};
use meta_dex::prelude::BlockInfo;
use revm::{
    primitives::{
        Address as rAddress, Bytecode, ExecutionResult, Output, TransactTo, B256, U256 as rU256,
    },
    EVM,
};
use std::str::FromStr;

use crate::forked_db::{ForkDB, ForkFactory};

use super::{
    simulation::SimulationError, BRAINDANCE_STARTING_BALANCE, DEV_BRAINDANCE_ADDRESS,
    DEV_BRAINDANCE_CONTRAOLLER_ADDRESS, DEV_CALLER_ADDRESS,
};

// Setup braindance for current fork factory by injecting braindance
// contract code and setting up balances
//
// Arguments:
// * `&mut fork_factory`: mutable reference to fork db factory
//
// Returns: This function returns nothing
pub fn attach_braindance_module(
    network: Network,
    fork_factory: &mut ForkFactory,
    weth_address: Address,
) {
    inject_braindance_code(network, fork_factory);

    // Get balance mapping of braindance contract inside of weth contract
    let slot: U256 = ethers::utils::keccak256(abi::encode(&[
        abi::Token::Address((*DEV_BRAINDANCE_ADDRESS).0.into()),
        abi::Token::Uint(U256::from(3)),
    ]))
    .into();

    let value = *BRAINDANCE_STARTING_BALANCE;

    fork_factory.insert_account_storage(weth_address.0.into(), slot.into(), value.into()).unwrap();
}

// Inject test sandwich code for when we run test. Allows us to test new
// sandwich contact locally
//
// Arguments:
// * `fork_factory`: mutable reference to fork db factory
// * `starting_weth_balance`: weth balance sandwich contract is initialized with
//
// Returns: This function returns nothing
pub fn inject_sando(
    network: Network,
    fork_factory: &mut ForkFactory,
    sandwidth_contract_starting_weth_balance: U256,
    sandwidth_contract_address: Address,
    searcher_address: Address,
    weth_address: Address,
) {
    // give searcher some balance to pay for gas fees
    let searcher = searcher_address;
    let gas_money = parse_ether(100).unwrap();
    let account = revm::primitives::AccountInfo::new(
        gas_money.into(),
        0,
        B256::default(),
        Bytecode::default(),
    );
    fork_factory.insert_account_info(searcher.0.into(), account);

    // setup sandwich contract
    let sandwich = sandwidth_contract_address;
    let account = revm::primitives::AccountInfo::new(
        rU256::from(0),
        0,
        B256::default(),
        Bytecode::new_raw(
            get_bot_contract_info(BotType::SANDWIDTH_HUFF, network)
                .unwrap()
                .get_byte_code_and_hash()
                .0
                 .0,
        ),
    );
    fork_factory.insert_account_info(sandwich.0.into(), account);

    // add starting weth balance to sandwich contract
    let slot: U256 = ethers::utils::keccak256(abi::encode(&[
        abi::Token::Address(sandwich.0.into()),
        abi::Token::Uint(U256::from(3)),
    ]))
    .into();

    // update changes
    fork_factory
        .insert_account_storage(
            weth_address.0.into(),
            slot.into(),
            sandwidth_contract_starting_weth_balance.into(),
        )
        .unwrap();
}

// Add bytecode to braindance address
//
// Arguments:
// `&mut fork_factory`: mutable reference to `ForkFactory` instance to inject
//
// Returns: This function returns nothing
fn inject_braindance_code(network: Network, fork_factory: &mut ForkFactory) {
    // setup braindance contract
    let account = revm::primitives::AccountInfo::new(
        rU256::from(0),
        0,
        B256::default(),
        Bytecode::new_raw(
            get_bot_contract_info(BotType::BRAIN_DANCE_SOL, network)
                .unwrap()
                .get_byte_code_and_hash()
                .0
                 .0,
        ),
    );
    fork_factory.insert_account_info((*DEV_BRAINDANCE_ADDRESS).0.into(), account);

    // setup braindance contract controller
    let account = revm::primitives::AccountInfo::new(
        parse_ether(69).unwrap().into(),
        0,
        B256::default(),
        Bytecode::default(),
    );
    fork_factory.insert_account_info((*DEV_BRAINDANCE_CONTRAOLLER_ADDRESS).0.into(), account);
}

// Setup evm blockstate
//
// Arguments:
// * `&mut evm`: mutable refernece to `EVM<ForkDB>` instance which we want to modify
// * `&next_block`: reference to `BlockInfo` of next block to set values against
//
// Returns: This function returns nothing
pub fn setup_block_state(evm: &mut EVM<ForkDB>, next_block: &BlockInfo) {
    evm.env.block.number = rU256::from(next_block.number.as_u64());
    evm.env.block.timestamp = next_block.timestamp.into();
    evm.env.block.basefee = next_block.base_fee.into();
    // use something other than default
    evm.env.block.coinbase =
        rAddress::from_str("0xDecafC0FFEe15BAD000000000000000000000000").unwrap();
}

// Find amount out from an amount in using the k=xy formula
// note: assuming fee is set to 3% for all pools (not case irl)
//
// Arguments:
// * `amount_in`: amount of token in
// * `target_pool`: address of pool
// * `token_in`: address of token in
// * `token_out`: address of token out
// * `evm`: mutable reference to evm used for query
//
// Returns:
// Ok(U256): amount out
// Err(SimulationError): if error during caluclation
pub fn get_amount_out_evm(
    amount_in: U256,
    target_pool: Address,
    token_in: Address,
    token_out: Address,
    evm: &mut EVM<ForkDB>,
) -> Result<U256, SimulationError> {
    // get reserves
    evm.env.tx.transact_to = TransactTo::Call(target_pool.0.into());
    evm.env.tx.caller = (*DEV_CALLER_ADDRESS).0.into();
    evm.env.tx.value = rU256::ZERO;
    evm.env.tx.data = Bytes::from_str("0x0902f1ac").unwrap().0; // getReserves()
    let result = match evm.transact_ref() {
        Ok(result) => result.result,
        Err(e) => return Err(SimulationError::EvmError(e)),
    };
    let output: Bytes = match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Call(o) => o.into(),
            Output::Create(o, _) => o.into(),
        },
        ExecutionResult::Revert { output, .. } => return Err(SimulationError::EvmReverted(output)),
        ExecutionResult::Halt { reason, .. } => return Err(SimulationError::EvmHalted(reason)),
    };

    let tokens = abi::decode(
        &vec![ParamType::Uint(128), ParamType::Uint(128), ParamType::Uint(32)],
        &output,
    )
    .unwrap();

    let reserves_0 = tokens[0].clone().into_uint().unwrap();
    let reserves_1 = tokens[1].clone().into_uint().unwrap();

    let (reserve_in, reserve_out) = match token_in < token_out {
        true => (reserves_0, reserves_1),
        false => (reserves_1, reserves_0),
    };

    let a_in_with_fee: U256 = amount_in * 997;
    let numerator: U256 = a_in_with_fee * reserve_out;
    let denominator: U256 = reserve_in * 1000 + a_in_with_fee;
    let amount_out: U256 = numerator.checked_div(denominator).unwrap_or(U256::zero());

    Ok(amount_out)
}

// Get token balance
//
// Arguments:
// * `token`: erc20 token to query
// * `owner`: address to find balance of
// * `next_block`: block to query balance at
// * `evm`: evm instance to run query on
//
// Returns:
// `Ok(balance: U256)` if successful, Err(SimulationError) otherwise
pub fn get_balance_of_evm(
    token: Address,
    owner: Address,
    next_block: &BlockInfo,
    evm: &mut EVM<ForkDB>,
) -> Result<U256, SimulationError> {
    let erc20 = BaseContract::from(
        parse_abi(&["function balanceOf(address) external returns (uint)"]).unwrap(),
    );

    evm.env.tx.transact_to = TransactTo::Call(token.0.into());
    evm.env.tx.data = erc20.encode("balanceOf", owner).unwrap().0;
    evm.env.tx.caller = (*DEV_CALLER_ADDRESS).0.into();
    evm.env.tx.gas_price = next_block.base_fee.into();
    evm.env.tx.gas_limit = 700000;
    evm.env.tx.value = rU256::ZERO;

    let result = match evm.transact_ref() {
        Ok(result) => result.result,
        Err(e) => {
            return Err(SimulationError::EvmError(e));
        }
    };

    let output: Bytes = match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Call(o) => o.into(),
            Output::Create(o, _) => o.into(),
        },
        ExecutionResult::Revert { output, .. } => return Err(SimulationError::EvmReverted(output)),
        ExecutionResult::Halt { reason, .. } => return Err(SimulationError::EvmHalted(reason)),
    };

    match erc20.decode_output("balanceOf", &output) {
        Ok(tokens) => return Ok(tokens),
        Err(e) => return Err(SimulationError::AbiError(e)),
    }
}

// Converts access list from revm to ethers type
//
// Arguments:
// * `access_list`: access list in revm format
//
// Returns:
// `AccessList` in ethers format
pub fn convert_access_list(access_list: Vec<(rAddress, Vec<rU256>)>) -> AccessList {
    let mut converted_access_list = Vec::new();
    for access in access_list {
        let address = access.0;
        let keys = access.1;
        let access_item = AccessListItem {
            address: address.0.into(),
            storage_keys: keys
                .iter()
                .map(|k| {
                    let slot_u256: U256 = k.clone().into();
                    let slot_h256: H256 = H256::from_uint(&slot_u256);
                    slot_h256
                })
                .collect::<Vec<H256>>(),
        };
        converted_access_list.push(access_item);
    }

    AccessList(converted_access_list)
}
