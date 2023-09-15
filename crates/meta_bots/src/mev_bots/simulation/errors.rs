use crate::{forked_db::DatabaseError, mev_bots::simulation::inspectors::is_sando_safu::OpCode};
use ethers::prelude::AbiError;

#[derive(Debug)]
pub enum SimulationError {
    FrontrunEvmError(revm::primitives::EVMError<DatabaseError>),
    FrontrunHalted(revm::primitives::Halt),
    FrontrunReverted(revm::primitives::Bytes),
    FrontrunNotSafu(Vec<OpCode>),
    BackrunEvmError(revm::primitives::EVMError<DatabaseError>),
    BackrunHalted(revm::primitives::Halt),
    BackrunReverted(revm::primitives::Bytes),
    BackrunNotSafu(Vec<OpCode>),
    FailedToDecodeOutput(AbiError),
    EvmError(revm::primitives::EVMError<DatabaseError>),
    EvmHalted(revm::primitives::Halt),
    EvmReverted(revm::primitives::Bytes),
    AbiError(AbiError),
    ZeroOptimal(),
}
