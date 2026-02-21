use crate::Key;

/// Errors returned when building a [`Catalog`](super::Catalog).
#[derive(Debug, thiserror::Error)]
pub enum CatalogBuildError {
    #[error("duplicate item key: {0}")]
    DuplicateItemKey(Key),
    #[error("duplicate facility key: {0}")]
    DuplicateFacilityKey(Key),
    #[error("recipe ingredients contains duplicate item id {item_id}")]
    DuplicateRecipeIngredientItem { item_id: u32 },
    #[error("recipe products contains duplicate item id {item_id}")]
    DuplicateRecipeProductItem { item_id: u32 },
    #[error("recipe ingredients must not be empty")]
    RecipeIngredientsMustNotBeEmpty,
    #[error("recipe products must not be empty")]
    RecipeProductsMustNotBeEmpty,
}
