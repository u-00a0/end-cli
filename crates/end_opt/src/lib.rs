//! Two-stage optimization for production planning.
//!
//! Stage 1 maximizes total revenue.
//! Stage 2 minimizes machine counts under a near-optimal revenue floor.

mod consts;
mod error;
mod logistics;
mod solver;
mod types;

pub use consts::LOGISTICS_EPS;
pub use error::{Error, Result};
pub use logistics::{build_item_subproblems, build_logistics_plan, solve_item_best_fit};
pub use solver::{NEAR_INT_EPS, run_two_stage};
