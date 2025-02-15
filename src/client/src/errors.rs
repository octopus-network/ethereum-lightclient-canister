use common::errors::BlockNotFoundError;
use eyre::Report;
use thiserror::Error;

/// Errors that can occur during Node calls
#[derive(Debug, Error)]
pub enum NodeError {

    #[error("execution error: {0}")]
    ExecutionError(Report),

    #[error("out of sync: {0} slots behind")]
    OutOfSync(u64),

    #[error("consensus payload error: {0}")]
    ConsensusPayloadError(Report),

    #[error("execution payload error: {0}")]
    ExecutionPayloadError(Report),

    #[error("consensus client creation error: {0}")]
    ConsensusClientCreationError(Report),

    #[error("execution client creation error: {0}")]
    ExecutionClientCreationError(Report),

    #[error("consensus advance error: {0}")]
    ConsensusAdvanceError(Report),

    #[error("consensus sync error: {0}")]
    ConsensusSyncError(Report),

    #[error(transparent)]
    BlockNotFoundError(#[from] BlockNotFoundError),
}