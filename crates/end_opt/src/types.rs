use crate::consts::LOGISTICS_EPS;
use crate::error::{Error, Result};
use end_model::{FacilityId, ItemId, OutpostId, PowerRecipeId, RecipeId};
use std::num::NonZeroU32;

/// Finite, strictly positive floating-point value.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PosF64(f64);

impl PosF64 {
    pub fn new(value: f64) -> Option<Self> {
        (value.is_finite() && value > 0.0).then_some(Self(value))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

/// Value utilization of one outpost.
#[derive(Debug, Clone)]
pub struct OutpostValue {
    pub outpost_index: OutpostId,
    /// Realized revenue on this outpost.
    pub value_per_min: f64,
    /// Theoretical cap from outpost config.
    pub cap_per_min: f64,
    /// `value_per_min / cap_per_min`, in `[0, +inf)`.
    pub ratio: f64,
}

/// One sale line contribution with quantity information.
#[derive(Debug, Clone, PartialEq)]
pub struct OutpostSaleQty<'id> {
    pub outpost_index: OutpostId,
    /// Item being sold.
    pub item: ItemId<'id>,
    /// Sold quantity in units/min.
    pub qty_per_min: PosF64,
    /// Unit price used by optimization objective.
    pub price: u32,
}

/// Machine count aggregated by facility type.
#[derive(Debug, Clone)]
pub struct FacilityMachineCount<'id> {
    /// Facility id from `Catalog.facilities`.
    pub facility: FacilityId<'id>,
    /// Integer machine count.
    pub machines: u32,
}

/// Execution and machine usage for one recipe.
#[derive(Debug, Clone)]
pub struct RecipeUsage<'id> {
    pub recipe_index: RecipeId<'id>,
    /// Integer machine count assigned to the recipe.
    pub machines: NonZeroU32,
    /// Recipe runs per minute.
    pub executions_per_min: f64,
}

/// Thermal bank deployment for one power recipe.
#[derive(Debug, Clone)]
pub struct ThermalBankUsage<'id> {
    pub power_recipe_index: PowerRecipeId<'id>,
    /// Fuel item consumed by the thermal bank.
    pub ingredient: ItemId<'id>,
    /// Number of thermal banks.
    pub banks: NonZeroU32,
    /// Per-bank power output.
    pub power_w: u32,
    /// Per-bank cycle duration in seconds.
    pub duration_s: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SupplyNodeId(u32);

impl SupplyNodeId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DemandNodeId(u32);

impl DemandNodeId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LogisticsNodeId(u32);

impl LogisticsNodeId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupplySite<'id> {
    ExternalSupply {
        item: ItemId<'id>,
    },
    RecipeOutput {
        recipe_index: RecipeId<'id>,
        item: ItemId<'id>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemandSite<'id> {
    RecipeInput {
        recipe_index: RecipeId<'id>,
        item: ItemId<'id>,
    },
    ExternalConsumption {
        item: ItemId<'id>,
    },
    OutpostSale {
        outpost_index: OutpostId,
        item: ItemId<'id>,
    },
    ThermalBankFuel {
        power_recipe_index: PowerRecipeId<'id>,
        item: ItemId<'id>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SupplyNode<'id> {
    pub id: SupplyNodeId,
    pub site: SupplySite<'id>,
    pub capacity_per_min: PosF64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DemandNode<'id> {
    pub id: DemandNodeId,
    pub site: DemandSite<'id>,
    pub demand_per_min: PosF64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemSubproblem<'id> {
    item: ItemId<'id>,
    supplies: Box<[SupplyNode<'id>]>,
    demands: Box<[DemandNode<'id>]>,
}

impl<'id> ItemSubproblem<'id> {
    pub(crate) fn new(
        item: ItemId<'id>,
        supplies: Box<[SupplyNode<'id>]>,
        demands: Box<[DemandNode<'id>]>,
    ) -> Result<Self> {
        if demands.is_empty() {
            return Err(Error::InvalidInput {
                message: format!(
                    "item {} subproblem requires at least one demand node",
                    item.as_u32()
                ).into_boxed_str(),
            });
        }
        let total_supply = supplies
            .iter()
            .map(|s| s.capacity_per_min.get())
            .sum::<f64>();
        let total_demand = demands.iter().map(|d| d.demand_per_min.get()).sum::<f64>();
        if total_supply + LOGISTICS_EPS < total_demand {
            return Err(Error::LogisticsInfeasible {
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

    pub fn item(&self) -> ItemId<'id> {
        self.item
    }

    pub fn supplies(&self) -> &[SupplyNode<'id>] {
        &self.supplies
    }

    pub fn demands(&self) -> &[DemandNode<'id>] {
        &self.demands
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogisticsNodeSite<'id> {
    ExternalSupply {
        item: ItemId<'id>,
    },
    ExternalConsumption {
        item: ItemId<'id>,
    },
    RecipeGroup {
        recipe_index: RecipeId<'id>,
    },
    OutpostSale {
        outpost_index: OutpostId,
        item: ItemId<'id>,
    },
    ThermalBankGroup {
        power_recipe_index: PowerRecipeId<'id>,
        item: ItemId<'id>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogisticsNode<'id> {
    pub id: LogisticsNodeId,
    pub site: LogisticsNodeSite<'id>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemFlowEdge<'id> {
    pub item: ItemId<'id>,
    pub from: SupplyNodeId,
    pub to: DemandNodeId,
    pub flow_per_min: PosF64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemFlowPlan<'id> {
    pub item: ItemId<'id>,
    pub edges: Box<[ItemFlowEdge<'id>]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogisticsEdge<'id> {
    pub item: ItemId<'id>,
    pub from: LogisticsNodeId,
    pub to: LogisticsNodeId,
    pub flow_per_min: PosF64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogisticsPlan<'id> {
    pub nodes: Box<[LogisticsNode<'id>]>,
    pub edges: Box<[LogisticsEdge<'id>]>,
}

/// Remaining slack for each externally supplied item.
#[derive(Debug, Clone)]
pub struct ExternalSupplySlack<'id> {
    /// Item id from `Catalog.items`.
    pub item: ItemId<'id>,
    /// Remaining quantity in the item balance constraint.
    pub slack_per_min: f64,
    /// Input external supply configured for this item.
    pub supply_per_min: f64,
}

/// Result of one optimization stage.
#[derive(Debug, Clone)]
pub struct StageSolution<'id> {
    /// Core generation capacity in watts.
    pub p_core_w: u32,
    /// External power consumption in watts.
    pub p_ext_w: u32,
    /// Total revenue objective value.
    pub revenue_per_min: f64,
    /// Sum of all production machines.
    pub total_machines: u32,
    /// Sum of all thermal banks.
    pub total_thermal_banks: u32,
    /// Total power generation.
    pub power_gen_w: i64,
    /// Total power usage.
    pub power_use_w: i64,
    /// `power_gen_w - power_use_w`.
    pub power_margin_w: i64,
    /// Per-outpost value realization.
    pub outpost_values: Box<[OutpostValue]>,
    /// Full sale lines with quantities and unit prices.
    /// Used to reconstruct logistics demands and derive top-sales summaries.
    pub outpost_sales_qty: Box<[OutpostSaleQty<'id>]>,
    /// Machine counts by facility.
    pub machines_by_facility: Box<[FacilityMachineCount<'id>]>,
    /// Top recipes by machine count.
    pub recipes_used: Box<[RecipeUsage<'id>]>,
    /// Thermal bank allocations.
    pub thermal_banks_used: Box<[ThermalBankUsage<'id>]>,
    /// Slack information for externally supplied items.
    pub external_supply_slack: Box<[ExternalSupplySlack<'id>]>,
}

/// Combined output for stage 1 and stage 2.
#[derive(Debug, Clone)]
pub struct OptimizationResult<'id> {
    /// Stage 1: max revenue.
    pub stage1: StageSolution<'id>,
    /// Stage 2: min machine counts with revenue floor.
    pub stage2: StageSolution<'id>,
    /// Machine-granularity logistics flow plan derived from stage 2.
    pub logistics: LogisticsPlan<'id>,
}
