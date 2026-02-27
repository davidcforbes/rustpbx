use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

use crate::api::api_get;
use crate::api::types::{
    AppointmentItem, CustomReportItem, DashboardData, ListResponse,
    NotificationRuleItem, ReportChart, ReportKpi, TagItem, TrackingSourceItem,
};

// ---------------------------------------------------------------------------
// Reports side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn ReportsSideNav() -> impl IntoView {
    let location = use_location();
    let active = |href: &'static str| {
        move || {
            if location.pathname.get() == href { "side-nav-item active" } else { "side-nav-item" }
        }
    };

    view! {
        <div class="px-4 pt-4 pb-2">
            <div class="flex items-center gap-2 text-iiz-cyan">
                <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsBarChartFill /></span>
                <span class="text-lg font-light">"Reports"</span>
            </div>
        </div>

        <nav class="px-2 pb-4 overflow-y-auto">
            // Analytics group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsGraphUp /></span>
                    "Analytics"
                </h3>
                <a href="/reports/activity" class=active("/reports/activity")>"Activity Reports"</a>
                <a href="/reports/roi" class=active("/reports/roi")>"ROI Reports"</a>
                <a href="/reports/accuracy" class=active("/reports/accuracy")>"Accuracy Reports"</a>
                <a href="/reports/map" class=active("/reports/map")>"Activity Map"</a>
                <a href="/reports/overview" class=active("/reports/overview")>"Overview"</a>
                <a href="/reports/todays-missed" class=active("/reports/todays-missed")>"Today's Missed Calls"</a>
                <a href="/reports/positive-daily" class=active("/reports/positive-daily")>"Positive Daily Reports"</a>
                <a href="/reports/google-ca" class=active("/reports/google-ca")>"Google CA Report"</a>
                <a href="/reports/saturday-calls" class=active("/reports/saturday-calls")>"saturday calls"</a>
                <a href="/reports/daily-calls" class=active("/reports/daily-calls")>"Daily Calls"</a>
                <a href="/reports/weekly-missed" class=active("/reports/weekly-missed")>"Weekly Missed Calls"</a>
                <a href="/reports/priming" class=active("/reports/priming")>"Priming Calls"</a>
                <a href="/reports/missed" class=active("/reports/missed")>"Missed Calls"</a>
                <a href="/reports/missed-daily-1st" class=active("/reports/missed-daily-1st")>"Missed Calls Daily - 1st"</a>
                <a href="/reports/cs-daily-missed" class=active("/reports/cs-daily-missed")>"CS Daily Missed Calls"</a>
                <a href="/reports/cs-daily-missed-2" class=active("/reports/cs-daily-missed-2")>"CS Daily Missed 2.0"</a>
                <a href="/reports/priming-missed" class=active("/reports/priming-missed")>"Priming Missed Calls"</a>
                <a href="/reports/daily-collection" class=active("/reports/daily-collection")>"Daily Collection Calls"</a>
                <a href="/reports/power-bi" class=active("/reports/power-bi")>"Power BI - Total Inbound"</a>
                <a href="/reports/realtime" class=active("/reports/realtime")>"real time"</a>
                <a href="/reports/appointments" class=active("/reports/appointments")>"Appointments"</a>
            </div>

            // Connect group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsPeopleFill /></span>
                    "Connect"
                </h3>
                <a href="/reports/realtime-agents" class=active("/reports/realtime-agents")>"Real-time Agents"</a>
                <a href="/reports/coaching" class=active("/reports/coaching")>"Coaching"</a>
                <a href="/reports/queue-report" class=active("/reports/queue-report")>"Queue Report"</a>
                <a href="/reports/agent-activity" class=active("/reports/agent-activity")>"Agent Activity"</a>
            </div>

            // Usage group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsSpeedometer /></span>
                    "Usage"
                </h3>
                <a href="/reports/agency-usage" class=active("/reports/agency-usage")>"Agency Usage"</a>
            </div>

            // Report Settings group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsGearFill /></span>
                    "Report Settings"
                </h3>
                <a href="/reports/custom-reports" class=active("/reports/custom-reports")>"Custom Reports"</a>
                <a href="/reports/notifications" class=active("/reports/notifications")>"Notifications"</a>
                <a href="/reports/scoring" class=active("/reports/scoring")>"Scoring"</a>
                <a href="/reports/tags" class=active("/reports/tags")>"Tags"</a>
            </div>
        </nav>
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Truncate an ISO-8601 datetime to just the date portion.
fn fmt_date(iso: &str) -> String {
    if iso.len() >= 10 { iso[..10].to_string() } else { iso.to_string() }
}

/// Loading spinner placeholder.
fn loading_view() -> impl IntoView {
    view! {
        <div class="flex-1 flex items-center justify-center p-8">
            <span class="loading loading-spinner loading-md text-iiz-cyan"></span>
            <span class="ml-2 text-gray-500">"Loading..."</span>
        </div>
    }
}

/// Error message display.
fn error_view(msg: String) -> impl IntoView {
    view! {
        <div class="flex-1 flex items-center justify-center p-8">
            <div class="text-red-500 text-sm">{msg}</div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Shared dashboard rendering components
// ---------------------------------------------------------------------------

/// Standard toolbar for dashboard report pages.
fn report_toolbar() -> impl IntoView {
    view! {
        <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
            <button class="btn btn-sm btn-ghost gap-1">
                <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsFunnel /></span>
                "Filter"
            </button>
            <div class="join">
                <input type="text" placeholder="Search..." class="input input-sm input-bordered join-item w-48" />
                <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGearFill /></span>
                </button>
                <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                </button>
            </div>
            <div class="flex-1"></div>
            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                "Info"
            </a>
            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsTelephone /></span>
                "Call Log"
            </a>
            <div class="dropdown dropdown-end">
                <button class="btn btn-sm btn-ghost gap-1">
                    "Export"
                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronDown /></span>
                </button>
            </div>
            <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Schedules..."</button>
        </header>
    }
}

/// Render KPI cards from dashboard data. Adapts grid cols to count.
fn kpi_cards_view(kpis: Vec<ReportKpi>) -> impl IntoView {
    let cols = match kpis.len() {
        0 => return view! { <div></div> }.into_any(),
        1..=3 => "grid grid-cols-3 gap-4 p-4",
        _ => "grid grid-cols-4 gap-4 p-4",
    };
    view! {
        <div class=cols>
            {kpis.into_iter().map(|kpi| {
                let value_color = match kpi.color.as_deref() {
                    Some("green") => "text-2xl font-bold text-green-600",
                    Some("red") => "text-2xl font-bold text-red-500",
                    Some("orange") => "text-2xl font-bold text-orange-500",
                    Some("cyan") => "text-2xl font-bold text-iiz-cyan",
                    _ => "text-2xl font-bold text-iiz-dark",
                };
                let trend_color = match kpi.trend.as_deref() {
                    Some("up") => "text-xs text-green-500",
                    Some("down") => "text-xs text-red-500",
                    _ => "text-xs text-gray-400",
                };
                let subtitle = kpi.subtitle.unwrap_or_default();
                view! {
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">{kpi.label}</div>
                            <div class=value_color>{kpi.value}</div>
                            <div class=trend_color>{subtitle}</div>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }.into_any()
}

/// Render a vertical bar chart from chart data.
fn bar_chart_view(chart: ReportChart) -> impl IntoView {
    let max_val = chart.points.iter()
        .flat_map(|p| p.values.iter())
        .cloned()
        .fold(1.0_f64, f64::max);

    view! {
        <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
            <h3 class="text-sm font-semibold text-gray-700 mb-3">{chart.title}</h3>
            <div class="h-40 flex items-end justify-around gap-2 px-4">
                {chart.points.into_iter().map(|pt| {
                    let total: f64 = pt.values.iter().sum();
                    let h = format!("height: {}%;", (total / max_val * 100.0) as u32);
                    view! {
                        <div class="flex flex-col items-center flex-1">
                            <span class="text-xs text-gray-500 mb-1">{format!("{}", total as i64)}</span>
                            <div class="w-8 bg-iiz-cyan rounded-t" style=h></div>
                            <span class="text-xs text-gray-400 mt-1">{pt.label}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            {if !chart.legend.is_empty() {
                Some(view! {
                    <div class="flex gap-4 mt-3 text-xs text-gray-400 justify-center">
                        {chart.legend.into_iter().map(|l| {
                            let dot_style = format!("background-color: {};", l.color);
                            view! {
                                <span class="flex items-center gap-1">
                                    <span class="w-3 h-3 rounded inline-block" style=dot_style></span>
                                    {l.label}
                                </span>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
}

/// Render a data table from dashboard data.
fn dashboard_table_view(data: &DashboardData) -> impl IntoView {
    if data.table_headers.is_empty() {
        return view! { <div></div> }.into_any();
    }
    let headers = data.table_headers.clone();
    let alignments = data.column_alignments.clone().unwrap_or_default();
    let rows = data.table_rows.clone();
    let footer = data.table_footer.clone();

    view! {
        <div class="mx-4 mb-4 card bg-white border border-gray-200">
            <div class="overflow-x-auto">
                <table class="table table-sm w-full">
                    <thead>
                        <tr class="border-b border-gray-200">
                            {headers.iter().enumerate().map(|(i, h)| {
                                let cls = if alignments.get(i).map(|a| a == "right").unwrap_or(false) {
                                    "text-xs font-medium text-gray-500 uppercase text-right"
                                } else {
                                    "text-xs font-medium text-gray-500 uppercase"
                                };
                                let h = h.clone();
                                view! { <th class=cls>{h}</th> }
                            }).collect::<Vec<_>>()}
                        </tr>
                    </thead>
                    <tbody>
                        {rows.into_iter().map(|row| {
                            let aligns = alignments.clone();
                            view! {
                                <tr class="border-b border-gray-100 hover:bg-gray-50">
                                    {row.cells.into_iter().enumerate().map(|(i, cell)| {
                                        let cls = if i == 0 {
                                            "text-sm font-medium".to_string()
                                        } else if aligns.get(i).map(|a| a == "right").unwrap_or(false) {
                                            "text-sm text-gray-600 text-right".to_string()
                                        } else {
                                            "text-sm text-gray-600".to_string()
                                        };
                                        view! { <td class=cls>{cell}</td> }
                                    }).collect::<Vec<_>>()}
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </tbody>
                    {footer.map(|f| {
                        let aligns2 = alignments.clone();
                        view! {
                            <tfoot>
                                <tr class="border-t border-gray-300 font-semibold">
                                    {f.into_iter().enumerate().map(|(i, cell)| {
                                        let cls = if aligns2.get(i).map(|a| a == "right").unwrap_or(false) {
                                            "text-sm text-right"
                                        } else {
                                            "text-sm"
                                        };
                                        view! { <td class=cls>{cell}</td> }
                                    }).collect::<Vec<_>>()}
                                </tr>
                            </tfoot>
                        }
                    })}
                </table>
            </div>
        </div>
    }.into_any()
}

/// Empty state when dashboard has no data.
fn empty_dashboard_view() -> impl IntoView {
    view! {
        <div class="flex-1 flex items-center justify-center p-16">
            <div class="text-center">
                <span class="w-12 h-12 inline-flex text-gray-300 mb-3"><Icon icon=icondata::BsBarChartFill /></span>
                <p class="text-gray-500 text-sm">"No data available for this report."</p>
                <p class="text-gray-400 text-xs mt-1">"Adjust filters or date range to see results."</p>
            </div>
        </div>
    }
}

/// Generic dashboard page that fetches and renders DashboardData.
#[component]
fn DashboardPageView(report_type: &'static str) -> impl IntoView {
    let url = format!("/reports/dashboard/{}", report_type);
    let data = LocalResource::new(move || {
        let url = url.clone();
        async move { api_get::<DashboardData>(&url).await }
    });

    view! {
        <div class="flex flex-col h-full">
            {report_toolbar()}
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(d)) => {
                    if d.kpis.is_empty() && d.table_rows.is_empty() {
                        view! {
                            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                                {empty_dashboard_view()}
                            </div>
                        }.into_any()
                    } else {
                        let chart = d.chart.clone();
                        view! {
                            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                                {kpi_cards_view(d.kpis.clone())}
                                {chart.map(|c| bar_chart_view(c))}
                                {dashboard_table_view(&d)}
                            </div>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Activity Report page (main analytics dashboard)
// ---------------------------------------------------------------------------

#[component]
pub fn ActivityReportPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<TrackingSourceItem>>("/numbers/sources?page=1&per_page=50").await
    });

    view! {
        <div class="flex flex-col h-full">
            // Top toolbar
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <button class="btn btn-sm btn-ghost gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsFunnel /></span>
                    "Filter"
                </button>
                <div class="join">
                    <input type="text" placeholder="Search..." class="input input-sm input-bordered join-item w-48" />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGearFill /></span>
                    </button>
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsTelephone /></span>
                    "Call Log"
                </a>
                <div class="dropdown dropdown-end">
                    <button class="btn btn-sm btn-ghost gap-1">
                        "Export"
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronDown /></span>
                    </button>
                </div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Schedules..."</button>
            </header>

            // View selector row
            <div class="h-10 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <span class="text-xs text-gray-500">"View by"</span>
                <select class="select select-xs select-bordered">
                    <option selected>"Tracking Source"</option>
                    <option>"Agent"</option>
                    <option>"Campaign"</option>
                    <option>"Number"</option>
                </select>
                <button class="btn btn-xs btn-ghost text-iiz-cyan">"+"</button>
                <div class="flex-1"></div>
                <div class="join">
                    <button class="btn btn-xs join-item">"Hour"</button>
                    <button class="btn btn-xs join-item bg-iiz-cyan text-white border-none">"Day"</button>
                    <button class="btn btn-xs join-item">"Week"</button>
                    <button class="btn btn-xs join-item">"Month"</button>
                    <button class="btn btn-xs join-item">"Quarter"</button>
                    <button class="btn btn-xs join-item">"Year"</button>
                </div>
            </div>

            <div class="flex-1 overflow-y-auto">
                // Chart placeholder (static — needs aggregation endpoint)
                <div class="bg-white border-b border-gray-200 p-4">
                    <div class="h-48 bg-gray-50 rounded-lg border border-gray-200 flex items-end justify-center gap-1 px-4 pb-4">
                        {[35, 42, 38, 45, 50, 47, 12, 8, 44, 46, 43, 41, 48, 11, 7, 45, 47, 44, 42, 49, 10, 6, 43, 45, 41, 39, 46, 9, 5].into_iter().enumerate().map(|(i, h)| {
                            let height = format!("height: {}%; min-height: 4px;", h * 2);
                            let color = if h < 15 { "bg-gray-300" } else { "bg-green-400" };
                            let class = format!("w-2 rounded-t {}", color);
                            let _ = i;
                            view! {
                                <div class=class style=height></div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div class="flex justify-between mt-2 text-xs text-gray-400">
                        <span>"Jan 26"</span>
                        <span>"Feb 02"</span>
                        <span>"Feb 09"</span>
                        <span>"Feb 16"</span>
                        <span>"Feb 23"</span>
                    </div>
                </div>

                // Data table — API-driven source rows
                <div class="overflow-x-auto">
                    // Column headers
                    <div class="grid grid-cols-[180px_80px_80px_80px_80px_80px_80px] gap-1 px-4 py-2 bg-gray-50 border-b border-gray-200 min-w-max">
                        <div class="col-header">"Source"</div>
                        <div class="col-header text-right">"Calls"</div>
                        <div class="col-header text-right">"Status"</div>
                        <div class="col-header text-right">"Position"</div>
                        <div class="col-header text-right">"Numbers"</div>
                        <div class="col-header text-right">"Type"</div>
                        <div class="col-header text-right">"Created"</div>
                    </div>

                    // Source rows from API
                    {move || match data.get() {
                        None => loading_view().into_any(),
                        Some(Err(e)) => error_view(e).into_any(),
                        Some(Ok(resp)) => {
                            let items = resp.items.clone();
                            view! {
                                <div>
                                    {items.into_iter().map(|s| {
                                        let status_class = if s.status == "active" {
                                            "badge badge-sm bg-green-100 text-green-700 border-green-200"
                                        } else {
                                            "badge badge-sm bg-gray-100 text-gray-500 border-gray-200"
                                        };
                                        view! {
                                            <div class="activity-row grid grid-cols-[180px_80px_80px_80px_80px_80px_80px] gap-1 px-4 py-2 items-center min-w-max">
                                                <div class="flex items-center gap-2">
                                                    <span class="text-sm truncate font-medium">{s.name.clone()}</span>
                                                </div>
                                                <div class="text-right text-sm">{s.call_count.to_string()}</div>
                                                <div class="text-right"><span class=status_class>{s.status.clone()}</span></div>
                                                <div class="text-right text-sm">{s.position.to_string()}</div>
                                                <div class="text-right text-sm">{s.number_count.to_string()}</div>
                                                <div class="text-right text-sm text-gray-500">{s.source_type.clone().unwrap_or_else(|| "\u{2014}".to_string())}</div>
                                                <div class="text-right text-sm text-gray-500">{fmt_date(&s.created_at)}</div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 25 Dashboard report pages — each delegates to DashboardPageView
// ---------------------------------------------------------------------------

#[component]
pub fn ROIReportPage() -> impl IntoView {
    view! { <DashboardPageView report_type="roi" /> }
}

#[component]
pub fn AccuracyReportPage() -> impl IntoView {
    view! { <DashboardPageView report_type="accuracy" /> }
}

#[component]
pub fn ActivityMapPage() -> impl IntoView {
    view! { <DashboardPageView report_type="activity-map" /> }
}

#[component]
pub fn OverviewPage() -> impl IntoView {
    view! { <DashboardPageView report_type="overview" /> }
}

#[component]
pub fn TodaysMissedPage() -> impl IntoView {
    view! { <DashboardPageView report_type="todays-missed" /> }
}

#[component]
pub fn PositiveDailyPage() -> impl IntoView {
    view! { <DashboardPageView report_type="positive-daily" /> }
}

#[component]
pub fn GoogleCAPage() -> impl IntoView {
    view! { <DashboardPageView report_type="google-ca" /> }
}

#[component]
pub fn SaturdayCallsPage() -> impl IntoView {
    view! { <DashboardPageView report_type="saturday-calls" /> }
}

#[component]
pub fn DailyCallsPage() -> impl IntoView {
    view! { <DashboardPageView report_type="daily-calls" /> }
}

#[component]
pub fn WeeklyMissedPage() -> impl IntoView {
    view! { <DashboardPageView report_type="weekly-missed" /> }
}

#[component]
pub fn PrimingCallsPage() -> impl IntoView {
    view! { <DashboardPageView report_type="priming-calls" /> }
}

#[component]
pub fn MissedCallsPage() -> impl IntoView {
    view! { <DashboardPageView report_type="missed-calls" /> }
}

#[component]
pub fn MissedDaily1stPage() -> impl IntoView {
    view! { <DashboardPageView report_type="missed-daily-1st" /> }
}

#[component]
pub fn CSDailyMissedPage() -> impl IntoView {
    view! { <DashboardPageView report_type="cs-daily-missed" /> }
}

#[component]
pub fn CSDailyMissed2Page() -> impl IntoView {
    view! { <DashboardPageView report_type="cs-daily-missed-2" /> }
}

#[component]
pub fn PrimingMissedPage() -> impl IntoView {
    view! { <DashboardPageView report_type="priming-missed" /> }
}

#[component]
pub fn DailyCollectionPage() -> impl IntoView {
    view! { <DashboardPageView report_type="daily-collection" /> }
}

#[component]
pub fn PowerBIPage() -> impl IntoView {
    view! { <DashboardPageView report_type="power-bi" /> }
}

#[component]
pub fn RealTimePage() -> impl IntoView {
    view! { <DashboardPageView report_type="realtime" /> }
}

#[component]
pub fn RealTimeAgentsPage() -> impl IntoView {
    view! { <DashboardPageView report_type="realtime-agents" /> }
}

#[component]
pub fn CoachingPage() -> impl IntoView {
    view! { <DashboardPageView report_type="coaching" /> }
}

#[component]
pub fn QueueReportPage() -> impl IntoView {
    view! { <DashboardPageView report_type="queue-report" /> }
}

#[component]
pub fn AgentActivityPage() -> impl IntoView {
    view! { <DashboardPageView report_type="agent-activity" /> }
}

#[component]
pub fn AgencyUsagePage() -> impl IntoView {
    view! { <DashboardPageView report_type="agency-usage" /> }
}

#[component]
pub fn ScoringPage() -> impl IntoView {
    view! { <DashboardPageView report_type="scoring" /> }
}

// ---------------------------------------------------------------------------
// Appointments page (API-driven, kept as-is)
// ---------------------------------------------------------------------------

#[component]
pub fn AppointmentsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<AppointmentItem>>("/reports/appointments?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <button class="btn btn-sm btn-ghost gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsFunnel /></span>
                    "Filter"
                </button>
                <div class="join">
                    <input type="text" placeholder="Search..." class="input input-sm input-bordered join-item w-48" />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGearFill /></span>
                    </button>
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsTelephone /></span>
                    "Call Log"
                </a>
                <div class="dropdown dropdown-end">
                    <button class="btn btn-sm btn-ghost gap-1">
                        "Export"
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronDown /></span>
                    </button>
                </div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Schedules..."</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                // KPI cards (static — needs aggregation endpoint)
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Appointments Today"</div>
                            <div class="text-2xl font-bold text-green-600">"12"</div>
                            <div class="text-xs text-green-500">"+ 3 vs yesterday"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"This Week"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"67"</div>
                            <div class="text-xs text-green-500">"+ 8 vs last week"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Conversion Rate"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"8.4%"</div>
                            <div class="text-xs text-green-500">"+ 0.6% vs avg"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"No-Show Rate"</div>
                            <div class="text-2xl font-bold text-red-500">"14.2%"</div>
                            <div class="text-xs text-red-500">"Above 10% target"</div>
                        </div>
                    </div>
                </div>

                // Chart: Weekly trend bars (static — needs aggregation endpoint)
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Weekly Appointments Trend"</h3>
                    <div class="h-36 flex items-end justify-around gap-3 px-4">
                        {[
                            ("Wk 1", 52), ("Wk 2", 58), ("Wk 3", 61), ("Wk 4", 67),
                        ].into_iter().map(|(week, count)| {
                            let h = format!("height: {}%;", (count as f32 / 67.0 * 100.0) as u32);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-xs text-gray-500 mb-1">{count}</span>
                                    <div class="w-16 bg-green-400 rounded-t" style=h></div>
                                    <span class="text-xs text-gray-400 mt-1">{week}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table — API-driven
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Date/Time"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Caller"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Phone"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Type"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Notes"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || match data.get() {
                                    None => loading_view().into_any(),
                                    Some(Err(e)) => error_view(e).into_any(),
                                    Some(Ok(resp)) => {
                                        let items = resp.items.clone();
                                        view! {
                                            <>
                                                {items.into_iter().map(|a| {
                                                    let type_class = match a.appointment_type.as_str() {
                                                        "new" | "New" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                                        "follow_up" | "Follow-up" => "badge badge-sm bg-blue-100 text-blue-700 border-blue-200",
                                                        _ => "badge badge-sm bg-purple-100 text-purple-700 border-purple-200",
                                                    };
                                                    let status_class = match a.status.as_str() {
                                                        "confirmed" | "Confirmed" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                                        "completed" | "Completed" => "badge badge-sm bg-gray-100 text-gray-700 border-gray-200",
                                                        "no_show" | "No-Show" => "badge badge-sm bg-red-100 text-red-700 border-red-200",
                                                        _ => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                                    };
                                                    let sched = fmt_date(&a.scheduled_at);
                                                    let caller = a.caller_name.clone().unwrap_or_else(|| "\u{2014}".to_string());
                                                    let phone = a.caller_phone.clone().unwrap_or_else(|| "\u{2014}".to_string());
                                                    let notes = a.notes.clone().unwrap_or_else(|| "\u{2014}".to_string());
                                                    view! {
                                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                            <td class="text-sm font-medium">{sched}</td>
                                                            <td class="text-sm text-gray-700">{caller}</td>
                                                            <td class="text-sm text-iiz-cyan">{phone}</td>
                                                            <td><span class=type_class>{a.appointment_type.clone()}</span></td>
                                                            <td><span class=status_class>{a.status.clone()}</span></td>
                                                            <td class="text-sm text-gray-500 truncate max-w-[150px]">{notes}</td>
                                                        </tr>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </>
                                        }.into_any()
                                    }
                                }}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 4 Report Settings pages
// ---------------------------------------------------------------------------

#[component]
pub fn CustomReportsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<CustomReportItem>>("/reports/custom-reports?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Custom Reports"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Report"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Type"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Schedule"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Last Run"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Created"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || match data.get() {
                                        None => loading_view().into_any(),
                                        Some(Err(e)) => error_view(e).into_any(),
                                        Some(Ok(resp)) => {
                                            let items = resp.items.clone();
                                            view! {
                                                <>
                                                    {items.into_iter().map(|r| {
                                                        let rtype = r.report_type.clone().unwrap_or_else(|| "\u{2014}".to_string());
                                                        let sched = r.schedule.clone().unwrap_or_else(|| "\u{2014}".to_string());
                                                        let last_run = r.last_run_at.as_deref().map(|d| fmt_date(d)).unwrap_or_else(|| "Never".to_string());
                                                        let created = fmt_date(&r.created_at);
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{r.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{rtype}</td>
                                                                <td class="text-sm text-gray-600">{sched}</td>
                                                                <td class="text-sm text-gray-600">{last_run}</td>
                                                                <td class="text-sm text-gray-600">{created}</td>
                                                            </tr>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </>
                                            }.into_any()
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<NotificationRuleItem>>("/reports/notification-rules?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Notifications"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Notification"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Method"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Metric"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Triggered"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Active"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || match data.get() {
                                        None => loading_view().into_any(),
                                        Some(Err(e)) => error_view(e).into_any(),
                                        Some(Ok(resp)) => {
                                            let items = resp.items.clone();
                                            view! {
                                                <>
                                                    {items.into_iter().map(|n| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                                <td class="text-sm font-medium">{n.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{n.notification_method.clone()}</td>
                                                                <td class="text-sm text-gray-600">{format!("{} {} {}", &n.metric, &n.condition_operator, n.threshold_value)}</td>
                                                                <td class="text-sm text-gray-600">{n.trigger_count.to_string()}" times"</td>
                                                                <td>
                                                                    {if n.is_active {
                                                                        view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-500 border-gray-200">"Inactive"</span> }.into_any()
                                                                    }}
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </>
                                            }.into_any()
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn TagsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<TagItem>>("/tags?page=1&per_page=50").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Tags"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Tag"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-2xl mx-auto p-6">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            {move || match data.get() {
                                None => loading_view().into_any(),
                                Some(Err(e)) => error_view(e).into_any(),
                                Some(Ok(resp)) => {
                                    let items = resp.items.clone();
                                    view! {
                                        <div class="space-y-3">
                                            {items.into_iter().map(|tag| {
                                                let color = tag.color.as_deref().unwrap_or("bg-gray-500");
                                                let dot_class = format!("w-3 h-3 rounded-full {}", color);
                                                view! {
                                                    <div class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 cursor-pointer">
                                                        <div class="flex items-center gap-3">
                                                            <span class=dot_class></span>
                                                            <span class="text-sm font-medium text-gray-700">{tag.name.clone()}</span>
                                                        </div>
                                                        <div class="flex items-center gap-3">
                                                            <span class="text-xs text-gray-400">{tag.usage_count.to_string()}" uses"</span>
                                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                                <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                                            </button>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
