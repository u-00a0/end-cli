mod builder;
mod types;

pub use builder::CatalogBuilder;
pub use types::{
    FacilityDef, FacilityId, ItemDef, ItemId, PowerRecipe, PowerRecipeId, Recipe, RecipeId, Stack,
    ThermalBankDef,
};

use std::collections::HashMap;

use generativity::{Guard, Id};

use crate::Key;

/// Canonical in-memory model resolved from TOML inputs.
///
/// ```compile_fail
/// use end_model::{Catalog, DisplayName, ItemDef, ItemId, Key, ThermalBankDef};
/// use generativity::make_guard;
///
/// fn key(value: &str) -> Key {
///     value.try_into().unwrap()
/// }
///
/// fn name(value: &str) -> DisplayName {
///     value.try_into().unwrap()
/// }
///
/// fn item_name<'id>(catalog: &Catalog<'id>, item: ItemId<'id>) -> String {
///     catalog.item(item).en.as_str().to_string()
/// }
///
/// fn with_catalog<R>(
///     item_key: &'static str,
///     bank_key: &'static str,
///     f: impl for<'id> FnOnce(Catalog<'id>, ItemId<'id>) -> R,
/// ) -> R {
///     make_guard!(guard);
///     let mut b = Catalog::builder(guard);
///     let item = b
///         .add_item(ItemDef {
///             key: key(item_key),
///             en: name("Ore"),
///             zh: name("Ore"),
///         })
///         .unwrap();
///     let b = b
///         .add_thermal_bank(ThermalBankDef {
///             key: key(bank_key),
///             en: name("Bank"),
///             zh: name("Bank"),
///         })
///         .unwrap();
///     let catalog = b.build();
///     f(catalog, item)
/// }
///
/// with_catalog("ore-1", "bank-1", |catalog_1, ore_1| {
///     with_catalog("ore-2", "bank-2", |catalog_2, _| {
///         let _ = catalog_1;
///         let _ = item_name(&catalog_2, ore_1);
///     });
/// });
/// ```
///
/// ## Design notes
///
/// This type intentionally keeps its fields private so that the internal indices (`key -> id`
/// lookups and the `thermal_bank` definition) cannot be desynchronized from the backing vectors by
/// accident.
///
/// Construct a catalog via [`Catalog::builder`] / [`CatalogBuilder::build`].
#[derive(Debug, Clone)]
pub struct Catalog<'id> {
    brand: Id<'id>,
    items: Vec<ItemDef>,
    facilities: Vec<FacilityDef>,
    recipes: Vec<Recipe<'id>>,
    power_recipes: Vec<PowerRecipe<'id>>,
    item_index: HashMap<Key, ItemId<'id>>,
    facility_index: HashMap<Key, FacilityId<'id>>,
    thermal_bank: ThermalBankDef,
}

/// Core power generation capacity in watts (fixed game constant).
const CORE_POWER_W: u32 = 200;

impl<'id> Catalog<'id> {
    /// Starts building a self-consistent [`Catalog`].
    pub fn builder(guard: Guard<'id>) -> CatalogBuilder<'id> {
        CatalogBuilder::new(guard)
    }

    /// Returns the core generation capacity in watts (fixed at 200W).
    pub fn core_power_w(&self) -> u32 {
        CORE_POWER_W
    }

    /// Returns item metadata by id.
    pub fn item(&self, id: ItemId<'id>) -> &ItemDef {
        let index = id.index();
        debug_assert!(
            index < self.items.len(),
            "invalid item id {} for catalog with {} items",
            id.as_u32(),
            self.items.len()
        );
        // SAFETY: `ItemId` values are minted by `CatalogBuilder` for this catalog brand.
        unsafe { self.items.get_unchecked(index) }
    }

    /// Returns facility metadata by id.
    pub fn facility(&self, id: FacilityId<'id>) -> &FacilityDef {
        let index = id.index();
        debug_assert!(
            index < self.facilities.len(),
            "invalid facility id {} for catalog with {} facilities",
            id.as_u32(),
            self.facilities.len()
        );
        // SAFETY: `FacilityId` values are minted by `CatalogBuilder` for this catalog brand.
        unsafe { self.facilities.get_unchecked(index) }
    }

    /// Returns a recipe by its id.
    ///
    /// The optimizer and report layers use this id as a stable reference.
    pub fn recipe(&self, id: RecipeId<'id>) -> &Recipe<'id> {
        let index = id.index();
        debug_assert!(
            index < self.recipes.len(),
            "invalid recipe id {} for catalog with {} recipes",
            id.as_u32(),
            self.recipes.len()
        );
        // SAFETY: `RecipeId` values are minted by `CatalogBuilder` for this catalog brand.
        unsafe { self.recipes.get_unchecked(index) }
    }

    /// Returns a power recipe by its id.
    pub fn power_recipe(&self, id: PowerRecipeId<'id>) -> &PowerRecipe<'id> {
        let index = id.index();
        debug_assert!(
            index < self.power_recipes.len(),
            "invalid power recipe id {} for catalog with {} power recipes",
            id.as_u32(),
            self.power_recipes.len()
        );
        // SAFETY: `PowerRecipeId` values are minted by `CatalogBuilder` for this catalog brand.
        unsafe { self.power_recipes.get_unchecked(index) }
    }

    /// Returns the unique thermal bank definition.
    pub fn thermal_bank(&self) -> &ThermalBankDef {
        &self.thermal_bank
    }

    /// Returns all items in id order.
    pub fn items(&self) -> &[ItemDef] {
        &self.items
    }

    /// Returns all facilities in id order.
    pub fn facilities(&self) -> &[FacilityDef] {
        &self.facilities
    }

    /// Returns all production recipes.
    pub fn recipes(&self) -> &[Recipe<'id>] {
        &self.recipes
    }

    /// Returns all production recipes paired with their stable ids.
    pub fn recipes_with_id(&self) -> impl Iterator<Item = (RecipeId<'id>, &Recipe<'id>)> + '_ {
        let brand = self.brand;
        self.recipes
            .iter()
            .enumerate()
            .map(move |(index, recipe)| (RecipeId::from_index(index, brand), recipe))
    }

    /// Returns all thermal-bank power recipes.
    pub fn power_recipes(&self) -> &[PowerRecipe<'id>] {
        &self.power_recipes
    }

    /// Returns all thermal-bank power recipes paired with their stable ids.
    pub fn power_recipes_with_id(
        &self,
    ) -> impl Iterator<Item = (PowerRecipeId<'id>, &PowerRecipe<'id>)> + '_ {
        let brand = self.brand;
        self.power_recipes
            .iter()
            .enumerate()
            .map(move |(index, recipe)| (PowerRecipeId::from_index(index, brand), recipe))
    }

    /// Resolves an item key into an [`ItemId`].
    pub fn item_id(&self, key: &str) -> Option<ItemId<'id>> {
        self.item_index.get(key).copied()
    }

    /// Resolves a facility key into a [`FacilityId`].
    pub fn facility_id(&self, key: &str) -> Option<FacilityId<'id>> {
        self.facility_index.get(key).copied()
    }
}
