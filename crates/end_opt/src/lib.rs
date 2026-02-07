use end_model::{AicInputs, Catalog, FacilityId, FacilityKind, ItemId};
use good_lp::{
    Expression, ResolutionError, Solution, SolverModel, Variable, constraint, default_solver,
    variable, variables,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use thiserror::Error;

pub const NEAR_INT_EPS: f64 = 1e-6;
pub const STAGE2_REVENUE_FLOOR_REL_EPS: f64 = 1e-6;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("solver failed: {source}")]
    Solver {
        #[source]
        source: ResolutionError,
    },

    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    #[error("missing machine power for facility `{facility}`")]
    MissingMachinePower { facility: String },

    #[error(
        "value not near integer for `{var_name}`: value={value}, nearest={nearest}, delta={delta}, eps={eps}"
    )]
    NotNearInt {
        var_name: String,
        value: f64,
        nearest: f64,
        delta: f64,
        eps: f64,
    },

    #[error("value out of range for `{var_name}`: {value}")]
    OutOfRange { var_name: String, value: f64 },

    #[error("internal error: {message}")]
    Internal { message: String },
}

#[derive(Debug, Clone, Copy)]
enum StageObjective {
    MaxRevenue,
    MinMachines { revenue_floor_per_min: f64 },
}

#[derive(Debug, Clone)]
struct OutpostVars {
    outpost_index: usize,
    money_cap_per_hour: u32,
    sell_lines: Vec<(ItemId, u32, Variable)>,
}

#[derive(Debug, Clone)]
struct RecipeVars {
    recipe_index: usize,
    facility: FacilityId,
    x: Variable,
    y: Variable,
    throughput_per_min: f64,
    net: HashMap<ItemId, f64>,
}

#[derive(Debug, Clone)]
struct PowerVars {
    power_recipe_index: usize,
    ingredient: ItemId,
    power_w: u32,
    duration_s: u32,
    z: Variable,
}

#[derive(Debug, Clone)]
pub struct SolveInputs {
    pub p_core_w: u32,
    pub aic: AicInputs,
}

#[derive(Debug, Clone)]
pub struct OutpostValue {
    pub outpost_index: usize,
    pub value_per_min: f64,
    pub cap_per_min: f64,
    pub ratio: f64,
}

#[derive(Debug, Clone)]
pub struct SaleValue {
    pub outpost_index: usize,
    pub item: ItemId,
    pub value_per_min: f64,
}

#[derive(Debug, Clone)]
pub struct FacilityMachineCount {
    pub facility: FacilityId,
    pub machines: u32,
}

#[derive(Debug, Clone)]
pub struct RecipeUsage {
    pub recipe_index: usize,
    pub machines: u32,
    pub executions_per_min: f64,
}

#[derive(Debug, Clone)]
pub struct ThermalBankUsage {
    pub power_recipe_index: usize,
    pub ingredient: ItemId,
    pub banks: u32,
    pub power_w: u32,
    pub duration_s: u32,
}

#[derive(Debug, Clone)]
pub struct ExternalSupplySlack {
    pub item: ItemId,
    pub slack_per_min: f64,
    pub supply_per_min: f64,
}

#[derive(Debug, Clone)]
pub struct StageSolution {
    pub p_core_w: u32,
    pub p_ext_w: u32,
    pub revenue_per_min: f64,
    pub total_machines: u32,
    pub total_thermal_banks: u32,
    pub power_gen_w: i64,
    pub power_use_w: i64,
    pub power_margin_w: i64,
    pub outpost_values: Vec<OutpostValue>,
    pub top_sales: Vec<SaleValue>,
    pub machines_by_facility: Vec<FacilityMachineCount>,
    pub recipes_used: Vec<RecipeUsage>,
    pub thermal_banks_used: Vec<ThermalBankUsage>,
    pub external_supply_slack: Vec<ExternalSupplySlack>,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub stage1: StageSolution,
    pub stage2: StageSolution,
}

pub fn run_two_stage(catalog: &Catalog, inputs: &SolveInputs) -> Result<OptimizationResult> {
    if catalog.recipes.is_empty() {
        return Err(Error::InvalidInput {
            message: "catalog.recipes must not be empty".to_string(),
        });
    }

    let stage1 = solve_stage(catalog, inputs, StageObjective::MaxRevenue)?;

    let rel_eps = STAGE2_REVENUE_FLOOR_REL_EPS * stage1.revenue_per_min.max(1.0);
    let revenue_floor_per_min = (stage1.revenue_per_min - rel_eps).max(0.0);
    let stage2 = solve_stage(
        catalog,
        inputs,
        StageObjective::MinMachines {
            revenue_floor_per_min,
        },
    )?;

    Ok(OptimizationResult { stage1, stage2 })
}

fn solve_stage(
    catalog: &Catalog,
    inputs: &SolveInputs,
    objective: StageObjective,
) -> Result<StageSolution> {
    let external_supply = &inputs.aic.supply_per_min;

    let mut vars = variables!();

    let mut recipe_vars = Vec::with_capacity(catalog.recipes.len());
    for (idx, recipe) in catalog.recipes.iter().enumerate() {
        let x = vars.add(variable().min(0.0));
        let y = vars.add(variable().integer().min(0.0));

        let time_s = recipe.time_s as f64;
        if time_s <= 0.0 {
            return Err(Error::InvalidInput {
                message: format!("recipe[{idx}] has non-positive time_s {}", recipe.time_s),
            });
        }

        let mut net: HashMap<ItemId, f64> = HashMap::new();
        for stack in &recipe.ingredients {
            *net.entry(stack.item).or_insert(0.0) -= stack.count as f64;
        }
        for stack in &recipe.products {
            *net.entry(stack.item).or_insert(0.0) += stack.count as f64;
        }

        recipe_vars.push(RecipeVars {
            recipe_index: idx,
            facility: recipe.facility,
            x,
            y,
            throughput_per_min: 60.0 / time_s,
            net,
        });
    }

    let mut power_vars = Vec::with_capacity(catalog.power_recipes.len());
    for (idx, p) in catalog.power_recipes.iter().enumerate() {
        let z = vars.add(variable().integer().min(0.0));
        power_vars.push(PowerVars {
            power_recipe_index: idx,
            ingredient: p.ingredient.item,
            power_w: p.power_w,
            duration_s: p.time_s,
            z,
        });
    }

    let mut outpost_vars = Vec::with_capacity(inputs.aic.outposts.len());
    for (idx, outpost) in inputs.aic.outposts.iter().enumerate() {
        let mut pairs: Vec<(ItemId, u32)> = outpost.prices.iter().map(|(k, v)| (*k, *v)).collect();
        pairs.sort_by_key(|(item, _)| *item);
        let sell_lines = pairs
            .into_iter()
            .map(|(item, price)| (item, price, vars.add(variable().min(0.0))))
            .collect();

        outpost_vars.push(OutpostVars {
            outpost_index: idx,
            money_cap_per_hour: outpost.money_cap_per_hour,
            sell_lines,
        });
    }

    let mut items = BTreeSet::new();
    items.extend(external_supply.keys().copied());
    for rv in &recipe_vars {
        items.extend(rv.net.keys().copied());
    }
    for ov in &outpost_vars {
        items.extend(ov.sell_lines.iter().map(|(item, _, _)| *item));
    }
    for pv in &power_vars {
        items.insert(pv.ingredient);
    }

    let mut revenue = Expression::default();
    for ov in &outpost_vars {
        for (_, price, qty) in &ov.sell_lines {
            revenue += *price as f64 * *qty;
        }
    }

    let mut total_machines_expr = Expression::default();
    for rv in &recipe_vars {
        total_machines_expr += 1.0 * rv.y;
    }

    let mut total_thermal_banks_expr = Expression::default();
    for pv in &power_vars {
        total_thermal_banks_expr += 1.0 * pv.z;
    }

    let mut power_gen = Expression::default();
    power_gen += inputs.p_core_w as f64;
    for pv in &power_vars {
        power_gen += pv.power_w as f64 * pv.z;
    }

    let mut power_use = Expression::default();
    power_use += inputs.aic.external_power_consumption_w as f64;
    for rv in &recipe_vars {
        let facility = catalog
            .facility(rv.facility)
            .ok_or_else(|| Error::Internal {
                message: format!("unknown facility id {}", rv.facility.0),
            })?;
        let machine_power_w = facility.power_w.ok_or_else(|| Error::MissingMachinePower {
            facility: facility.key.clone(),
        })?;
        if facility.kind != FacilityKind::Machine {
            return Err(Error::InvalidInput {
                message: format!(
                    "recipe[{}] references non-machine facility `{}`",
                    rv.recipe_index, facility.key
                ),
            });
        }
        power_use += machine_power_w as f64 * rv.y;
    }

    let mut balance_exprs: BTreeMap<ItemId, Expression> = BTreeMap::new();
    for item in items {
        let mut balance = Expression::default();
        balance += external_supply.get(&item).copied().unwrap_or(0) as f64;

        for rv in &recipe_vars {
            if let Some(delta) = rv.net.get(&item) {
                balance += *delta * rv.x;
            }
        }

        for ov in &outpost_vars {
            for (sell_item, _, qty) in &ov.sell_lines {
                if *sell_item == item {
                    balance -= 1.0 * *qty;
                }
            }
        }

        for pv in &power_vars {
            if pv.ingredient == item {
                let consume_per_min = 60.0 / pv.duration_s as f64;
                balance -= consume_per_min * pv.z;
            }
        }

        balance_exprs.insert(item, balance);
    }

    let mut model = match objective {
        StageObjective::MaxRevenue => vars.maximise(revenue.clone()).using(default_solver),
        StageObjective::MinMachines {
            revenue_floor_per_min,
        } => vars
            .minimise(total_machines_expr.clone() + total_thermal_banks_expr.clone())
            .using(default_solver)
            .with(constraint!(revenue.clone() >= revenue_floor_per_min)),
    };

    for ov in &outpost_vars {
        let mut outpost_value = Expression::default();
        for (_, price, qty) in &ov.sell_lines {
            outpost_value += *price as f64 * *qty;
        }
        model = model.with(constraint!(
            outpost_value <= ov.money_cap_per_hour as f64 / 60.0
        ));
    }

    model = model.with(constraint!(power_gen.clone() >= power_use.clone()));

    for rv in &recipe_vars {
        model = model.with(constraint!(rv.x <= rv.throughput_per_min * rv.y));
    }

    for expr in balance_exprs.values() {
        model = model.with(constraint!(expr.clone() >= 0.0));
    }

    let solution = model.solve().map_err(|source| Error::Solver { source })?;

    let revenue_per_min = solution.eval(&revenue);
    let total_machines = near_u32("total_machines", solution.eval(&total_machines_expr))?;
    let total_thermal_banks = near_u32(
        "total_thermal_banks",
        solution.eval(&total_thermal_banks_expr),
    )?;

    let power_gen_w = near_i64("power_gen_w", solution.eval(&power_gen))?;
    let power_use_w = near_i64("power_use_w", solution.eval(&power_use))?;
    let power_margin_w = power_gen_w - power_use_w;

    let mut outpost_values = Vec::new();
    let mut top_sales = Vec::new();

    for ov in &outpost_vars {
        let mut value_per_min = 0.0;
        for (item, price, qty) in &ov.sell_lines {
            let qty_value = solution.value(*qty);
            if qty_value <= 1e-9 {
                continue;
            }
            let value = *price as f64 * qty_value;
            if value <= 1e-9 {
                continue;
            }
            value_per_min += value;
            top_sales.push(SaleValue {
                outpost_index: ov.outpost_index,
                item: *item,
                value_per_min: value,
            });
        }

        let cap_per_min = ov.money_cap_per_hour as f64 / 60.0;
        let ratio = if cap_per_min > 0.0 {
            value_per_min / cap_per_min
        } else {
            0.0
        };

        outpost_values.push(OutpostValue {
            outpost_index: ov.outpost_index,
            value_per_min,
            cap_per_min,
            ratio,
        });
    }

    top_sales.sort_by(|a, b| b.value_per_min.total_cmp(&a.value_per_min));
    top_sales.truncate(10);

    let mut machines_by_facility_map: HashMap<FacilityId, u32> = HashMap::new();
    let mut recipes_used = Vec::new();
    for rv in &recipe_vars {
        let machines = near_u32(
            &format!("recipes[{}].machines", rv.recipe_index),
            solution.value(rv.y),
        )?;
        let executions_per_min = solution.value(rv.x);
        if machines > 0 {
            *machines_by_facility_map.entry(rv.facility).or_insert(0) += machines;
            recipes_used.push(RecipeUsage {
                recipe_index: rv.recipe_index,
                machines,
                executions_per_min,
            });
        }
    }

    let mut machines_by_facility = machines_by_facility_map
        .into_iter()
        .map(|(facility, machines)| FacilityMachineCount { facility, machines })
        .collect::<Vec<_>>();
    machines_by_facility.sort_by(|a, b| b.machines.cmp(&a.machines));

    recipes_used.sort_by(|a, b| b.machines.cmp(&a.machines));
    recipes_used.truncate(20);

    let mut thermal_banks_used = Vec::new();
    for pv in &power_vars {
        let banks = near_u32(
            &format!("power_recipes[{}].banks", pv.power_recipe_index),
            solution.value(pv.z),
        )?;
        if banks > 0 {
            thermal_banks_used.push(ThermalBankUsage {
                power_recipe_index: pv.power_recipe_index,
                ingredient: pv.ingredient,
                banks,
                power_w: pv.power_w,
                duration_s: pv.duration_s,
            });
        }
    }
    thermal_banks_used.sort_by(|a, b| b.banks.cmp(&a.banks));

    let mut external_supply_slack = Vec::new();
    for (item, supply) in external_supply {
        if let Some(expr) = balance_exprs.get(item) {
            external_supply_slack.push(ExternalSupplySlack {
                item: *item,
                slack_per_min: solution.eval(expr),
                supply_per_min: *supply as f64,
            });
        }
    }
    external_supply_slack.sort_by(|a, b| a.slack_per_min.total_cmp(&b.slack_per_min));

    Ok(StageSolution {
        p_core_w: inputs.p_core_w,
        p_ext_w: inputs.aic.external_power_consumption_w,
        revenue_per_min,
        total_machines,
        total_thermal_banks,
        power_gen_w,
        power_use_w,
        power_margin_w,
        outpost_values,
        top_sales,
        machines_by_facility,
        recipes_used,
        thermal_banks_used,
        external_supply_slack,
    })
}

fn near_u32(var_name: &str, value: f64) -> Result<u32> {
    if !value.is_finite() {
        return Err(Error::OutOfRange {
            var_name: var_name.to_string(),
            value,
        });
    }

    let nearest = value.round();
    let delta = (value - nearest).abs();
    if delta > NEAR_INT_EPS {
        return Err(Error::NotNearInt {
            var_name: var_name.to_string(),
            value,
            nearest,
            delta,
            eps: NEAR_INT_EPS,
        });
    }

    if nearest < 0.0 || nearest > u32::MAX as f64 {
        return Err(Error::OutOfRange {
            var_name: var_name.to_string(),
            value,
        });
    }

    Ok(nearest as u32)
}

fn near_i64(var_name: &str, value: f64) -> Result<i64> {
    if !value.is_finite() {
        return Err(Error::OutOfRange {
            var_name: var_name.to_string(),
            value,
        });
    }

    let nearest = value.round();
    let delta = (value - nearest).abs();
    if delta > NEAR_INT_EPS {
        return Err(Error::NotNearInt {
            var_name: var_name.to_string(),
            value,
            nearest,
            delta,
            eps: NEAR_INT_EPS,
        });
    }

    if nearest < i64::MIN as f64 || nearest > i64::MAX as f64 {
        return Err(Error::OutOfRange {
            var_name: var_name.to_string(),
            value,
        });
    }

    Ok(nearest as i64)
}
