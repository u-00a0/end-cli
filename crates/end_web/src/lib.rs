mod api;
mod dto;
mod error;
mod ffi;
mod lang;

pub use api::{bootstrap, solve_from_aic_toml};
pub use dto::{
    BootstrapPayload, CatalogDto, CatalogItemDto, ExternalSupplySlackDto, FacilityUsageDto,
    LogisticsEdgeDto, LogisticsGraphDto, LogisticsItemSummaryDto, LogisticsNodeDto,
    OutpostValueDto, SaleValueDto, SolvePayload, SummaryDto,
};
pub use error::{Error, Result};
pub use ffi::{end_web_bootstrap, end_web_free_slice, end_web_solve_from_aic_toml};
pub use lang::Lang;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{Lang, bootstrap, solve_from_aic_toml};
    use end_io::{default_aic_toml, load_catalog};
    use generativity::make_guard;

    #[test]
    fn bootstrap_returns_catalog_and_default_aic() {
        let payload = bootstrap(Lang::Zh).expect("bootstrap should succeed");
        assert!(
            payload
                .default_aic_toml
                .contains("external_power_consumption_w"),
            "default aic should include external power field"
        );
        assert!(
            payload
                .default_aic_toml
                .contains("external_consumption_per_min"),
            "default aic should include external consumption field"
        );
        assert!(
            !payload.catalog.items.is_empty(),
            "catalog items should not be empty"
        );
    }

    #[test]
    fn solve_runs_with_builtin_default_aic() {
        make_guard!(guard);
        let catalog = load_catalog(None, guard).expect("builtin catalog should load");
        let aic_toml = default_aic_toml(&catalog).expect("default aic should build");

        let payload = solve_from_aic_toml(Lang::Zh, &aic_toml).expect("solve should succeed");
        assert!(
            payload.summary.stage2_revenue_per_min >= 0.0,
            "revenue should be non-negative"
        );
        assert!(
            !payload.logistics_graph.nodes.is_empty(),
            "default scenario should include logistics nodes"
        );
        assert!(
            payload
                .logistics_graph
                .nodes
                .iter()
                .any(|node| node.kind.as_ref() == "external_consumption"),
            "default scenario should include external consumption nodes"
        );
    }

    #[test]
    fn solve_rejects_invalid_toml() {
        let err = solve_from_aic_toml(Lang::Zh, "external_power_consumption_w = 'oops'")
            .expect_err("invalid toml should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("Aic parse failed"),
            "error message should mention aic parse"
        );
    }
}
