mod builder;
mod error;
mod model;
mod types;

pub use builder::CatalogBuilder;
pub use error::CatalogBuildError;
pub use model::Catalog;
pub use types::{
    FacilityDef, FacilityId, ItemDef, ItemId, PowerRecipe, PowerRecipeId, Recipe, RecipeId, Stack,
    ThermalBankDef,
};

/// Base/core generation capacity (watts) used by the default CLI flow.
pub const P_CORE_W: u32 = 200;

#[cfg(test)]
mod tests;
