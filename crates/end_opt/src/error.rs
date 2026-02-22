use good_lp::ResolutionError;
use thiserror::Error;

/// Result alias for optimizer operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors raised while building or solving optimization models.
#[derive(Debug, Error)]
pub enum Error {
    #[error("HiGHS Solver failed")]
    Solver {
        #[source]
        source: ResolutionError,
    },

    #[error("Invalid input: {message}")]
    InvalidInput { message: Box<str> },

    #[error(
        "Value not near integer for `{var_name}`: value={value}, nearest={nearest}, delta={delta}, eps={eps}"
    )]
    NotNearInt {
        var_name: Box<str>,
        value: f64,
        nearest: f64,
        delta: f64,
        eps: f64,
    },

    #[error("Value out of range for `{var_name}`: {value}")]
    OutOfRange { var_name: Box<str>, value: f64 },

    #[error("Expected strictly positive finite flow for `{context}`, got {value}")]
    InvalidPositiveFlow { context: Box<str>, value: f64 },

    #[error(
        "Logistics infeasible for item {item}: total supply {total_supply_per_min} < total demand {total_demand_per_min}"
    )]
    LogisticsInfeasible {
        item: u32,
        total_supply_per_min: f64,
        total_demand_per_min: f64,
    },
}
