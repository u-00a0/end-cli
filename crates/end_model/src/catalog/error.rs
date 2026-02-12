use crate::Key;

/// Errors returned when building a [`Catalog`](super::Catalog).
#[derive(Debug, thiserror::Error)]
pub enum CatalogBuildError {
    #[error("duplicate item key: {0}")]
    DuplicateItemKey(Key),
    #[error("duplicate facility key: {0}")]
    DuplicateFacilityKey(Key),
    #[error("missing thermal bank facility")]
    MissingThermalBank,
    #[error("multiple thermal banks are not allowed")]
    MultipleThermalBanks,
    #[error("recipe facility id {0} is out of bounds")]
    UnknownRecipeFacilityId(u32),
    #[error("recipe time_s must be >= 1")]
    RecipeTimeMustBePositive,
    #[error("recipe ingredients must not be empty")]
    RecipeIngredientsMustNotBeEmpty,
    #[error("recipe products must not be empty")]
    RecipeProductsMustNotBeEmpty,
    #[error("recipe {list} references unknown item id {item_id}")]
    UnknownRecipeItemId { list: &'static str, item_id: u32 },
    #[error("recipe {list} contains duplicate item id {item_id}")]
    DuplicateRecipeItem { list: &'static str, item_id: u32 },
    #[error("recipe {list} item id {item_id} must have count >= 1")]
    RecipeStackCountMustBePositive { list: &'static str, item_id: u32 },
    #[error("power recipe ingredient item id {0} is out of bounds")]
    UnknownPowerRecipeIngredientItemId(u32),
    #[error("power recipe ingredient item id {item_id} must have count >= 1")]
    PowerRecipeIngredientCountMustBePositive { item_id: u32 },
    #[error("power recipe power_w must be >= 1")]
    PowerRecipePowerMustBePositive,
    #[error("power recipe time_s must be >= 1")]
    PowerRecipeTimeMustBePositive,
}
