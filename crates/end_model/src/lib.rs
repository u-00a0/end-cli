use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const P_CORE_W: u32 = 200;
pub const DEFAULT_EXTERNAL_POWER_CONSUMPTION_W: u32 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FacilityId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacilityKind {
    Machine,
    ThermalBank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub key: String,
    pub en: String,
    pub zh: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityDef {
    pub key: String,
    pub kind: FacilityKind,
    pub power_w: Option<u32>,
    pub en: String,
    pub zh: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stack {
    pub item: ItemId,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub facility: FacilityId,
    pub time_s: u32,
    pub ingredients: Vec<Stack>,
    pub products: Vec<Stack>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PowerRecipe {
    pub ingredient: Stack,
    pub power_w: u32,
    pub time_s: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub items: Vec<ItemDef>,
    pub facilities: Vec<FacilityDef>,
    pub recipes: Vec<Recipe>,
    pub power_recipes: Vec<PowerRecipe>,
    pub item_index: HashMap<String, ItemId>,
    pub facility_index: HashMap<String, FacilityId>,
    pub thermal_bank: FacilityId,
}

impl Catalog {
    pub fn item(&self, id: ItemId) -> Option<&ItemDef> {
        self.items.get(id.0 as usize)
    }

    pub fn facility(&self, id: FacilityId) -> Option<&FacilityDef> {
        self.facilities.get(id.0 as usize)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutpostInput {
    pub key: String,
    pub en: Option<String>,
    pub zh: Option<String>,
    pub money_cap_per_hour: u32,
    pub prices: HashMap<ItemId, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AicInputs {
    pub external_power_consumption_w: u32,
    pub supply_per_min: HashMap<ItemId, u32>,
    pub outposts: Vec<OutpostInput>,
}
