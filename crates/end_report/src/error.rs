use end_model::{FacilityId, ItemId, OutpostId, RecipeId};
use end_opt::{DemandNodeId, Error as OptError, SupplyNodeId};
use thiserror::Error;

/// Result alias for report generation.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors raised when report rendering references missing catalog/input indices.
#[derive(Debug, Error)]
pub enum Error {
    #[error("missing item id {0:?}")]
    MissingItem(ItemId),

    #[error("missing facility id {0:?}")]
    MissingFacility(FacilityId),

    #[error("missing outpost index {}", .0.as_u32())]
    MissingOutpost(OutpostId),

    #[error("missing recipe index {}", .0.as_u32())]
    MissingRecipe(RecipeId),

    #[error("failed to rebuild logistics subproblems: {0}")]
    LogisticsBuild(#[source] OptError),

    #[error("missing logistics subproblem for item id {0:?}")]
    MissingLogisticsItem(ItemId),

    #[error("missing logistics supply node {node:?} for item id {item:?}")]
    MissingLogisticsSupplyNode { item: ItemId, node: SupplyNodeId },

    #[error("missing logistics demand node {node:?} for item id {item:?}")]
    MissingLogisticsDemandNode { item: ItemId, node: DemandNodeId },
}
