use end_model::{ItemId, OutpostId, PosF64, PowerRecipeId, RecipeId};
use generativity::Id;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SupplyNodeId<'rid> {
    raw: u32,
    brand: Id<'rid>,
}

impl<'rid> SupplyNodeId<'rid> {
    pub fn from_index(index: usize, brand: Id<'rid>) -> Self {
        Self {
            raw: index as u32,
            brand,
        }
    }

    pub fn as_u32(self) -> u32 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DemandNodeId<'rid> {
    raw: u32,
    brand: Id<'rid>,
}

impl<'rid> DemandNodeId<'rid> {
    pub fn from_index(index: usize, brand: Id<'rid>) -> Self {
        Self {
            raw: index as u32,
            brand,
        }
    }

    pub fn as_u32(self) -> u32 {
        self.raw
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupplySite<'cid> {
    ExternalSupply {
        item: ItemId<'cid>,
    },
    RecipeOutput {
        recipe_index: RecipeId<'cid>,
        item: ItemId<'cid>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemandSite<'cid, 'sid> {
    RecipeInput {
        recipe_index: RecipeId<'cid>,
        item: ItemId<'cid>,
    },
    ExternalConsumption {
        item: ItemId<'cid>,
    },
    OutpostSale {
        outpost_index: OutpostId<'sid>,
        item: ItemId<'cid>,
    },
    ThermalBankFuel {
        power_recipe_index: PowerRecipeId<'cid>,
        item: ItemId<'cid>,
    },
    /// Stockpile leftovers into warehouse.
    WarehouseStockpile,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SupplyNode<'cid, 'rid> {
    pub id: SupplyNodeId<'rid>,
    pub site: SupplySite<'cid>,
    pub capacity_per_min: PosF64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DemandNode<'cid, 'sid, 'rid> {
    pub id: DemandNodeId<'rid>,
    pub site: DemandSite<'cid, 'sid>,
    pub demand_per_min: PosF64,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ItemSubproblemBuildError {
    #[error("item {item} subproblem requires at least one demand node")]
    EmptyDemands { item: u32 },

    #[error(
        "Logistics infeasible for item {item}: total supply {total_supply_per_min} < total demand {total_demand_per_min}"
    )]
    Infeasible {
        item: u32,
        total_supply_per_min: f64,
        total_demand_per_min: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemSubproblem<'cid, 'sid, 'rid> {
    item: ItemId<'cid>,
    supplies: Box<[SupplyNode<'cid, 'rid>]>,
    demands: Box<[DemandNode<'cid, 'sid, 'rid>]>,
}

impl<'cid, 'sid, 'rid> ItemSubproblem<'cid, 'sid, 'rid> {
    pub fn new(
        item: ItemId<'cid>,
        supplies: Box<[SupplyNode<'cid, 'rid>]>,
        demands: Box<[DemandNode<'cid, 'sid, 'rid>]>,
        logistics_eps: f64,
    ) -> std::result::Result<Self, ItemSubproblemBuildError> {
        if demands.is_empty() {
            return Err(ItemSubproblemBuildError::EmptyDemands {
                item: item.as_u32(),
            });
        }
        let total_supply = supplies
            .iter()
            .map(|s| s.capacity_per_min.get())
            .sum::<f64>();
        let total_demand = demands.iter().map(|d| d.demand_per_min.get()).sum::<f64>();
        if total_supply + logistics_eps < total_demand {
            return Err(ItemSubproblemBuildError::Infeasible {
                item: item.as_u32(),
                total_supply_per_min: total_supply,
                total_demand_per_min: total_demand,
            });
        }
        Ok(Self {
            item,
            supplies,
            demands,
        })
    }

    pub fn item(&self) -> ItemId<'cid> {
        self.item
    }

    pub fn supplies(&self) -> &[SupplyNode<'cid, 'rid>] {
        &self.supplies
    }

    pub fn demands(&self) -> &[DemandNode<'cid, 'sid, 'rid>] {
        &self.demands
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemFlowEdge<'cid, 'rid> {
    pub item: ItemId<'cid>,
    pub from: SupplyNodeId<'rid>,
    pub to: DemandNodeId<'rid>,
    pub flow_per_min: PosF64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemFlowPlan<'cid, 'rid> {
    pub item: ItemId<'cid>,
    pub edges: Box<[ItemFlowEdge<'cid, 'rid>]>,
}
