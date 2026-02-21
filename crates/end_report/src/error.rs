use end_model::OutpostId;
use end_opt::LogisticsNodeId;
use thiserror::Error;

/// Result alias for report generation.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors raised when report rendering references missing input/logistics indices.
#[derive(Debug, Error)]
pub enum Error {
    #[error("missing outpost index {}", .0.as_u32())]
    MissingOutpost(OutpostId),

    #[error("missing logistics node {node:?} for item id {item:?}")]
    MissingLogisticsNode { item: u32, node: LogisticsNodeId },
}
