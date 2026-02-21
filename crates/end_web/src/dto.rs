use serde::Serialize;

use crate::Result;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub default_aic_toml: String,
    pub catalog: CatalogDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDto {
    pub items: Vec<CatalogItemDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItemDto {
    pub key: String,
    pub en: String,
    pub zh: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolvePayload {
    pub report_text: String,
    pub summary: SummaryDto,
    pub logistics_graph: LogisticsGraphDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryDto {
    pub lang: &'static str,
    pub stage1_revenue_per_min: f64,
    pub stage2_revenue_per_min: f64,
    pub stage2_revenue_per_hour: f64,
    pub total_machines: u32,
    pub total_thermal_banks: u32,
    pub power_gen_w: i64,
    pub power_use_w: i64,
    pub power_margin_w: i64,
    pub outposts: Vec<OutpostValueDto>,
    pub top_sales: Vec<SaleValueDto>,
    pub facilities: Vec<FacilityUsageDto>,
    pub external_supply_slack: Vec<ExternalSupplySlackDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutpostValueDto {
    pub key: String,
    pub name: String,
    pub value_per_min: f64,
    pub cap_per_min: f64,
    pub ratio: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleValueDto {
    pub outpost_key: String,
    pub outpost_name: String,
    pub item_key: String,
    pub item_name: String,
    pub value_per_min: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacilityUsageDto {
    pub key: String,
    pub name: String,
    pub machines: u32,
    pub power_w: u32,
    pub total_power_w: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSupplySlackDto {
    pub item_key: String,
    pub item_name: String,
    pub supply_per_min: f64,
    pub slack_per_min: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogisticsGraphDto {
    pub items: Vec<LogisticsItemSummaryDto>,
    pub nodes: Vec<LogisticsNodeDto>,
    pub edges: Vec<LogisticsEdgeDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogisticsItemSummaryDto {
    pub item_key: String,
    pub item_name: String,
    pub edge_count: usize,
    pub node_count: usize,
    pub total_flow_per_min: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogisticsNodeDto {
    pub id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogisticsEdgeDto {
    pub id: String,
    pub item_key: String,
    pub item_name: String,
    pub source: String,
    pub target: String,
    pub flow_per_min: f64,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum ApiEnvelope<T> {
    Ok { data: T },
    Err { error: ApiErrorDto },
}

#[derive(Debug, Serialize)]
pub(crate) struct ApiErrorDto {
    message: String,
}

pub(crate) fn envelope_json<T: Serialize>(result: Result<T>) -> String {
    let envelope = match result {
        Ok(data) => ApiEnvelope::Ok { data },
        Err(err) => ApiEnvelope::Err {
            error: ApiErrorDto {
                message: err.to_string(),
            },
        },
    };

    match serde_json::to_string(&envelope) {
        Ok(json) => json,
        Err(_) => "{\"status\":\"err\",\"error\":{\"message\":\"failed to serialize response\"}}"
            .to_string(),
    }
}
