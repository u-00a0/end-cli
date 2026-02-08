use std::collections::HashMap;

/// Base/core generation capacity (watts) used by the default CLI flow.
pub const P_CORE_W: u32 = 200;
/// Default external power consumption (watts) used by generated example inputs.
pub const DEFAULT_EXTERNAL_POWER_CONSUMPTION_W: u32 = 300;

/// Stable identifier for an item in [`Catalog`].
///
/// This is an *opaque* id: external callers cannot construct it from a raw `u32`.
/// IDs are minted by [`CatalogBuilder`] when building a self-consistent [`Catalog`].
///
/// Note: an `ItemId` is only meaningful *relative to a specific* [`Catalog`] instance.
/// Even if you can obtain the underlying number (via [`ItemId::as_u32`]), that does
/// **not** mean it is valid in another catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(u32);

impl ItemId {
    /// Returns the underlying numeric representation.
    ///
    /// This is intended for diagnostics/logging and serialization of *derived* data.
    /// It does **not** make the id safe to construct externally.
    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }

    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// Stable identifier for a facility in [`Catalog`].
///
/// Like [`ItemId`], this is an opaque id minted by [`CatalogBuilder`], and it is only
/// meaningful relative to the catalog it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FacilityId(u32);

impl FacilityId {
    /// Returns the underlying numeric representation (for diagnostics/logging only).
    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }

    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// Facility category used by the optimizer and report layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FacilityKind {
    Machine,
    ThermalBank,
}

/// Item metadata and display texts.
#[derive(Debug, Clone)]
pub struct ItemDef {
    pub key: String,
    pub en: String,
    pub zh: String,
}

/// Facility metadata and display texts.
#[derive(Debug, Clone)]
pub struct FacilityDef {
    pub key: String,
    pub kind: FacilityKind,
    pub power_w: Option<u32>,
    pub en: String,
    pub zh: String,
}

/// `(item, count)` pair used in recipes.
#[derive(Debug, Clone, Copy)]
pub struct Stack {
    pub item: ItemId,
    pub count: u32,
}

/// Production recipe definition.
#[derive(Debug, Clone)]
pub struct Recipe {
    pub facility: FacilityId,
    pub time_s: u32,
    pub ingredients: Vec<Stack>,
    pub products: Vec<Stack>,
}

/// Thermal-bank power recipe definition.
#[derive(Debug, Clone, Copy)]
pub struct PowerRecipe {
    pub ingredient: Stack,
    pub power_w: u32,
    pub time_s: u32,
}

/// Canonical in-memory model resolved from TOML inputs.
///
/// ## Design notes
///
/// This type intentionally keeps its fields private so that the internal indices
/// (`key -> id` lookups and the `thermal_bank` id) cannot be desynchronized from the
/// backing vectors by accident.
///
/// Construct a catalog via [`Catalog::builder`] / [`CatalogBuilder::build`].
#[derive(Debug, Clone)]
pub struct Catalog {
    items: Vec<ItemDef>,
    facilities: Vec<FacilityDef>,
    recipes: Vec<Recipe>,
    power_recipes: Vec<PowerRecipe>,
    item_index: HashMap<String, ItemId>,
    facility_index: HashMap<String, FacilityId>,
    thermal_bank: FacilityId,
}

impl Catalog {
    /// Starts building a self-consistent [`Catalog`].
    pub fn builder() -> CatalogBuilder {
        CatalogBuilder::new()
    }

    /// Returns item metadata by id.
    pub fn item(&self, id: ItemId) -> Option<&ItemDef> {
        self.items.get(id.index())
    }

    /// Returns facility metadata by id.
    pub fn facility(&self, id: FacilityId) -> Option<&FacilityDef> {
        self.facilities.get(id.index())
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
    pub fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    /// Returns a recipe by its index in [`Catalog::recipes`].
    ///
    /// The optimizer and report layers use this index as a stable reference.
    pub fn recipe(&self, index: usize) -> Option<&Recipe> {
        self.recipes.get(index)
    }

    /// Returns all thermal-bank power recipes.
    pub fn power_recipes(&self) -> &[PowerRecipe] {
        &self.power_recipes
    }

    /// Returns a power recipe by its index in [`Catalog::power_recipes`].
    pub fn power_recipe(&self, index: usize) -> Option<&PowerRecipe> {
        self.power_recipes.get(index)
    }

    /// Returns the facility id of the unique thermal bank facility.
    pub fn thermal_bank(&self) -> FacilityId {
        self.thermal_bank
    }

    /// Resolves an item key into an [`ItemId`].
    pub fn item_id(&self, key: &str) -> Option<ItemId> {
        self.item_index.get(key).copied()
    }

    /// Resolves a facility key into a [`FacilityId`].
    pub fn facility_id(&self, key: &str) -> Option<FacilityId> {
        self.facility_index.get(key).copied()
    }
}

/// Builder for [`Catalog`].
///
/// This is the only supported way to construct a catalog, because it:
/// - Mints opaque ids (`ItemId`/`FacilityId`) in a consistent order.
/// - Maintains key->id indices (`item_index`, `facility_index`).
/// - Enforces catalog-level invariants.
#[derive(Debug, Default)]
pub struct CatalogBuilder {
    items: Vec<ItemDef>,
    facilities: Vec<FacilityDef>,
    recipes: Vec<Recipe>,
    power_recipes: Vec<PowerRecipe>,
    item_index: HashMap<String, ItemId>,
    facility_index: HashMap<String, FacilityId>,
    thermal_bank: Option<FacilityId>,
}

impl CatalogBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an item definition and returns its newly assigned [`ItemId`].
    ///
    /// Item keys must be unique.
    pub fn add_item(&mut self, def: ItemDef) -> Result<ItemId, CatalogBuildError> {
        if self.item_index.contains_key(def.key.as_str()) {
            return Err(CatalogBuildError::DuplicateItemKey(def.key));
        }
        let id = ItemId::from_index(self.items.len());
        self.item_index.insert(def.key.clone(), id);
        self.items.push(def);
        Ok(id)
    }

    /// Adds a facility definition and returns its newly assigned [`FacilityId`].
    ///
    /// Facility keys must be unique. Exactly one facility with kind
    /// [`FacilityKind::ThermalBank`] is required to build a [`Catalog`].
    pub fn add_facility(&mut self, def: FacilityDef) -> Result<FacilityId, CatalogBuildError> {
        if self.facility_index.contains_key(def.key.as_str()) {
            return Err(CatalogBuildError::DuplicateFacilityKey(def.key));
        }
        let id = FacilityId::from_index(self.facilities.len());
        self.facility_index.insert(def.key.clone(), id);
        if def.kind == FacilityKind::ThermalBank {
            if self.thermal_bank.is_some() {
                return Err(CatalogBuildError::MultipleThermalBanks);
            }
            self.thermal_bank = Some(id);
        }
        self.facilities.push(def);
        Ok(id)
    }

    /// Appends a production recipe.
    ///
    /// This does not perform deep validation of cross-references; callers are expected
    /// to resolve ids via [`CatalogBuilder::item_id`] / [`CatalogBuilder::facility_id`]
    /// before constructing recipes.
    pub fn push_recipe(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    /// Appends a thermal-bank power recipe.
    pub fn push_power_recipe(&mut self, recipe: PowerRecipe) {
        self.power_recipes.push(recipe);
    }

    /// Resolves an item key into an [`ItemId`] using the current builder state.
    pub fn item_id(&self, key: &str) -> Option<ItemId> {
        self.item_index.get(key).copied()
    }

    /// Resolves a facility key into a [`FacilityId`] using the current builder state.
    pub fn facility_id(&self, key: &str) -> Option<FacilityId> {
        self.facility_index.get(key).copied()
    }

    /// Returns the thermal bank id if it has already been added.
    pub fn thermal_bank(&self) -> Option<FacilityId> {
        self.thermal_bank
    }

    /// Finalizes the builder and returns a self-consistent [`Catalog`].
    ///
    /// Fails if required invariants are not met (e.g. thermal bank missing).
    pub fn build(self) -> Result<Catalog, CatalogBuildError> {
        let thermal_bank = self
            .thermal_bank
            .ok_or(CatalogBuildError::MissingThermalBank)?;
        Ok(Catalog {
            items: self.items,
            facilities: self.facilities,
            recipes: self.recipes,
            power_recipes: self.power_recipes,
            item_index: self.item_index,
            facility_index: self.facility_index,
            thermal_bank,
        })
    }
}

/// Errors returned when building a [`Catalog`].
#[derive(Debug, thiserror::Error)]
pub enum CatalogBuildError {
    #[error("duplicate item key: {0}")]
    DuplicateItemKey(String),
    #[error("duplicate facility key: {0}")]
    DuplicateFacilityKey(String),
    #[error("missing thermal bank facility")]
    MissingThermalBank,
    #[error("multiple thermal banks are not allowed")]
    MultipleThermalBanks,
}

/// One outpost demand/cap configuration.
#[derive(Debug, Clone)]
pub struct OutpostInput {
    pub key: String,
    pub en: Option<String>,
    pub zh: Option<String>,
    pub money_cap_per_hour: u32,
    pub prices: HashMap<ItemId, u32>,
}

/// Full scenario inputs consumed by optimization.
#[derive(Debug, Clone)]
pub struct AicInputs {
    pub external_power_consumption_w: u32,
    pub supply_per_min: HashMap<ItemId, u32>,
    pub outposts: Vec<OutpostInput>,
}
