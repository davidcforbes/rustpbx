//! Generic dashboard data endpoint for report pages.
//!
//! Returns KPI cards, optional chart data, and table rows for any
//! report type. Real aggregation queries will be added per report
//! type over time; initially returns empty data.

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::iiz::api::IizState;

/// A single KPI card shown at the top of a dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportKpi {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// "up", "down", or "neutral"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend: Option<String>,
    /// CSS color class hint: "green", "red", "orange", "cyan", "dark"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// A single bar/point in a chart series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    pub label: String,
    pub values: Vec<f64>,
}

/// Legend entry for a chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartLegend {
    pub label: String,
    pub color: String,
}

/// Chart data for a dashboard report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportChart {
    pub title: String,
    /// "bar-vertical", "bar-horizontal", "stacked", "grid"
    pub chart_type: String,
    pub legend: Vec<ChartLegend>,
    pub points: Vec<ChartPoint>,
}

/// A single row in the dashboard data table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<String>,
}

/// Complete dashboard data for a report page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub report_type: String,
    pub kpis: Vec<ReportKpi>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart: Option<ReportChart>,
    pub table_headers: Vec<String>,
    pub table_rows: Vec<TableRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_footer: Option<Vec<String>>,
    /// Per-column alignment: "left" or "right". If absent, all left-aligned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_alignments: Option<Vec<String>>,
}

/// GET /reports/dashboard/:report_type
///
/// Returns dashboard data for the given report type. Currently returns
/// empty placeholder data — real aggregation queries added per type.
pub async fn get_dashboard(
    State(_state): State<IizState>,
    Path(report_type): Path<String>,
) -> Json<DashboardData> {
    Json(DashboardData {
        report_type,
        kpis: vec![],
        chart: None,
        table_headers: vec![],
        table_rows: vec![],
        table_footer: None,
        column_alignments: None,
    })
}
