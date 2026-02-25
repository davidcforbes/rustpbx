use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

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
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct SourceRow {
    name: &'static str,
    badge_pct: &'static str,
    badge_color: &'static str,
    total: &'static str,
    total_pct: &'static str,
    period_unique: &'static str,
    period_unique_pct: &'static str,
    globally_unique: &'static str,
    globally_unique_pct: &'static str,
    ring_avg: &'static str,
    ring_total: &'static str,
    talk_avg: &'static str,
    talk_total: &'static str,
    total_time_avg: &'static str,
    total_time_total: &'static str,
}

// ---------------------------------------------------------------------------
// Mock data
// ---------------------------------------------------------------------------

fn mock_source_rows() -> Vec<SourceRow> {
    vec![
        SourceRow { name: "Google Organic", badge_pct: "73%", badge_color: "bg-green-500", total: "80,374", total_pct: "73.03%", period_unique: "19,988", period_unique_pct: "18.16%", globally_unique: "6,260", globally_unique_pct: "5.69%", ring_avg: "0:25", ring_total: "33,489.12", talk_avg: "2:22", talk_total: "190,487.08", total_time_avg: "2:55", total_time_total: "234,167.90" },
        SourceRow { name: "Customer Service Line", badge_pct: "20%", badge_color: "bg-orange-500", total: "22,270", total_pct: "20.24%", period_unique: "6,809", period_unique_pct: "6.19%", globally_unique: "425", globally_unique_pct: "0.39%", ring_avg: "0:30", ring_total: "11,135.00", talk_avg: "2:05", talk_total: "46,429.17", total_time_avg: "2:40", total_time_total: "59,386.67" },
        SourceRow { name: "Tiktok Organic", badge_pct: "2%", badge_color: "bg-red-500", total: "2,526", total_pct: "2.30%", period_unique: "1,746", period_unique_pct: "1.59%", globally_unique: "1,219", globally_unique_pct: "1.11%", ring_avg: "0:28", ring_total: "1,178.80", talk_avg: "2:10", talk_total: "5,473.00", total_time_avg: "2:45", total_time_total: "6,946.50" },
        SourceRow { name: "Facebook Paid", badge_pct: "3%", badge_color: "bg-red-500", total: "1,942", total_pct: "1.76%", period_unique: "1,178", period_unique_pct: "1.07%", globally_unique: "771", globally_unique_pct: "0.70%", ring_avg: "0:27", ring_total: "873.90", talk_avg: "2:15", talk_total: "4,369.50", total_time_avg: "2:48", total_time_total: "5,437.60" },
        SourceRow { name: "Facebook Organic", badge_pct: "1%", badge_color: "bg-green-500", total: "701", total_pct: "0.64%", period_unique: "556", period_unique_pct: "0.51%", globally_unique: "410", globally_unique_pct: "0.37%", ring_avg: "0:26", ring_total: "303.77", talk_avg: "2:20", talk_total: "1,635.67", total_time_avg: "2:50", total_time_total: "1,986.17" },
        SourceRow { name: "Book of Truths Trum", badge_pct: "1%", badge_color: "bg-green-500", total: "625", total_pct: "0.57%", period_unique: "326", period_unique_pct: "0.30%", globally_unique: "81", globally_unique_pct: "0.07%", ring_avg: "0:24", ring_total: "250.00", talk_avg: "2:30", talk_total: "1,562.50", total_time_avg: "3:00", total_time_total: "1,875.00" },
        SourceRow { name: "Radio La Ley", badge_pct: "\u{2014}", badge_color: "bg-gray-400", total: "425", total_pct: "0.39%", period_unique: "302", period_unique_pct: "0.27%", globally_unique: "136", globally_unique_pct: "0.12%", ring_avg: "0:29", ring_total: "205.42", talk_avg: "2:12", talk_total: "935.00", total_time_avg: "2:45", total_time_total: "1,168.75" },
        SourceRow { name: "Website", badge_pct: "0%", badge_color: "bg-red-500", total: "288", total_pct: "0.26%", period_unique: "194", period_unique_pct: "0.18%", globally_unique: "88", globally_unique_pct: "0.08%", ring_avg: "0:31", ring_total: "148.80", talk_avg: "1:58", talk_total: "566.40", total_time_avg: "2:35", total_time_total: "744.00" },
        SourceRow { name: "Instagram Organic", badge_pct: "0%", badge_color: "bg-red-500", total: "187", total_pct: "0.17%", period_unique: "127", period_unique_pct: "0.12%", globally_unique: "86", globally_unique_pct: "0.08%", ring_avg: "0:25", ring_total: "77.92", talk_avg: "2:18", talk_total: "430.10", total_time_avg: "2:48", total_time_total: "523.60" },
        SourceRow { name: "Yelp Organic", badge_pct: "\u{2014}", badge_color: "bg-gray-400", total: "173", total_pct: "0.16%", period_unique: "119", period_unique_pct: "0.11%", globally_unique: "59", globally_unique_pct: "0.05%", ring_avg: "0:27", ring_total: "77.85", talk_avg: "2:05", talk_total: "360.42", total_time_avg: "2:35", total_time_total: "447.58" },
        SourceRow { name: "Mass SMS", badge_pct: "0%", badge_color: "bg-green-500", total: "167", total_pct: "0.15%", period_unique: "129", period_unique_pct: "0.12%", globally_unique: "8", globally_unique_pct: "0.01%", ring_avg: "0:22", ring_total: "61.23", talk_avg: "2:45", talk_total: "459.25", total_time_avg: "3:10", total_time_total: "529.17" },
        SourceRow { name: "WhatsApp", badge_pct: "0%", badge_color: "bg-red-500", total: "110", total_pct: "0.10%", period_unique: "68", period_unique_pct: "0.06%", globally_unique: "38", globally_unique_pct: "0.03%", ring_avg: "0:30", ring_total: "55.00", talk_avg: "2:00", talk_total: "220.00", total_time_avg: "2:35", total_time_total: "284.17" },
        SourceRow { name: "Mystery Shopper", badge_pct: "0%", badge_color: "bg-green-500", total: "74", total_pct: "0.07%", period_unique: "35", period_unique_pct: "0.03%", globally_unique: "4", globally_unique_pct: "0.00%", ring_avg: "0:18", ring_total: "22.20", talk_avg: "3:05", talk_total: "228.17", total_time_avg: "3:25", total_time_total: "253.17" },
        SourceRow { name: "Google Ads", badge_pct: "0%", badge_color: "bg-red-500", total: "67", total_pct: "0.06%", period_unique: "45", period_unique_pct: "0.04%", globally_unique: "20", globally_unique_pct: "0.02%", ring_avg: "0:26", ring_total: "29.03", talk_avg: "2:10", talk_total: "145.17", total_time_avg: "2:40", total_time_total: "178.67" },
    ]
}

// ---------------------------------------------------------------------------
// Activity Report page (main analytics dashboard)
// ---------------------------------------------------------------------------

#[component]
pub fn ActivityReportPage() -> impl IntoView {
    let sources = mock_source_rows();

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
                // Chart placeholder
                <div class="bg-white border-b border-gray-200 p-4">
                    <div class="h-48 bg-gray-50 rounded-lg border border-gray-200 flex items-end justify-center gap-1 px-4 pb-4">
                        // Simplified bar chart representation
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

                // Data table
                <div class="overflow-x-auto">
                    // Column headers
                    <div class="grid grid-cols-[180px_80px_80px_80px_80px_80px_80px] gap-1 px-4 py-2 bg-gray-50 border-b border-gray-200 min-w-max">
                        <div class="col-header">"Source"</div>
                        <div class="col-header text-right">"Total"</div>
                        <div class="col-header text-right">"Period Unique"</div>
                        <div class="col-header text-right">"Globally Unique"</div>
                        <div class="col-header text-right">"Ring Time"</div>
                        <div class="col-header text-right">"Talk Time"</div>
                        <div class="col-header text-right">"Total Time"</div>
                    </div>

                    // Totals row
                    <div class="grid grid-cols-[180px_80px_80px_80px_80px_80px_80px] gap-1 px-4 py-2 bg-gray-50 border-b border-gray-300 font-semibold min-w-max">
                        <div class="text-sm">"Total"</div>
                        <div class="text-right">
                            <div class="text-sm">"110,050"</div>
                        </div>
                        <div class="text-right">
                            <div class="text-sm">"31,721"</div>
                        </div>
                        <div class="text-right">
                            <div class="text-sm">"9,671"</div>
                        </div>
                        <div class="text-right">
                            <div class="text-xs">"0:27 avg"</div>
                            <div class="text-xs text-gray-400 font-normal">"51,342.78"</div>
                        </div>
                        <div class="text-right">
                            <div class="text-xs">"2:18 avg"</div>
                            <div class="text-xs text-gray-400 font-normal">"254,905.82"</div>
                        </div>
                        <div class="text-right">
                            <div class="text-xs">"2:52 avg"</div>
                            <div class="text-xs text-gray-400 font-normal">"316,130.67"</div>
                        </div>
                    </div>

                    // Source rows
                    {sources.into_iter().map(|s| {
                        let badge_class = format!("inline-block w-10 text-center text-[10px] text-white rounded px-1 py-0.5 {}", s.badge_color);
                        view! {
                            <div class="activity-row grid grid-cols-[180px_80px_80px_80px_80px_80px_80px] gap-1 px-4 py-2 items-center min-w-max">
                                <div class="flex items-center gap-2">
                                    <span class=badge_class>{s.badge_pct}</span>
                                    <span class="text-sm truncate">{s.name}</span>
                                </div>
                                <div class="text-right">
                                    <div class="text-sm">{s.total}</div>
                                    <div class="text-xs text-gray-400">{s.total_pct}</div>
                                </div>
                                <div class="text-right">
                                    <div class="text-sm">{s.period_unique}</div>
                                    <div class="text-xs text-gray-400">{s.period_unique_pct}</div>
                                </div>
                                <div class="text-right">
                                    <div class="text-sm">{s.globally_unique}</div>
                                    <div class="text-xs text-gray-400">{s.globally_unique_pct}</div>
                                </div>
                                <div class="text-right">
                                    <div class="text-xs">{s.ring_avg}" avg"</div>
                                    <div class="text-xs text-gray-400">{s.ring_total}</div>
                                </div>
                                <div class="text-right">
                                    <div class="text-xs">{s.talk_avg}" avg"</div>
                                    <div class="text-xs text-gray-400">{s.talk_total}</div>
                                </div>
                                <div class="text-right">
                                    <div class="text-xs">{s.total_time_avg}" avg"</div>
                                    <div class="text-xs text-gray-400">{s.total_time_total}</div>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Analytics report pages (unique implementations)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 1. ROI Reports
// ---------------------------------------------------------------------------

#[component]
pub fn ROIReportPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            // Toolbar
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Total Revenue"</div>
                            <div class="text-2xl font-bold text-green-600">"$487,250"</div>
                            <div class="text-xs text-green-500">"+ 12.3% vs last period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Cost Per Call"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"$12.45"</div>
                            <div class="text-xs text-red-500">"+ 2.1% vs last period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"ROAS"</div>
                            <div class="text-2xl font-bold text-green-600">"4.2x"</div>
                            <div class="text-xs text-green-500">"+ 0.3x vs last period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Conversions"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"3,847"</div>
                            <div class="text-xs text-green-500">"+ 8.7% vs last period"</div>
                        </div>
                    </div>
                </div>

                // Chart: Horizontal stacked bars (revenue vs cost per source)
                <div class="bg-white border-b border-gray-200 mx-4 mb-4 rounded-lg border p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Revenue vs Cost by Source"</h3>
                    <div class="space-y-3">
                        {[
                            ("Google Organic", 72, 15),
                            ("Facebook Paid", 45, 30),
                            ("TikTok", 28, 22),
                            ("Direct", 18, 5),
                            ("Radio", 12, 8),
                        ].into_iter().map(|(name, rev_pct, cost_pct)| {
                            let rev_w = format!("width: {}%;", rev_pct);
                            let cost_w = format!("width: {}%;", cost_pct);
                            view! {
                                <div>
                                    <div class="flex justify-between text-xs text-gray-500 mb-1">
                                        <span>{name}</span>
                                        <span class="text-green-600">{format!("{}% rev", rev_pct)}</span>
                                    </div>
                                    <div class="flex h-4 bg-gray-100 rounded overflow-hidden gap-px">
                                        <div class="bg-green-400 rounded-l" style=rev_w></div>
                                        <div class="bg-red-300 rounded-r" style=cost_w></div>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                        <div class="flex gap-4 mt-2 text-xs text-gray-400">
                            <span class="flex items-center gap-1"><span class="w-3 h-3 bg-green-400 rounded inline-block"></span>"Revenue"</span>
                            <span class="flex items-center gap-1"><span class="w-3 h-3 bg-red-300 rounded inline-block"></span>"Cost"</span>
                        </div>
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Source"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Calls"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Revenue"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Cost"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"ROI%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Cost/Call"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Cost/Conv"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Conversions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Google Organic", "42,310", "$245,800", "$52,400", "369%", "$1.24", "$18.50", "2,832"),
                                    ("Facebook Paid", "18,450", "$112,500", "$45,200", "149%", "$2.45", "$62.40", "724"),
                                    ("TikTok", "5,230", "$48,200", "$22,100", "118%", "$4.23", "$142.00", "156"),
                                    ("Direct", "3,120", "$52,750", "$3,900", "1253%", "$1.25", "$52.75", "74"),
                                    ("Radio", "1,940", "$28,000", "$12,800", "119%", "$6.60", "$209.84", "61"),
                                ].into_iter().map(|(source, calls, rev, cost, roi, cpc, cpconv, conv)| {
                                    let roi_class = if roi.starts_with('-') { "text-sm text-red-600 text-right" } else { "text-sm text-green-600 text-right" };
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{source}</td>
                                            <td class="text-sm text-gray-600 text-right">{calls}</td>
                                            <td class="text-sm text-green-600 text-right">{rev}</td>
                                            <td class="text-sm text-red-500 text-right">{cost}</td>
                                            <td class=roi_class>{roi}</td>
                                            <td class="text-sm text-gray-600 text-right">{cpc}</td>
                                            <td class="text-sm text-gray-600 text-right">{cpconv}</td>
                                            <td class="text-sm text-gray-600 text-right">{conv}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                            <tfoot>
                                <tr class="border-t border-gray-300 font-semibold">
                                    <td class="text-sm">"Total"</td>
                                    <td class="text-sm text-right">"71,050"</td>
                                    <td class="text-sm text-green-600 text-right">"$487,250"</td>
                                    <td class="text-sm text-red-500 text-right">"$136,400"</td>
                                    <td class="text-sm text-green-600 text-right">"257%"</td>
                                    <td class="text-sm text-right">"$1.92"</td>
                                    <td class="text-sm text-right">"$35.46"</td>
                                    <td class="text-sm text-right">"3,847"</td>
                                </tr>
                            </tfoot>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 2. Accuracy Reports
// ---------------------------------------------------------------------------

#[component]
pub fn AccuracyReportPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            // Toolbar
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Attribution Accuracy"</div>
                            <div class="text-2xl font-bold text-green-600">"94.2%"</div>
                            <div class="text-xs text-green-500">"+ 1.8% vs last period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Misattributed"</div>
                            <div class="text-2xl font-bold text-red-500">"5.8%"</div>
                            <div class="text-xs text-green-500">"- 1.8% vs last period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Verified Sources"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"12/14"</div>
                            <div class="text-xs text-gray-400">"2 pending verification"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Data Quality Score"</div>
                            <div class="text-2xl font-bold text-green-600">"A"</div>
                            <div class="text-xs text-green-500">"Top tier quality"</div>
                        </div>
                    </div>
                </div>

                // Chart: Donut/ring style showing accurate vs misattributed
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Attribution Distribution"</h3>
                    <div class="flex items-center justify-center gap-8">
                        // CSS ring chart
                        <div class="relative w-40 h-40">
                            <div class="w-40 h-40 rounded-full" style="background: conic-gradient(#22c55e 0% 94.2%, #ef4444 94.2% 100%);"></div>
                            <div class="absolute inset-4 bg-white rounded-full flex items-center justify-center flex-col">
                                <span class="text-2xl font-bold text-green-600">"94.2%"</span>
                                <span class="text-xs text-gray-400">"Accurate"</span>
                            </div>
                        </div>
                        <div class="space-y-2">
                            <div class="flex items-center gap-2">
                                <span class="w-3 h-3 bg-green-500 rounded inline-block"></span>
                                <span class="text-sm text-gray-600">"Correctly Attributed: "</span>
                                <span class="text-sm font-semibold">"94.2%"</span>
                            </div>
                            <div class="flex items-center gap-2">
                                <span class="w-3 h-3 bg-red-500 rounded inline-block"></span>
                                <span class="text-sm text-gray-600">"Misattributed: "</span>
                                <span class="text-sm font-semibold">"5.8%"</span>
                            </div>
                        </div>
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Source"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Total Calls"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Attributed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Misattributed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Accuracy%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Last Verified"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Google Organic", "42,310", "40,588", "1,722", "95.9%", "Feb 24, 2026"),
                                    ("Facebook Paid", "18,450", "17,103", "1,347", "92.7%", "Feb 23, 2026"),
                                    ("TikTok", "5,230", "4,968", "262", "95.0%", "Feb 24, 2026"),
                                    ("Direct", "3,120", "2,933", "187", "94.0%", "Feb 22, 2026"),
                                    ("Radio", "1,940", "1,804", "136", "93.0%", "Feb 20, 2026"),
                                    ("Instagram", "1,200", "1,134", "66", "94.5%", "Feb 24, 2026"),
                                ].into_iter().map(|(source, total, attr, misattr, acc, verified)| {
                                    let acc_class = if acc.starts_with("9") { "text-sm text-green-600 text-right" } else { "text-sm text-orange-500 text-right" };
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{source}</td>
                                            <td class="text-sm text-gray-600 text-right">{total}</td>
                                            <td class="text-sm text-green-600 text-right">{attr}</td>
                                            <td class="text-sm text-red-500 text-right">{misattr}</td>
                                            <td class=acc_class>{acc}</td>
                                            <td class="text-sm text-gray-500">{verified}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

struct MapRegion {
    name: &'static str,
    calls: &'static str,
    pct: f32,
    answered: &'static str,
    missed: &'static str,
    avg_duration: &'static str,
    top_source: &'static str,
    color: &'static str,
}

fn map_region_data() -> Vec<MapRegion> {
    vec![
        MapRegion { name: "North Carolina", calls: "42,156", pct: 38.3, answered: "35,812", missed: "6,344", avg_duration: "2:45", top_source: "Google", color: "#00bcd4" },
        MapRegion { name: "Virginia", calls: "18,234", pct: 16.6, answered: "15,499", missed: "2,735", avg_duration: "2:22", top_source: "Direct", color: "#26c6da" },
        MapRegion { name: "South Carolina", calls: "12,890", pct: 11.7, answered: "10,698", missed: "2,192", avg_duration: "2:18", top_source: "Facebook", color: "#4dd0e1" },
        MapRegion { name: "Georgia", calls: "9,456", pct: 8.6, answered: "7,944", missed: "1,512", avg_duration: "2:05", top_source: "Google", color: "#80deea" },
        MapRegion { name: "Florida", calls: "8,123", pct: 7.4, answered: "6,742", missed: "1,381", avg_duration: "1:58", top_source: "Referral", color: "#b2ebf2" },
        MapRegion { name: "New York", calls: "6,890", pct: 6.3, answered: "5,650", missed: "1,240", avg_duration: "2:12", top_source: "Google", color: "#b2ebf2" },
        MapRegion { name: "California", calls: "5,234", pct: 4.8, answered: "4,292", missed: "942", avg_duration: "2:30", top_source: "TikTok", color: "#e0f7fa" },
        MapRegion { name: "Other States", calls: "7,067", pct: 6.4, answered: "5,724", missed: "1,343", avg_duration: "2:08", top_source: "Various", color: "#e0f7fa" },
    ]
}

#[component]
pub fn ActivityMapPage() -> impl IntoView {
    let regions = map_region_data();
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
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                <div class="flex-1"></div>
                <select class="select select-sm select-bordered">
                    <option selected>"By State"</option>
                    <option>"By City"</option>
                    <option>"By ZIP Code"</option>
                    <option>"By Area Code"</option>
                </select>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
                <div class="dropdown dropdown-end">
                    <button class="btn btn-sm btn-ghost gap-1">
                        "Export"
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronDown /></span>
                    </button>
                </div>
            </header>

            // Title row
            <div class="bg-white border-b border-gray-200 px-4 py-3 flex-shrink-0">
                <h2 class="text-lg font-semibold text-iiz-dark">"Activity Map"</h2>
                <p class="text-xs text-gray-500">"Geographic visualization of call activity by region"</p>
            </div>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg p-4">
                // KPI cards
                <div class="grid grid-cols-4 gap-3 mb-4">
                    <div class="bg-white rounded-lg border border-gray-200 p-4">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Total Regions"</div>
                        <div class="text-2xl font-bold text-iiz-dark mt-1">"48"</div>
                        <div class="text-xs text-gray-400 mt-1">"Active states"</div>
                    </div>
                    <div class="bg-white rounded-lg border border-gray-200 p-4">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Top Region"</div>
                        <div class="text-2xl font-bold text-iiz-cyan mt-1">"NC"</div>
                        <div class="text-xs text-green-600 mt-1">"38.3% of calls"</div>
                    </div>
                    <div class="bg-white rounded-lg border border-gray-200 p-4">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Concentration"</div>
                        <div class="text-2xl font-bold text-iiz-dark mt-1">"75.2%"</div>
                        <div class="text-xs text-gray-400 mt-1">"Top 4 states"</div>
                    </div>
                    <div class="bg-white rounded-lg border border-gray-200 p-4">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Avg Distance"</div>
                        <div class="text-2xl font-bold text-iiz-dark mt-1">"245 mi"</div>
                        <div class="text-xs text-gray-400 mt-1">"From office"</div>
                    </div>
                </div>

                // Map visualization (CSS-based US region heatmap)
                <div class="bg-white rounded-lg border border-gray-200 p-4 mb-4">
                    <h3 class="text-sm font-semibold text-iiz-dark mb-3">"Call Distribution Heatmap"</h3>
                    <div class="flex gap-6">
                        // CSS heatmap grid representing regions
                        <div class="flex-1">
                            <div class="grid grid-cols-8 gap-1" style="min-height: 200px;">
                                {regions.iter().map(|r| {
                                    let height = format!("{}%", (r.pct * 2.5).min(100.0));
                                    let bg = r.color;
                                    view! {
                                        <div class="flex flex-col items-center justify-end">
                                            <div
                                                class="w-full rounded-t-sm transition-all relative group cursor-pointer"
                                                style=format!("height: {}; background-color: {}; min-height: 20px;", height, bg)
                                            >
                                                <div class="absolute -top-6 left-1/2 -translate-x-1/2 text-xs font-semibold text-gray-700">
                                                    {r.calls}
                                                </div>
                                            </div>
                                            <div class="text-xs text-gray-600 mt-1 font-medium truncate w-full text-center">{r.name}</div>
                                            <div class="text-xs text-gray-400">{format!("{}%", r.pct)}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                        // Legend
                        <div class="w-48 flex-shrink-0">
                            <h4 class="text-xs font-semibold text-gray-500 uppercase mb-2">"Heat Scale"</h4>
                            <div class="space-y-1">
                                <div class="flex items-center gap-2">
                                    <span class="w-4 h-4 rounded" style="background-color: #00bcd4;"></span>
                                    <span class="text-xs text-gray-600">"High (>25%)"</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <span class="w-4 h-4 rounded" style="background-color: #4dd0e1;"></span>
                                    <span class="text-xs text-gray-600">"Medium (10-25%)"</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <span class="w-4 h-4 rounded" style="background-color: #80deea;"></span>
                                    <span class="text-xs text-gray-600">"Low (5-10%)"</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <span class="w-4 h-4 rounded" style="background-color: #e0f7fa;"></span>
                                    <span class="text-xs text-gray-600">"Minimal (<5%)"</span>
                                </div>
                            </div>
                            <div class="mt-4 p-2 bg-gray-50 rounded border border-gray-100">
                                <div class="text-xs text-gray-500 mb-1">"Total Calls"</div>
                                <div class="text-lg font-bold text-iiz-dark">"110,050"</div>
                                <div class="text-xs text-gray-400">"Across 48 states"</div>
                            </div>
                        </div>
                    </div>
                </div>

                // Region detail table
                <div class="bg-white rounded-lg border border-gray-200">
                    <table class="table table-sm w-full">
                        <thead>
                            <tr class="border-b border-gray-200">
                                <th class="text-xs text-gray-500 font-semibold uppercase">"Region"</th>
                                <th class="text-xs text-gray-500 font-semibold uppercase text-right">"Calls"</th>
                                <th class="text-xs text-gray-500 font-semibold uppercase text-right">"% of Total"</th>
                                <th class="text-xs text-gray-500 font-semibold uppercase text-right">"Answered"</th>
                                <th class="text-xs text-gray-500 font-semibold uppercase text-right">"Missed"</th>
                                <th class="text-xs text-gray-500 font-semibold uppercase text-right">"Avg Duration"</th>
                                <th class="text-xs text-gray-500 font-semibold uppercase">"Top Source"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {regions.iter().map(|r| {
                                let bar_width = format!("{}%", r.pct * 2.5);
                                view! {
                                    <tr class="border-b border-gray-100 hover:bg-gray-50">
                                        <td class="text-sm font-medium text-iiz-dark">
                                            <div class="flex items-center gap-2">
                                                <span class="w-3 h-3 rounded" style=format!("background-color: {};", r.color)></span>
                                                {r.name}
                                            </div>
                                        </td>
                                        <td class="text-sm text-right">{r.calls}</td>
                                        <td class="text-sm text-right">
                                            <div class="flex items-center justify-end gap-2">
                                                <div class="w-16 bg-gray-100 rounded-full h-1.5">
                                                    <div class="h-1.5 rounded-full" style=format!("width: {}; background-color: {};", bar_width, r.color)></div>
                                                </div>
                                                {format!("{}%", r.pct)}
                                            </div>
                                        </td>
                                        <td class="text-sm text-right text-green-600">{r.answered}</td>
                                        <td class="text-sm text-right text-red-500">{r.missed}</td>
                                        <td class="text-sm text-right">{r.avg_duration}</td>
                                        <td class="text-sm">{r.top_source}</td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 3. Overview
// ---------------------------------------------------------------------------

#[component]
pub fn OverviewPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Total Calls"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"110,050"</div>
                            <div class="text-xs text-gray-400">"This period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Answered"</div>
                            <div class="text-2xl font-bold text-green-600">"89,241"</div>
                            <div class="text-xs text-green-500">"81.1% answer rate"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Missed"</div>
                            <div class="text-2xl font-bold text-red-500">"15,407"</div>
                            <div class="text-xs text-red-500">"14.0% miss rate"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Avg Duration"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"2:18"</div>
                            <div class="text-xs text-gray-400">"Minutes per call"</div>
                        </div>
                    </div>
                </div>

                // Chart: 7-day stacked bars answered/missed
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Weekly Call Distribution"</h3>
                    <div class="h-48 flex items-end justify-around gap-2 px-4">
                        {[
                            ("Mon", 82, 12),
                            ("Tue", 90, 14),
                            ("Wed", 85, 11),
                            ("Thu", 88, 13),
                            ("Fri", 78, 15),
                            ("Sat", 42, 22),
                            ("Sun", 25, 18),
                        ].into_iter().map(|(day, answered_pct, missed_pct)| {
                            let ans_h = format!("height: {}px;", answered_pct * 2);
                            let miss_h = format!("height: {}px;", missed_pct * 2);
                            view! {
                                <div class="flex flex-col items-center gap-0.5 flex-1">
                                    <div class="w-full flex flex-col items-center gap-0.5">
                                        <div class="w-6 bg-red-300 rounded-t" style=miss_h></div>
                                        <div class="w-6 bg-green-400 rounded-b" style=ans_h></div>
                                    </div>
                                    <span class="text-xs text-gray-500 mt-1">{day}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div class="flex gap-4 mt-3 text-xs text-gray-400 justify-center">
                        <span class="flex items-center gap-1"><span class="w-3 h-3 bg-green-400 rounded inline-block"></span>"Answered"</span>
                        <span class="flex items-center gap-1"><span class="w-3 h-3 bg-red-300 rounded inline-block"></span>"Missed"</span>
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Day"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Total"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Answered"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Voicemail"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Ring"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Talk"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Conversion%"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Monday", "18,420", "15,312", "2,108", "1,002", "0:22", "2:35", "8.2%"),
                                    ("Tuesday", "19,847", "16,470", "2,377", "1,142", "0:24", "2:28", "9.1%"),
                                    ("Wednesday", "18,105", "15,248", "1,857", "923", "0:21", "2:42", "8.7%"),
                                    ("Thursday", "19,230", "16,013", "2,217", "1,078", "0:23", "2:31", "8.9%"),
                                    ("Friday", "17,560", "14,328", "2,232", "1,247", "0:26", "2:15", "7.8%"),
                                    ("Saturday", "10,240", "7,168", "2,072", "1,587", "0:34", "1:48", "5.2%"),
                                    ("Sunday", "6,648", "4,702", "1,946", "1,423", "0:38", "1:32", "3.8%"),
                                ].into_iter().map(|(day, total, ans, miss, vm, ring, talk, conv)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{day}</td>
                                            <td class="text-sm text-gray-600 text-right">{total}</td>
                                            <td class="text-sm text-green-600 text-right">{ans}</td>
                                            <td class="text-sm text-red-500 text-right">{miss}</td>
                                            <td class="text-sm text-gray-600 text-right">{vm}</td>
                                            <td class="text-sm text-gray-600 text-right">{ring}</td>
                                            <td class="text-sm text-gray-600 text-right">{talk}</td>
                                            <td class="text-sm text-gray-600 text-right">{conv}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 4. Today's Missed Calls
// ---------------------------------------------------------------------------

#[component]
pub fn TodaysMissedPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Missed Today"</div>
                            <div class="text-2xl font-bold text-red-500">"47"</div>
                            <div class="text-xs text-red-500">"+ 8 vs yesterday"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Callbacks Made"</div>
                            <div class="text-2xl font-bold text-green-600">"12"</div>
                            <div class="text-xs text-green-500">"25.5% callback rate"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Avg Response Time"</div>
                            <div class="text-2xl font-bold text-orange-500">"18 min"</div>
                            <div class="text-xs text-orange-500">"Target: < 10 min"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Still Pending"</div>
                            <div class="text-2xl font-bold text-red-500">"35"</div>
                            <div class="text-xs text-red-500">"Awaiting callback"</div>
                        </div>
                    </div>
                </div>

                // Chart: Hourly bar chart 8am-6pm
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Missed Calls by Hour"</h3>
                    <div class="h-36 flex items-end justify-around gap-1 px-2">
                        {[
                            ("8a", 2), ("9a", 5), ("10a", 8), ("11a", 6),
                            ("12p", 4), ("1p", 7), ("2p", 5), ("3p", 4),
                            ("4p", 3), ("5p", 2), ("6p", 1),
                        ].into_iter().map(|(hour, count)| {
                            let h = format!("height: {}px;", count * 14);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-xs text-gray-500 mb-1">{count}</span>
                                    <div class="w-6 bg-red-400 rounded-t" style=h></div>
                                    <span class="text-xs text-gray-400 mt-1">{hour}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Time"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Caller"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Phone"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Source"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Tracking Number"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Ring Time"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Callback Status"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("8:12 AM", "John Smith", "(910) 555-0142", "Google Organic", "(800) 555-0100", "0:32", "Completed"),
                                    ("8:45 AM", "Maria Lopez", "(910) 555-0198", "Facebook Paid", "(800) 555-0101", "0:28", "Pending"),
                                    ("9:23 AM", "Robert Chen", "(919) 555-0234", "Direct", "(800) 555-0100", "0:45", "Pending"),
                                    ("10:05 AM", "Sarah Davis", "(910) 555-0312", "Google Organic", "(800) 555-0102", "0:18", "Completed"),
                                    ("10:42 AM", "James Wilson", "(336) 555-0187", "TikTok", "(800) 555-0103", "0:52", "Pending"),
                                    ("11:15 AM", "Emily Brown", "(704) 555-0265", "Radio", "(800) 555-0104", "0:22", "No Answer"),
                                    ("1:30 PM", "Carlos Reyes", "(910) 555-0421", "Google Organic", "(800) 555-0100", "0:38", "Pending"),
                                    ("2:48 PM", "Lisa Park", "(919) 555-0543", "Facebook Paid", "(800) 555-0101", "0:15", "Completed"),
                                ].into_iter().map(|(time, caller, phone, source, tracking, ring, status)| {
                                    let status_class = match status {
                                        "Completed" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                        "No Answer" => "badge badge-sm bg-red-100 text-red-700 border-red-200",
                                        _ => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                    };
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm text-gray-600">{time}</td>
                                            <td class="text-sm font-medium">{caller}</td>
                                            <td class="text-sm text-iiz-cyan">{phone}</td>
                                            <td class="text-sm text-gray-600">{source}</td>
                                            <td class="text-sm text-gray-500">{tracking}</td>
                                            <td class="text-sm text-gray-600 text-right">{ring}</td>
                                            <td><span class=status_class>{status}</span></td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 5. Positive Daily Reports
// ---------------------------------------------------------------------------

#[component]
pub fn PositiveDailyPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Appointments Set"</div>
                            <div class="text-2xl font-bold text-green-600">"23"</div>
                            <div class="text-xs text-green-500">"+ 4 vs yesterday"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Conversions"</div>
                            <div class="text-2xl font-bold text-green-600">"18"</div>
                            <div class="text-xs text-green-500">"+ 3 vs yesterday"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Positive Rate"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"34.2%"</div>
                            <div class="text-xs text-green-500">"+ 2.1% vs avg"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Revenue"</div>
                            <div class="text-2xl font-bold text-green-600">"$12,450"</div>
                            <div class="text-xs text-green-500">"+ $1,200 vs yesterday"</div>
                        </div>
                    </div>
                </div>

                // Chart: Daily trend (last 7 days)
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Positive Outcomes Trend (7 Days)"</h3>
                    <div class="h-36 flex items-end justify-around gap-2 px-4">
                        {[
                            ("Feb 18", 28), ("Feb 19", 32), ("Feb 20", 25),
                            ("Feb 21", 34), ("Feb 22", 30), ("Feb 23", 22),
                            ("Feb 24", 34),
                        ].into_iter().map(|(day, val)| {
                            let h = format!("height: {}px;", val * 4);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-xs text-gray-500 mb-1">{val}</span>
                                    <div class="w-8 bg-green-400 rounded-t" style=h></div>
                                    <span class="text-xs text-gray-400 mt-1">{day}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Date"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Appointments"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Conversions"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Positive Calls"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Total Calls"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Positive%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Revenue"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Feb 18", "18", "14", "28", "85", "32.9%", "$9,800"),
                                    ("Feb 19", "22", "17", "32", "92", "34.8%", "$11,900"),
                                    ("Feb 20", "15", "12", "25", "78", "32.1%", "$8,750"),
                                    ("Feb 21", "24", "19", "34", "95", "35.8%", "$13,300"),
                                    ("Feb 22", "20", "15", "30", "88", "34.1%", "$10,500"),
                                    ("Feb 23", "12", "9", "22", "62", "35.5%", "$7,700"),
                                    ("Feb 24", "23", "18", "34", "97", "35.1%", "$12,450"),
                                ].into_iter().map(|(date, appt, conv, positive, total, pct, rev)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{date}</td>
                                            <td class="text-sm text-gray-600 text-right">{appt}</td>
                                            <td class="text-sm text-green-600 text-right">{conv}</td>
                                            <td class="text-sm text-gray-600 text-right">{positive}</td>
                                            <td class="text-sm text-gray-600 text-right">{total}</td>
                                            <td class="text-sm text-green-600 text-right">{pct}</td>
                                            <td class="text-sm text-green-600 text-right">{rev}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 6. Google CA Report
// ---------------------------------------------------------------------------

#[component]
pub fn GoogleCAPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Google Calls"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"2,847"</div>
                            <div class="text-xs text-green-500">"+ 12.5% vs last period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Click-to-Call"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"1,203"</div>
                            <div class="text-xs text-gray-400">"42.3% of total"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Cost / Call"</div>
                            <div class="text-2xl font-bold text-orange-500">"$8.72"</div>
                            <div class="text-xs text-green-500">"- $0.45 vs last period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Conversion Rate"</div>
                            <div class="text-2xl font-bold text-green-600">"12.4%"</div>
                            <div class="text-xs text-green-500">"+ 1.2% vs last period"</div>
                        </div>
                    </div>
                </div>

                // Chart: Campaign comparison bars
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Calls by Campaign"</h3>
                    <div class="h-36 flex items-end justify-around gap-3 px-4">
                        {[
                            ("Brand", 85), ("Local Svc", 65), ("Emergency", 48),
                            ("Reviews", 32), ("General", 20),
                        ].into_iter().map(|(name, pct)| {
                            let h = format!("height: {}%;", pct);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-xs text-gray-500 mb-1">{pct}"%"</span>
                                    <div class="w-10 bg-blue-400 rounded-t h-full" style=h></div>
                                    <span class="text-xs text-gray-400 mt-1 text-center">{name}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Campaign"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Ad Group"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Calls"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Impressions"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"CTR%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Cost"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Cost/Call"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Conversions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Brand", "Brand Terms", "982", "45,200", "2.17%", "$4,280", "$4.36", "145"),
                                    ("Local Services", "Area Targeting", "724", "38,100", "1.90%", "$7,820", "$10.80", "98"),
                                    ("Emergency", "24/7 Keywords", "512", "22,400", "2.29%", "$5,630", "$11.00", "62"),
                                    ("Reviews", "Reputation", "387", "31,200", "1.24%", "$3,480", "$8.99", "34"),
                                    ("General", "Broad Match", "242", "28,900", "0.84%", "$3,620", "$14.96", "14"),
                                ].into_iter().map(|(campaign, adgroup, calls, impr, ctr, cost, cpc, conv)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{campaign}</td>
                                            <td class="text-sm text-gray-600">{adgroup}</td>
                                            <td class="text-sm text-gray-600 text-right">{calls}</td>
                                            <td class="text-sm text-gray-600 text-right">{impr}</td>
                                            <td class="text-sm text-gray-600 text-right">{ctr}</td>
                                            <td class="text-sm text-red-500 text-right">{cost}</td>
                                            <td class="text-sm text-gray-600 text-right">{cpc}</td>
                                            <td class="text-sm text-green-600 text-right">{conv}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 7. Saturday Calls
// ---------------------------------------------------------------------------

#[component]
pub fn SaturdayCallsPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Total Saturday"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"1,247"</div>
                            <div class="text-xs text-gray-400">"Last Saturday"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Answered"</div>
                            <div class="text-2xl font-bold text-green-600">"823"</div>
                            <div class="text-xs text-green-500">"66.0% answer rate"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Missed"</div>
                            <div class="text-2xl font-bold text-red-500">"424"</div>
                            <div class="text-xs text-red-500">"34.0% miss rate"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"vs Weekday Avg"</div>
                            <div class="text-2xl font-bold text-red-500">"-18%"</div>
                            <div class="text-xs text-red-500">"Below weekday average"</div>
                        </div>
                    </div>
                </div>

                // Chart: Hourly bars 6am-8pm
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Saturday Calls by Hour"</h3>
                    <div class="h-36 flex items-end justify-around gap-1 px-2">
                        {[
                            ("6a", 12), ("7a", 28), ("8a", 65), ("9a", 98),
                            ("10a", 125), ("11a", 118), ("12p", 95), ("1p", 102),
                            ("2p", 110), ("3p", 98), ("4p", 85), ("5p", 72),
                            ("6p", 58), ("7p", 42), ("8p", 18),
                        ].into_iter().map(|(hour, count)| {
                            let h = format!("height: {}%;", (count as f32 / 125.0 * 100.0) as u32);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <div class="w-4 bg-orange-400 rounded-t" style=h></div>
                                    <span class="text-[10px] text-gray-400 mt-1">{hour}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Hour"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Calls"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Answered"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Answer%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Wait"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Talk"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("6:00 AM", "12", "8", "4", "66.7%", "0:42", "1:15"),
                                    ("7:00 AM", "28", "20", "8", "71.4%", "0:35", "1:28"),
                                    ("8:00 AM", "65", "48", "17", "73.8%", "0:28", "1:45"),
                                    ("9:00 AM", "98", "72", "26", "73.5%", "0:25", "2:02"),
                                    ("10:00 AM", "125", "88", "37", "70.4%", "0:30", "2:18"),
                                    ("11:00 AM", "118", "82", "36", "69.5%", "0:32", "2:12"),
                                    ("12:00 PM", "95", "65", "30", "68.4%", "0:35", "1:55"),
                                    ("1:00 PM", "102", "70", "32", "68.6%", "0:33", "2:05"),
                                    ("2:00 PM", "110", "75", "35", "68.2%", "0:34", "2:10"),
                                    ("3:00 PM", "98", "68", "30", "69.4%", "0:30", "2:00"),
                                    ("4:00 PM", "85", "55", "30", "64.7%", "0:38", "1:48"),
                                    ("5:00 PM", "72", "42", "30", "58.3%", "0:42", "1:35"),
                                    ("6:00 PM", "58", "32", "26", "55.2%", "0:48", "1:22"),
                                    ("7:00 PM", "42", "22", "20", "52.4%", "0:52", "1:12"),
                                ].into_iter().map(|(hour, calls, ans, miss, pct, wait, talk)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{hour}</td>
                                            <td class="text-sm text-gray-600 text-right">{calls}</td>
                                            <td class="text-sm text-green-600 text-right">{ans}</td>
                                            <td class="text-sm text-red-500 text-right">{miss}</td>
                                            <td class="text-sm text-gray-600 text-right">{pct}</td>
                                            <td class="text-sm text-gray-600 text-right">{wait}</td>
                                            <td class="text-sm text-gray-600 text-right">{talk}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 8. Daily Calls
// ---------------------------------------------------------------------------

#[component]
pub fn DailyCallsPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Period Total"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"15,407"</div>
                            <div class="text-xs text-gray-400">"Last 7 days"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Daily Average"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"2,201"</div>
                            <div class="text-xs text-green-500">"+ 3.2% vs prior week"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Peak Day"</div>
                            <div class="text-2xl font-bold text-green-600">"Tue 2,847"</div>
                            <div class="text-xs text-gray-400">"Highest volume"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Low Day"</div>
                            <div class="text-2xl font-bold text-red-500">"Sat 1,247"</div>
                            <div class="text-xs text-gray-400">"Lowest volume"</div>
                        </div>
                    </div>
                </div>

                // Chart: Daily bars for last 7 days
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Daily Call Volume"</h3>
                    <div class="h-40 flex items-end justify-around gap-3 px-4">
                        {[
                            ("Mon", 2180), ("Tue", 2847), ("Wed", 2340),
                            ("Thu", 2520), ("Fri", 2105), ("Sat", 1247), ("Sun", 968),
                        ].into_iter().map(|(day, count)| {
                            let h = format!("height: {}%;", (count as f32 / 2847.0 * 100.0) as u32);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-xs text-gray-500 mb-1">{format!("{}", count)}</span>
                                    <div class="w-10 bg-blue-400 rounded-t" style=h></div>
                                    <span class="text-xs text-gray-400 mt-1">{day}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Date"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Day"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Total"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Answered"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Answer%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"First-Time"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Repeat"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Duration"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Feb 18", "Mon", "2,180", "1,812", "368", "83.1%", "642", "1,538", "2:25"),
                                    ("Feb 19", "Tue", "2,847", "2,392", "455", "84.0%", "823", "2,024", "2:32"),
                                    ("Feb 20", "Wed", "2,340", "1,965", "375", "84.0%", "698", "1,642", "2:18"),
                                    ("Feb 21", "Thu", "2,520", "2,092", "428", "83.0%", "745", "1,775", "2:22"),
                                    ("Feb 22", "Fri", "2,105", "1,726", "379", "82.0%", "612", "1,493", "2:10"),
                                    ("Feb 23", "Sat", "1,247", "823", "424", "66.0%", "398", "849", "1:48"),
                                    ("Feb 24", "Sun", "968", "612", "356", "63.2%", "285", "683", "1:32"),
                                ].into_iter().map(|(date, day, total, ans, miss, pct, first, repeat, dur)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{date}</td>
                                            <td class="text-sm text-gray-600">{day}</td>
                                            <td class="text-sm text-gray-600 text-right">{total}</td>
                                            <td class="text-sm text-green-600 text-right">{ans}</td>
                                            <td class="text-sm text-red-500 text-right">{miss}</td>
                                            <td class="text-sm text-gray-600 text-right">{pct}</td>
                                            <td class="text-sm text-gray-600 text-right">{first}</td>
                                            <td class="text-sm text-gray-600 text-right">{repeat}</td>
                                            <td class="text-sm text-gray-600 text-right">{dur}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 9. Weekly Missed Calls
// ---------------------------------------------------------------------------

#[component]
pub fn WeeklyMissedPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"This Week"</div>
                            <div class="text-2xl font-bold text-red-500">"312"</div>
                            <div class="text-xs text-red-500">"Current week missed"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Last Week"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"287"</div>
                            <div class="text-xs text-gray-400">"Prior week missed"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Change"</div>
                            <div class="text-2xl font-bold text-red-500">"+8.7%"</div>
                            <div class="text-xs text-red-500">"Week over week"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Avg Response"</div>
                            <div class="text-2xl font-bold text-orange-500">"22 min"</div>
                            <div class="text-xs text-orange-500">"Target: < 15 min"</div>
                        </div>
                    </div>
                </div>

                // Chart: Week-over-week comparison (4 weeks)
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Weekly Missed Calls Trend"</h3>
                    <div class="h-36 flex items-end justify-around gap-4 px-8">
                        {[
                            ("Week 1", 265), ("Week 2", 298), ("Week 3", 287), ("Week 4", 312),
                        ].into_iter().map(|(week, count)| {
                            let h = format!("height: {}%;", (count as f32 / 312.0 * 100.0) as u32);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-xs text-gray-500 mb-1">{count}</span>
                                    <div class="w-16 bg-red-400 rounded-t" style=h></div>
                                    <span class="text-xs text-gray-400 mt-1">{week}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Week"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Total Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"After Hours"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"During Hours"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Callback%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Response"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Worst Hour"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Feb 3 - Feb 9", "265", "112", "153", "38.5%", "19 min", "10:00 AM"),
                                    ("Feb 10 - Feb 16", "298", "124", "174", "35.2%", "21 min", "11:00 AM"),
                                    ("Feb 17 - Feb 23", "287", "118", "169", "36.9%", "20 min", "10:00 AM"),
                                    ("Feb 24 - Mar 2", "312", "132", "180", "34.2%", "22 min", "2:00 PM"),
                                ].into_iter().map(|(week, total, after, during, cb, resp, worst)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{week}</td>
                                            <td class="text-sm text-red-500 text-right">{total}</td>
                                            <td class="text-sm text-gray-600 text-right">{after}</td>
                                            <td class="text-sm text-gray-600 text-right">{during}</td>
                                            <td class="text-sm text-gray-600 text-right">{cb}</td>
                                            <td class="text-sm text-orange-500 text-right">{resp}</td>
                                            <td class="text-sm text-gray-600">{worst}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 10. Priming Calls
// ---------------------------------------------------------------------------

#[component]
pub fn PrimingCallsPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Priming Calls"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"4,523"</div>
                            <div class="text-xs text-gray-400">"Total attempted"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Connected"</div>
                            <div class="text-2xl font-bold text-green-600">"3,891"</div>
                            <div class="text-xs text-green-500">"86.0% connect rate"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Conversions"</div>
                            <div class="text-2xl font-bold text-green-600">"862"</div>
                            <div class="text-xs text-green-500">"+ 45 vs last period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Success Rate"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"22.1%"</div>
                            <div class="text-xs text-green-500">"+ 1.8% vs last period"</div>
                        </div>
                    </div>
                </div>

                // Chart: Funnel visualization
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Priming Funnel"</h3>
                    <div class="space-y-2 px-4">
                        {[
                            ("Attempted", 4523, "bg-blue-400", 100),
                            ("Connected", 3891, "bg-green-400", 86),
                            ("Qualified", 1845, "bg-yellow-400", 41),
                            ("Converted", 862, "bg-green-600", 19),
                        ].into_iter().map(|(stage, count, color, pct)| {
                            let w = format!("width: {}%;", pct);
                            let bar_class = format!("h-8 rounded flex items-center px-3 text-white text-sm font-medium {}", color);
                            view! {
                                <div>
                                    <div class="flex justify-between text-xs text-gray-500 mb-1">
                                        <span>{stage}</span>
                                        <span>{format!("{} ({}%)", count, pct)}</span>
                                    </div>
                                    <div class=bar_class style=w>{format!("{}", count)}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Source"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Attempted"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Connected"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Qualified"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Converted"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Connect%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Convert%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Duration"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Google Organic", "1,820", "1,592", "782", "368", "87.5%", "23.1%", "3:12"),
                                    ("Facebook Paid", "1,245", "1,058", "498", "212", "85.0%", "20.1%", "2:48"),
                                    ("TikTok", "680", "578", "245", "118", "85.0%", "20.4%", "2:35"),
                                    ("Direct", "452", "389", "198", "102", "86.1%", "26.2%", "3:45"),
                                    ("Radio", "326", "274", "122", "62", "84.0%", "22.6%", "2:55"),
                                ].into_iter().map(|(source, att, conn, qual, conv, conn_pct, conv_pct, dur)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{source}</td>
                                            <td class="text-sm text-gray-600 text-right">{att}</td>
                                            <td class="text-sm text-green-600 text-right">{conn}</td>
                                            <td class="text-sm text-gray-600 text-right">{qual}</td>
                                            <td class="text-sm text-green-600 text-right">{conv}</td>
                                            <td class="text-sm text-gray-600 text-right">{conn_pct}</td>
                                            <td class="text-sm text-green-600 text-right">{conv_pct}</td>
                                            <td class="text-sm text-gray-600 text-right">{dur}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 11. Missed Calls
// ---------------------------------------------------------------------------

#[component]
pub fn MissedCallsPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Total Missed"</div>
                            <div class="text-2xl font-bold text-red-500">"15,407"</div>
                            <div class="text-xs text-red-500">"14.0% of all calls"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Business Hours"</div>
                            <div class="text-2xl font-bold text-orange-500">"9,244"</div>
                            <div class="text-xs text-gray-400">"60.0% of missed"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"After Hours"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"6,163"</div>
                            <div class="text-xs text-gray-400">"40.0% of missed"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Callback Rate"</div>
                            <div class="text-2xl font-bold text-green-600">"34.2%"</div>
                            <div class="text-xs text-green-500">"+ 2.4% vs last period"</div>
                        </div>
                    </div>
                </div>

                // Chart: Missed calls by hour of day (24 bars)
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Missed Calls by Hour of Day"</h3>
                    <div class="h-32 flex items-end gap-px px-2">
                        {[
                            45, 32, 18, 12, 8, 15, 85, 220, 380, 520, 610, 580,
                            490, 540, 510, 480, 420, 350, 280, 180, 120, 85, 62, 48,
                        ].into_iter().enumerate().map(|(i, count)| {
                            let h = format!("height: {}%;", (count as f32 / 610.0 * 100.0) as u32);
                            let color = if (8..18).contains(&i) { "bg-red-400" } else { "bg-red-200" };
                            let class = format!("flex-1 rounded-t {}", color);
                            view! {
                                <div class=class style=h title=format!("{}:00 - {} missed", i, count)></div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div class="flex justify-between mt-1 text-[10px] text-gray-400 px-2">
                        <span>"12am"</span><span>"6am"</span><span>"12pm"</span><span>"6pm"</span><span>"11pm"</span>
                    </div>
                    <div class="flex gap-4 mt-2 text-xs text-gray-400">
                        <span class="flex items-center gap-1"><span class="w-3 h-3 bg-red-400 rounded inline-block"></span>"Business Hours"</span>
                        <span class="flex items-center gap-1"><span class="w-3 h-3 bg-red-200 rounded inline-block"></span>"After Hours"</span>
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Source"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Total Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Business Hrs"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"After Hrs"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Ring"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Callback Made"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Callback%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Rev Lost (est.)"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Google Organic", "8,245", "4,947", "3,298", "0:28", "2,820", "34.2%", "$82,450"),
                                    ("Facebook Paid", "2,890", "1,734", "1,156", "0:32", "988", "34.2%", "$28,900"),
                                    ("TikTok", "1,520", "912", "608", "0:25", "520", "34.2%", "$15,200"),
                                    ("Direct", "1,245", "747", "498", "0:30", "426", "34.2%", "$12,450"),
                                    ("Radio", "890", "534", "356", "0:35", "304", "34.2%", "$8,900"),
                                    ("Instagram", "617", "370", "247", "0:27", "211", "34.2%", "$6,170"),
                                ].into_iter().map(|(source, total, biz, after, ring, cb, cb_pct, rev)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{source}</td>
                                            <td class="text-sm text-red-500 text-right">{total}</td>
                                            <td class="text-sm text-gray-600 text-right">{biz}</td>
                                            <td class="text-sm text-gray-600 text-right">{after}</td>
                                            <td class="text-sm text-gray-600 text-right">{ring}</td>
                                            <td class="text-sm text-green-600 text-right">{cb}</td>
                                            <td class="text-sm text-gray-600 text-right">{cb_pct}</td>
                                            <td class="text-sm text-red-500 text-right">{rev}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 12. Missed Calls Daily - 1st
// ---------------------------------------------------------------------------

#[component]
pub fn MissedDaily1stPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"First-Time Missed"</div>
                            <div class="text-2xl font-bold text-red-500">"2,341"</div>
                            <div class="text-xs text-red-500">"New callers missed"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"New Leads Lost (est.)"</div>
                            <div class="text-2xl font-bold text-red-500">"1,872"</div>
                            <div class="text-xs text-red-500">"80% estimated loss"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Callback < 5min"</div>
                            <div class="text-2xl font-bold text-green-600">"423"</div>
                            <div class="text-xs text-green-500">"18.1% rapid response"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Recovery Rate"</div>
                            <div class="text-2xl font-bold text-orange-500">"18.1%"</div>
                            <div class="text-xs text-orange-500">"Target: > 30%"</div>
                        </div>
                    </div>
                </div>

                // Chart: Daily trend bars for first-time missed
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"First-Time Missed Calls Trend"</h3>
                    <div class="h-36 flex items-end justify-around gap-2 px-4">
                        {[
                            ("Feb 18", 312), ("Feb 19", 378), ("Feb 20", 298),
                            ("Feb 21", 356), ("Feb 22", 342), ("Feb 23", 412), ("Feb 24", 243),
                        ].into_iter().map(|(day, count)| {
                            let h = format!("height: {}%;", (count as f32 / 412.0 * 100.0) as u32);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-xs text-gray-500 mb-1">{count}</span>
                                    <div class="w-8 bg-red-400 rounded-t" style=h></div>
                                    <span class="text-xs text-gray-400 mt-1">{day}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Date"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"1st-Time Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Repeat Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Total Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"1st-Time%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"CB < 5min"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"CB < 30min"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Recovered"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Feb 18", "312", "56", "368", "84.8%", "58", "124", "72"),
                                    ("Feb 19", "378", "77", "455", "83.1%", "65", "142", "85"),
                                    ("Feb 20", "298", "77", "375", "79.5%", "52", "118", "68"),
                                    ("Feb 21", "356", "72", "428", "83.2%", "62", "134", "78"),
                                    ("Feb 22", "342", "37", "379", "90.2%", "60", "128", "75"),
                                    ("Feb 23", "412", "12", "424", "97.2%", "72", "155", "42"),
                                    ("Feb 24", "243", "113", "356", "68.3%", "54", "98", "62"),
                                ].into_iter().map(|(date, first, repeat, total, pct, cb5, cb30, recovered)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{date}</td>
                                            <td class="text-sm text-red-500 text-right">{first}</td>
                                            <td class="text-sm text-gray-600 text-right">{repeat}</td>
                                            <td class="text-sm text-gray-600 text-right">{total}</td>
                                            <td class="text-sm text-gray-600 text-right">{pct}</td>
                                            <td class="text-sm text-green-600 text-right">{cb5}</td>
                                            <td class="text-sm text-gray-600 text-right">{cb30}</td>
                                            <td class="text-sm text-green-600 text-right">{recovered}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 13. CS Daily Missed
// ---------------------------------------------------------------------------

#[component]
pub fn CSDailyMissedPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"CS Missed Today"</div>
                            <div class="text-2xl font-bold text-red-500">"89"</div>
                            <div class="text-xs text-red-500">"+ 12 vs yesterday"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Queue Avg Wait"</div>
                            <div class="text-2xl font-bold text-orange-500">"3:42"</div>
                            <div class="text-xs text-orange-500">"Above 2:00 target"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Abandonment Rate"</div>
                            <div class="text-2xl font-bold text-red-500">"12.4%"</div>
                            <div class="text-xs text-red-500">"Above 8% threshold"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"SLA Met"</div>
                            <div class="text-2xl font-bold text-orange-500">"87.6%"</div>
                            <div class="text-xs text-orange-500">"Target: 95%"</div>
                        </div>
                    </div>
                </div>

                // Chart: Queue-level comparison bars
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Missed by Queue"</h3>
                    <div class="h-36 flex items-end justify-around gap-4 px-8">
                        {[
                            ("Main CS", 38), ("Billing", 24), ("Support", 18), ("Spanish", 9),
                        ].into_iter().map(|(queue, count)| {
                            let h = format!("height: {}%;", (count as f32 / 38.0 * 100.0) as u32);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-xs text-gray-500 mb-1">{count}</span>
                                    <div class="w-16 bg-red-400 rounded-t" style=h></div>
                                    <span class="text-xs text-gray-400 mt-1">{queue}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Queue"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Abandoned"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Wait"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Max Wait"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"SLA%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Agents Avail"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"After Hours"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Main CS", "38", "12", "4:15", "12:30", "82.4%", "4", "8"),
                                    ("Billing", "24", "8", "3:45", "9:20", "85.2%", "3", "6"),
                                    ("Support", "18", "5", "2:58", "7:45", "91.5%", "5", "4"),
                                    ("Spanish", "9", "3", "3:30", "8:10", "88.0%", "2", "3"),
                                ].into_iter().map(|(queue, missed, abandoned, avg_wait, max_wait, sla, agents, after)| {
                                    let sla_class = if sla.starts_with("9") { "text-sm text-green-600 text-right" } else { "text-sm text-orange-500 text-right" };
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{queue}</td>
                                            <td class="text-sm text-red-500 text-right">{missed}</td>
                                            <td class="text-sm text-red-500 text-right">{abandoned}</td>
                                            <td class="text-sm text-orange-500 text-right">{avg_wait}</td>
                                            <td class="text-sm text-red-500 text-right">{max_wait}</td>
                                            <td class=sla_class>{sla}</td>
                                            <td class="text-sm text-gray-600 text-right">{agents}</td>
                                            <td class="text-sm text-gray-600 text-right">{after}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 14. CS Daily Missed 2.0
// ---------------------------------------------------------------------------

#[component]
pub fn CSDailyMissed2Page() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"CS Missed"</div>
                            <div class="text-2xl font-bold text-red-500">"89"</div>
                            <div class="text-xs text-gray-400">"Total missed today"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"By Agent Gap"</div>
                            <div class="text-2xl font-bold text-orange-500">"34"</div>
                            <div class="text-xs text-orange-500">"38.2% of missed"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"By Overflow"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"28"</div>
                            <div class="text-xs text-gray-400">"31.5% of missed"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"By Timeout"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"27"</div>
                            <div class="text-xs text-gray-400">"30.3% of missed"</div>
                        </div>
                    </div>
                </div>

                // Chart: Stacked bars showing miss reasons per queue
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Miss Reasons by Queue"</h3>
                    <div class="space-y-3 px-4">
                        {[
                            ("Main CS", 15, 12, 11),
                            ("Billing", 9, 8, 7),
                            ("Support", 6, 5, 7),
                            ("Spanish", 4, 3, 2),
                        ].into_iter().map(|(queue, gap, overflow, timeout)| {
                            let total = gap + overflow + timeout;
                            let gap_w = format!("width: {}%;", gap * 100 / total);
                            let overflow_w = format!("width: {}%;", overflow * 100 / total);
                            let timeout_w = format!("width: {}%;", timeout * 100 / total);
                            view! {
                                <div>
                                    <div class="flex justify-between text-xs text-gray-500 mb-1">
                                        <span>{queue}</span>
                                        <span>{format!("{} total", total)}</span>
                                    </div>
                                    <div class="flex h-5 bg-gray-100 rounded overflow-hidden">
                                        <div class="bg-orange-400" style=gap_w></div>
                                        <div class="bg-blue-400" style=overflow_w></div>
                                        <div class="bg-gray-400" style=timeout_w></div>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                        <div class="flex gap-4 mt-2 text-xs text-gray-400">
                            <span class="flex items-center gap-1"><span class="w-3 h-3 bg-orange-400 rounded inline-block"></span>"Agent Gap"</span>
                            <span class="flex items-center gap-1"><span class="w-3 h-3 bg-blue-400 rounded inline-block"></span>"Overflow"</span>
                            <span class="flex items-center gap-1"><span class="w-3 h-3 bg-gray-400 rounded inline-block"></span>"Timeout"</span>
                        </div>
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Agent"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Queue"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Reason"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Handle"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Availability%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Missed @ Lunch"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Missed After Hrs"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Maria Garcia", "Main CS", "8", "Agent Gap", "4:12", "82%", "3", "2"),
                                    ("James Wilson", "Support", "6", "Timeout", "5:45", "75%", "2", "1"),
                                    ("Sarah Chen", "Billing", "5", "Overflow", "3:28", "88%", "1", "2"),
                                    ("Mike Johnson", "Main CS", "7", "Agent Gap", "6:02", "68%", "4", "1"),
                                    ("Emily Davis", "Spanish", "4", "Timeout", "3:55", "85%", "1", "1"),
                                    ("Carlos Reyes", "Support", "3", "Overflow", "4:30", "90%", "0", "2"),
                                ].into_iter().map(|(agent, queue, missed, reason, handle, avail, lunch, after)| {
                                    let reason_class = match reason {
                                        "Agent Gap" => "badge badge-sm bg-orange-100 text-orange-700 border-orange-200",
                                        "Overflow" => "badge badge-sm bg-blue-100 text-blue-700 border-blue-200",
                                        _ => "badge badge-sm bg-gray-100 text-gray-700 border-gray-200",
                                    };
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{agent}</td>
                                            <td class="text-sm text-gray-600">{queue}</td>
                                            <td class="text-sm text-red-500 text-right">{missed}</td>
                                            <td><span class=reason_class>{reason}</span></td>
                                            <td class="text-sm text-gray-600 text-right">{handle}</td>
                                            <td class="text-sm text-gray-600 text-right">{avail}</td>
                                            <td class="text-sm text-gray-600 text-right">{lunch}</td>
                                            <td class="text-sm text-gray-600 text-right">{after}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 15. Priming Missed Calls
// ---------------------------------------------------------------------------

#[component]
pub fn PrimingMissedPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Priming Missed"</div>
                            <div class="text-2xl font-bold text-red-500">"632"</div>
                            <div class="text-xs text-red-500">"14.0% of priming calls"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Not Reached"</div>
                            <div class="text-2xl font-bold text-orange-500">"418"</div>
                            <div class="text-xs text-gray-400">"66.1% of missed"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Voicemail"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"214"</div>
                            <div class="text-xs text-gray-400">"33.9% of missed"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Retry Pending"</div>
                            <div class="text-2xl font-bold text-orange-500">"156"</div>
                            <div class="text-xs text-orange-500">"Scheduled for retry"</div>
                        </div>
                    </div>
                </div>

                // Chart: Daily trend of priming missed
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Priming Missed Calls Trend"</h3>
                    <div class="h-36 flex items-end justify-around gap-2 px-4">
                        {[
                            ("Feb 18", 82), ("Feb 19", 98), ("Feb 20", 75),
                            ("Feb 21", 105), ("Feb 22", 92), ("Feb 23", 110), ("Feb 24", 70),
                        ].into_iter().map(|(day, count)| {
                            let h = format!("height: {}%;", (count as f32 / 110.0 * 100.0) as u32);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-xs text-gray-500 mb-1">{count}</span>
                                    <div class="w-8 bg-orange-400 rounded-t" style=h></div>
                                    <span class="text-xs text-gray-400 mt-1">{day}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Date"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Attempted"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Not Reached"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Voicemail"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Busy"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"No Answer"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Retry Sched."</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Final Status"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Feb 18", "620", "52", "30", "8", "44", "18", "Partial"),
                                    ("Feb 19", "680", "65", "33", "12", "53", "22", "Partial"),
                                    ("Feb 20", "590", "48", "27", "6", "42", "15", "Partial"),
                                    ("Feb 21", "720", "72", "33", "10", "62", "25", "Partial"),
                                    ("Feb 22", "650", "60", "32", "9", "51", "20", "Partial"),
                                    ("Feb 23", "740", "78", "32", "14", "64", "28", "Pending"),
                                    ("Feb 24", "523", "43", "27", "5", "38", "28", "Pending"),
                                ].into_iter().map(|(date, att, nr, vm, busy, na, retry, status)| {
                                    let status_class = match status {
                                        "Pending" => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                        _ => "badge badge-sm bg-gray-100 text-gray-700 border-gray-200",
                                    };
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{date}</td>
                                            <td class="text-sm text-gray-600 text-right">{att}</td>
                                            <td class="text-sm text-red-500 text-right">{nr}</td>
                                            <td class="text-sm text-gray-600 text-right">{vm}</td>
                                            <td class="text-sm text-gray-600 text-right">{busy}</td>
                                            <td class="text-sm text-gray-600 text-right">{na}</td>
                                            <td class="text-sm text-orange-500 text-right">{retry}</td>
                                            <td><span class=status_class>{status}</span></td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 16. Daily Collection Calls
// ---------------------------------------------------------------------------

#[component]
pub fn DailyCollectionPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Collection Calls"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"342"</div>
                            <div class="text-xs text-gray-400">"Made today"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Connected"</div>
                            <div class="text-2xl font-bold text-green-600">"218"</div>
                            <div class="text-xs text-green-500">"63.7% connect rate"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Promises"</div>
                            <div class="text-2xl font-bold text-green-600">"47"</div>
                            <div class="text-xs text-green-500">"21.6% promise rate"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Amount Collected"</div>
                            <div class="text-2xl font-bold text-green-600">"$23,450"</div>
                            <div class="text-xs text-green-500">"+ $3,200 vs yesterday"</div>
                        </div>
                    </div>
                </div>

                // Chart: Daily bars connected vs not connected
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Daily Collection Results"</h3>
                    <div class="h-36 flex items-end justify-around gap-2 px-4">
                        {[
                            ("Feb 18", 58, 32), ("Feb 19", 72, 28), ("Feb 20", 48, 35),
                            ("Feb 21", 65, 30), ("Feb 22", 55, 38), ("Feb 23", 42, 22),
                            ("Feb 24", 68, 24),
                        ].into_iter().map(|(day, connected, not_connected)| {
                            let conn_h = format!("height: {}px;", connected);
                            let nc_h = format!("height: {}px;", not_connected);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <div class="flex gap-0.5 items-end">
                                        <div class="w-4 bg-green-400 rounded-t" style=conn_h></div>
                                        <div class="w-4 bg-red-300 rounded-t" style=nc_h></div>
                                    </div>
                                    <span class="text-xs text-gray-400 mt-1">{day}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div class="flex gap-4 mt-2 text-xs text-gray-400 justify-center">
                        <span class="flex items-center gap-1"><span class="w-3 h-3 bg-green-400 rounded inline-block"></span>"Connected"</span>
                        <span class="flex items-center gap-1"><span class="w-3 h-3 bg-red-300 rounded inline-block"></span>"Not Connected"</span>
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Agent"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Calls Made"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Connected"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Promises"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Amt Promised"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Collected"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Connect%"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Promise%"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Maria Garcia", "82", "55", "14", "$8,200", "$6,450", "67.1%", "25.5%"),
                                    ("James Wilson", "75", "48", "12", "$7,500", "$5,800", "64.0%", "25.0%"),
                                    ("Sarah Chen", "68", "45", "10", "$5,800", "$4,200", "66.2%", "22.2%"),
                                    ("Mike Johnson", "62", "38", "6", "$3,200", "$3,800", "61.3%", "15.8%"),
                                    ("Emily Davis", "55", "32", "5", "$4,100", "$3,200", "58.2%", "15.6%"),
                                ].into_iter().map(|(agent, calls, conn, promises, promised, collected, conn_pct, promise_pct)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{agent}</td>
                                            <td class="text-sm text-gray-600 text-right">{calls}</td>
                                            <td class="text-sm text-green-600 text-right">{conn}</td>
                                            <td class="text-sm text-green-600 text-right">{promises}</td>
                                            <td class="text-sm text-gray-600 text-right">{promised}</td>
                                            <td class="text-sm text-green-600 text-right">{collected}</td>
                                            <td class="text-sm text-gray-600 text-right">{conn_pct}</td>
                                            <td class="text-sm text-gray-600 text-right">{promise_pct}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 17. Power BI - Total Inbound
// ---------------------------------------------------------------------------

#[component]
pub fn PowerBIPage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Total Inbound"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"110,050"</div>
                            <div class="text-xs text-green-500">"+ 5.4% vs last period"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Unique Callers"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"31,721"</div>
                            <div class="text-xs text-gray-400">"28.8% of total"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Avg Handle Time"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"2:18"</div>
                            <div class="text-xs text-gray-400">"Minutes per call"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Busiest Hour"</div>
                            <div class="text-2xl font-bold text-orange-500">"10:00 AM"</div>
                            <div class="text-xs text-gray-400">"Peak inbound volume"</div>
                        </div>
                    </div>
                </div>

                // Chart: Wide hourly distribution
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Hourly Inbound Distribution"</h3>
                    <div class="h-40 flex items-end gap-1 px-2">
                        {[
                            ("8a", 3200), ("9a", 7800), ("10a", 12400), ("11a", 11200),
                            ("12p", 8900), ("1p", 10200), ("2p", 11800), ("3p", 10500),
                            ("4p", 9200), ("5p", 7400), ("6p", 5100), ("7p", 3200),
                        ].into_iter().map(|(hour, count)| {
                            let h = format!("height: {}%;", (count as f32 / 12400.0 * 100.0) as u32);
                            view! {
                                <div class="flex flex-col items-center flex-1">
                                    <span class="text-[10px] text-gray-500 mb-1">{format!("{}k", count / 1000)}</span>
                                    <div class="w-full bg-blue-400 rounded-t" style=h></div>
                                    <span class="text-[10px] text-gray-400 mt-1">{hour}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Hour"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Inbound"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Answered"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Missed"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Voicemail"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Wait"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Talk"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Peak Agents"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("8:00 AM", "3,200", "2,816", "384", "182", "0:18", "2:05", "8"),
                                    ("9:00 AM", "7,800", "6,786", "1,014", "487", "0:22", "2:12", "12"),
                                    ("10:00 AM", "12,400", "10,416", "1,984", "952", "0:28", "2:25", "15"),
                                    ("11:00 AM", "11,200", "9,520", "1,680", "806", "0:25", "2:20", "14"),
                                    ("12:00 PM", "8,900", "7,476", "1,424", "684", "0:23", "2:08", "12"),
                                    ("1:00 PM", "10,200", "8,670", "1,530", "735", "0:24", "2:18", "14"),
                                    ("2:00 PM", "11,800", "9,912", "1,888", "906", "0:27", "2:22", "15"),
                                    ("3:00 PM", "10,500", "8,925", "1,575", "756", "0:24", "2:15", "14"),
                                    ("4:00 PM", "9,200", "7,728", "1,472", "707", "0:25", "2:10", "12"),
                                    ("5:00 PM", "7,400", "6,216", "1,184", "568", "0:28", "2:02", "10"),
                                    ("6:00 PM", "5,100", "4,182", "918", "441", "0:32", "1:52", "8"),
                                    ("7:00 PM", "3,200", "2,432", "768", "369", "0:38", "1:42", "6"),
                                ].into_iter().map(|(hour, inbound, ans, miss, vm, wait, talk, agents)| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{hour}</td>
                                            <td class="text-sm text-gray-600 text-right">{inbound}</td>
                                            <td class="text-sm text-green-600 text-right">{ans}</td>
                                            <td class="text-sm text-red-500 text-right">{miss}</td>
                                            <td class="text-sm text-gray-600 text-right">{vm}</td>
                                            <td class="text-sm text-gray-600 text-right">{wait}</td>
                                            <td class="text-sm text-gray-600 text-right">{talk}</td>
                                            <td class="text-sm text-gray-600 text-right">{agents}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 18. Real Time
// ---------------------------------------------------------------------------

#[component]
pub fn RealTimePage() -> impl IntoView {
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
                // KPI cards
                <div class="grid grid-cols-4 gap-4 p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Active Calls"</div>
                            <div class="text-2xl font-bold text-green-600">"23"</div>
                            <div class="flex items-center gap-1 mt-1">
                                <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
                                <span class="text-xs text-green-500">"Live"</span>
                            </div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"In Queue"</div>
                            <div class="text-2xl font-bold text-orange-500">"7"</div>
                            <div class="text-xs text-orange-500">"Waiting for agent"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Agents Available"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"12"</div>
                            <div class="text-xs text-gray-400">"Ready to take calls"</div>
                        </div>
                    </div>
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-4">
                            <div class="text-xs text-gray-500 uppercase font-medium">"Avg Wait"</div>
                            <div class="text-2xl font-bold text-iiz-dark">"0:45"</div>
                            <div class="text-xs text-green-500">"Below 1:00 target"</div>
                        </div>
                    </div>
                </div>

                // Live status indicators (instead of chart)
                <div class="bg-white border mx-4 mb-4 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-gray-700 mb-3">"Live Status"</h3>
                    <div class="grid grid-cols-4 gap-3">
                        {[
                            ("Ringing", 4, "bg-yellow-500", "text-yellow-700"),
                            ("Connected", 15, "bg-green-500", "text-green-700"),
                            ("On Hold", 2, "bg-orange-500", "text-orange-700"),
                            ("Wrapping Up", 2, "bg-blue-500", "text-blue-700"),
                        ].into_iter().map(|(status, count, dot_color, text_color)| {
                            let dot_class = format!("w-3 h-3 rounded-full {}", dot_color);
                            let text_class = format!("text-lg font-bold {}", text_color);
                            view! {
                                <div class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                                    <span class=dot_class></span>
                                    <div>
                                        <div class=text_class>{count}</div>
                                        <div class="text-xs text-gray-500">{status}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Caller"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Phone"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Source"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Queue"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Agent"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Duration"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase text-right">"Wait Time"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("John Smith", "(910) 555-0142", "Google", "Sales", "Maria G.", "Connected", "3:42", "0:18"),
                                    ("Maria Lopez", "(910) 555-0198", "Facebook", "Support", "James W.", "Connected", "7:18", "0:22"),
                                    ("Robert Chen", "(919) 555-0234", "Direct", "Sales", "Sarah C.", "On Hold", "2:05", "0:12"),
                                    ("Sarah Davis", "(910) 555-0312", "Google", "Billing", "---", "Ringing", "0:15", "0:45"),
                                    ("James Wilson", "(336) 555-0187", "TikTok", "Sales", "Emily D.", "Connected", "1:30", "0:08"),
                                    ("Emily Brown", "(704) 555-0265", "Radio", "Support", "---", "Ringing", "0:08", "1:02"),
                                    ("Carlos Reyes", "(910) 555-0421", "Google", "Sales", "Robert T.", "Wrapping Up", "5:22", "0:15"),
                                    ("Lisa Park", "(919) 555-0543", "Facebook", "Billing", "Mike J.", "Connected", "4:15", "0:32"),
                                ].into_iter().map(|(caller, phone, source, queue, agent, status, duration, wait)| {
                                    let status_class = match status {
                                        "Connected" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                        "Ringing" => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                        "On Hold" => "badge badge-sm bg-orange-100 text-orange-700 border-orange-200",
                                        "Wrapping Up" => "badge badge-sm bg-blue-100 text-blue-700 border-blue-200",
                                        _ => "badge badge-sm bg-gray-100 text-gray-700 border-gray-200",
                                    };
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{caller}</td>
                                            <td class="text-sm text-iiz-cyan">{phone}</td>
                                            <td class="text-sm text-gray-600">{source}</td>
                                            <td class="text-sm text-gray-600">{queue}</td>
                                            <td class="text-sm text-gray-600">{agent}</td>
                                            <td><span class=status_class>{status}</span></td>
                                            <td class="text-sm text-gray-600 text-right">{duration}</td>
                                            <td class="text-sm text-gray-600 text-right">{wait}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 19. Appointments
// ---------------------------------------------------------------------------

#[component]
pub fn AppointmentsPage() -> impl IntoView {
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
                // KPI cards
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

                // Chart: Weekly trend bars
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

                // Data table
                <div class="mx-4 mb-4 card bg-white border border-gray-200">
                    <div class="overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Date/Time"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Caller"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Phone"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Source"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Agent"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Type"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                    <th class="text-xs font-medium text-gray-500 uppercase">"Notes"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {[
                                    ("Feb 24, 9:00 AM", "John Smith", "(910) 555-0142", "Google", "Maria G.", "New", "Confirmed", "Initial consultation"),
                                    ("Feb 24, 9:30 AM", "Maria Lopez", "(910) 555-0198", "Facebook", "James W.", "Follow-up", "Completed", "Case review"),
                                    ("Feb 24, 10:00 AM", "Robert Chen", "(919) 555-0234", "Direct", "Sarah C.", "Consultation", "Confirmed", "Estate planning"),
                                    ("Feb 24, 10:30 AM", "Sarah Davis", "(910) 555-0312", "Google", "Emily D.", "New", "No-Show", "---"),
                                    ("Feb 24, 11:00 AM", "James Wilson", "(336) 555-0187", "TikTok", "Maria G.", "New", "Confirmed", "Personal injury consult"),
                                    ("Feb 24, 1:00 PM", "Emily Brown", "(704) 555-0265", "Radio", "James W.", "Follow-up", "Pending", "Document review"),
                                    ("Feb 24, 2:00 PM", "Carlos Reyes", "(910) 555-0421", "Google", "Sarah C.", "Consultation", "Confirmed", "Immigration case"),
                                    ("Feb 24, 3:30 PM", "Lisa Park", "(919) 555-0543", "Facebook", "Robert T.", "New", "Pending", "Family law inquiry"),
                                ].into_iter().map(|(datetime, caller, phone, source, agent, appt_type, status, notes)| {
                                    let type_class = match appt_type {
                                        "New" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                        "Follow-up" => "badge badge-sm bg-blue-100 text-blue-700 border-blue-200",
                                        _ => "badge badge-sm bg-purple-100 text-purple-700 border-purple-200",
                                    };
                                    let status_class = match status {
                                        "Confirmed" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                        "Completed" => "badge badge-sm bg-gray-100 text-gray-700 border-gray-200",
                                        "No-Show" => "badge badge-sm bg-red-100 text-red-700 border-red-200",
                                        _ => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                    };
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="text-sm font-medium">{datetime}</td>
                                            <td class="text-sm text-gray-700">{caller}</td>
                                            <td class="text-sm text-iiz-cyan">{phone}</td>
                                            <td class="text-sm text-gray-600">{source}</td>
                                            <td class="text-sm text-gray-600">{agent}</td>
                                            <td><span class=type_class>{appt_type}</span></td>
                                            <td><span class=status_class>{status}</span></td>
                                            <td class="text-sm text-gray-500 truncate max-w-[150px]">{notes}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 4 Connect pages (real-time dashboards)
// ---------------------------------------------------------------------------

#[component]
pub fn RealTimeAgentsPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Real-time Agents"</h1>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4 space-y-4">
                    // Status cards
                    <div class="grid grid-cols-4 gap-4">
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-3xl font-bold text-green-500">"5"</div>
                                <div class="text-sm text-gray-500">"Available"</div>
                                <div class="w-full h-1 bg-green-500 rounded mt-2"></div>
                            </div>
                        </div>
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-3xl font-bold text-blue-500">"3"</div>
                                <div class="text-sm text-gray-500">"On Call"</div>
                                <div class="w-full h-1 bg-blue-500 rounded mt-2"></div>
                            </div>
                        </div>
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-3xl font-bold text-orange-500">"2"</div>
                                <div class="text-sm text-gray-500">"After Call"</div>
                                <div class="w-full h-1 bg-orange-500 rounded mt-2"></div>
                            </div>
                        </div>
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-3xl font-bold text-gray-400">"8"</div>
                                <div class="text-sm text-gray-500">"Offline"</div>
                                <div class="w-full h-1 bg-gray-400 rounded mt-2"></div>
                            </div>
                        </div>
                    </div>

                    // Agent table
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Duration"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Queue"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {[
                                        ("Maria Garcia", "Available", "0:00", "Sales"),
                                        ("James Wilson", "On Call", "3:42", "Support"),
                                        ("Sarah Chen", "Available", "0:00", "Sales"),
                                        ("Mike Johnson", "After Call", "1:15", "Support"),
                                        ("Emily Davis", "On Call", "7:18", "Billing"),
                                        ("Robert Taylor", "Available", "0:00", "Sales"),
                                    ].into_iter().map(|(name, status, duration, queue)| {
                                        let status_class = match status {
                                            "Available" => "text-green-600",
                                            "On Call" => "text-blue-600",
                                            "After Call" => "text-orange-600",
                                            _ => "text-gray-400",
                                        };
                                        let dot_class = match status {
                                            "Available" => "bg-green-500",
                                            "On Call" => "bg-blue-500",
                                            "After Call" => "bg-orange-500",
                                            _ => "bg-gray-400",
                                        };
                                        let dot_cls = format!("w-2 h-2 rounded-full {}", dot_class);
                                        view! {
                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                <td class="text-sm font-medium">{name}</td>
                                                <td>
                                                    <span class="flex items-center gap-1.5">
                                                        <span class=dot_cls></span>
                                                        <span class={format!("text-sm {}", status_class)}>{status}</span>
                                                    </span>
                                                </td>
                                                <td class="text-sm text-gray-600">{duration}</td>
                                                <td class="text-sm text-gray-600">{queue}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
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
pub fn CoachingPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Coaching"</h1>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4 space-y-4">
                    // Info banner
                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <p class="text-sm text-gray-600">
                            "Use coaching tools to monitor live calls. "
                            <span class="font-medium">"Listen"</span>" to silently observe, "
                            <span class="font-medium">"Whisper"</span>" to speak only to the agent, or "
                            <span class="font-medium">"Barge"</span>" to join the conversation with both parties."
                        </p>
                    </div>

                    // Coaching table
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Agent"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Call Duration"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {[
                                        ("James Wilson", "On Call", "3:42"),
                                        ("Emily Davis", "On Call", "7:18"),
                                        ("Carlos Reyes", "On Call", "1:05"),
                                    ].into_iter().map(|(agent, status, duration)| {
                                        view! {
                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                <td class="text-sm font-medium">{agent}</td>
                                                <td>
                                                    <span class="flex items-center gap-1.5">
                                                        <span class="w-2 h-2 rounded-full bg-blue-500"></span>
                                                        <span class="text-sm text-blue-600">{status}</span>
                                                    </span>
                                                </td>
                                                <td class="text-sm text-gray-600">{duration}</td>
                                                <td>
                                                    <div class="flex items-center gap-2">
                                                        <button class="btn btn-xs btn-outline text-green-600 border-green-300 hover:bg-green-50">"Listen"</button>
                                                        <button class="btn btn-xs btn-outline text-orange-600 border-orange-300 hover:bg-orange-50">"Whisper"</button>
                                                        <button class="btn btn-xs btn-outline text-red-600 border-red-300 hover:bg-red-50">"Barge"</button>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
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
pub fn QueueReportPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Queue Report"</h1>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4 space-y-4">
                    // Stat cards
                    <div class="grid grid-cols-3 gap-4">
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-3xl font-bold text-orange-500">"2"</div>
                                <div class="text-sm text-gray-500">"Calls Waiting"</div>
                            </div>
                        </div>
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-3xl font-bold text-blue-500">"1:23"</div>
                                <div class="text-sm text-gray-500">"Avg Wait"</div>
                            </div>
                        </div>
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-3xl font-bold text-green-500">"94%"</div>
                                <div class="text-sm text-gray-500">"Service Level"</div>
                            </div>
                        </div>
                    </div>

                    // Queue table
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Queue"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Calls Waiting"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Agents"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Avg Wait"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Abandoned"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {[
                                        ("Sales", "1", "5", "0:45", "2"),
                                        ("Support", "1", "4", "1:52", "5"),
                                        ("Billing", "0", "3", "0:32", "1"),
                                        ("General", "0", "2", "2:10", "3"),
                                    ].into_iter().map(|(queue, waiting, agents, avg_wait, abandoned)| {
                                        view! {
                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                <td class="text-sm font-medium">{queue}</td>
                                                <td class="text-sm text-gray-600">{waiting}</td>
                                                <td class="text-sm text-gray-600">{agents}</td>
                                                <td class="text-sm text-gray-600">{avg_wait}</td>
                                                <td class="text-sm text-gray-600">{abandoned}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
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
pub fn AgentActivityPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Agent Activity"</h1>
                <div class="flex-1"></div>
                <div class="flex items-center gap-2">
                    <span class="text-xs text-gray-500">"Date Range:"</span>
                    <input type="date" class="input input-xs input-bordered" value="2026-02-17" />
                    <span class="text-xs text-gray-400">"to"</span>
                    <input type="date" class="input input-xs input-bordered" value="2026-02-24" />
                </div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Agent"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase text-right">"Calls Handled"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase text-right">"Avg Handle Time"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase text-right">"ACW Time"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase text-right">"Availability %"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {[
                                        ("Maria Garcia", "142", "3:15", "0:45", "92%"),
                                        ("James Wilson", "128", "4:02", "1:10", "87%"),
                                        ("Sarah Chen", "156", "2:48", "0:38", "95%"),
                                        ("Mike Johnson", "98", "5:22", "1:45", "78%"),
                                        ("Emily Davis", "134", "3:35", "0:52", "90%"),
                                        ("Robert Taylor", "115", "3:50", "1:02", "85%"),
                                    ].into_iter().map(|(agent, calls, avg_handle, acw, avail)| {
                                        view! {
                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                <td class="text-sm font-medium">{agent}</td>
                                                <td class="text-sm text-gray-600 text-right">{calls}</td>
                                                <td class="text-sm text-gray-600 text-right">{avg_handle}</td>
                                                <td class="text-sm text-gray-600 text-right">{acw}</td>
                                                <td class="text-sm text-gray-600 text-right">{avail}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 1 Usage page
// ---------------------------------------------------------------------------

#[component]
pub fn AgencyUsagePage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Agency Usage"</h1>
                <div class="flex-1"></div>
                <div class="flex items-center gap-2">
                    <span class="text-xs text-gray-500">"Date Range:"</span>
                    <input type="date" class="input input-xs input-bordered" value="2026-02-01" />
                    <span class="text-xs text-gray-400">"to"</span>
                    <input type="date" class="input input-xs input-bordered" value="2026-02-24" />
                </div>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4 space-y-4">
                    // Summary cards
                    <div class="grid grid-cols-3 gap-4">
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-2xl font-bold text-iiz-dark">"110,050"</div>
                                <div class="text-sm text-gray-500">"Total Calls"</div>
                            </div>
                        </div>
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-2xl font-bold text-iiz-dark">"316,131"</div>
                                <div class="text-sm text-gray-500">"Total Minutes"</div>
                            </div>
                        </div>
                        <div class="card bg-white border border-gray-200">
                            <div class="card-body p-4 text-center">
                                <div class="text-2xl font-bold text-iiz-dark">"45,230"</div>
                                <div class="text-sm text-gray-500">"Text Messages"</div>
                            </div>
                        </div>
                    </div>

                    // Usage breakdown table
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Usage Breakdown"</h2>
                            <div class="overflow-x-auto mt-4">
                                <table class="table table-sm w-full">
                                    <thead>
                                        <tr class="border-b border-gray-200">
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Account"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase text-right">"Calls"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase text-right">"Minutes"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase text-right">"Texts"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase text-right">"Cost"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {[
                                            ("Diener Law PA", "80,374", "230,856", "32,150", "$4,520.00"),
                                            ("Branch Office - Raleigh", "22,270", "63,847", "9,830", "$1,285.00"),
                                            ("Branch Office - Charlotte", "7,406", "21,428", "3,250", "$478.50"),
                                        ].into_iter().map(|(account, calls, minutes, texts, cost)| {
                                            view! {
                                                <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                    <td class="text-sm font-medium">{account}</td>
                                                    <td class="text-sm text-gray-600 text-right">{calls}</td>
                                                    <td class="text-sm text-gray-600 text-right">{minutes}</td>
                                                    <td class="text-sm text-gray-600 text-right">{texts}</td>
                                                    <td class="text-sm text-gray-600 text-right">{cost}</td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                    <tfoot>
                                        <tr class="border-t border-gray-300 font-semibold">
                                            <td class="text-sm">"Total"</td>
                                            <td class="text-sm text-right">"110,050"</td>
                                            <td class="text-sm text-right">"316,131"</td>
                                            <td class="text-sm text-right">"45,230"</td>
                                            <td class="text-sm text-right">"$6,283.50"</td>
                                        </tr>
                                    </tfoot>
                                </table>
                            </div>
                        </div>
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
                                    {[
                                        ("Weekly Call Summary", "Activity", "Every Monday", "Feb 17, 2026", "Jan 05, 2025"),
                                        ("Monthly ROI Dashboard", "ROI", "1st of month", "Feb 01, 2026", "Mar 12, 2025"),
                                        ("Daily Missed Calls Alert", "Missed Calls", "Daily at 6pm", "Feb 24, 2026", "Jun 20, 2025"),
                                    ].into_iter().map(|(name, report_type, schedule, last_run, created)| {
                                        view! {
                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                <td class="text-sm font-medium text-iiz-cyan">{name}</td>
                                                <td class="text-sm text-gray-600">{report_type}</td>
                                                <td class="text-sm text-gray-600">{schedule}</td>
                                                <td class="text-sm text-gray-600">{last_run}</td>
                                                <td class="text-sm text-gray-600">{created}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
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
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Type"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Recipients"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Trigger"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Active"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {[
                                        ("Missed Call Alert", "Email", "team@dienerlaw.net", "Missed call", true),
                                        ("High Volume Warning", "SMS", "+1 (910) 555-0101", "> 50 calls/hr", true),
                                        ("Daily Summary", "Email", "chris@dienerlaw.net", "Daily at 6pm", true),
                                        ("Weekend Calls", "Email + SMS", "on-call@dienerlaw.net", "Weekend call", false),
                                    ].into_iter().map(|(name, notif_type, recipients, trigger, active)| {
                                        view! {
                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                <td class="text-sm font-medium">{name}</td>
                                                <td class="text-sm text-gray-600">{notif_type}</td>
                                                <td class="text-sm text-gray-600">{recipients}</td>
                                                <td class="text-sm text-gray-600">{trigger}</td>
                                                <td>
                                                    {if active {
                                                        view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                    } else {
                                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-500 border-gray-200">"Inactive"</span> }.into_any()
                                                    }}
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
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
pub fn ScoringPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Scoring"</h1>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // Info banner
                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <p class="text-sm text-gray-600">
                            "Call scoring lets you automatically rate calls based on configurable criteria. Scores are displayed in the activity log and can be used in reports and notifications."
                        </p>
                    </div>

                    // Score Criteria card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Score Criteria"</h2>
                            <p class="text-sm text-gray-500 mt-1">"Configure the weight of each scoring criterion."</p>

                            <div class="space-y-5 mt-6">
                                {[
                                    ("Answer Rate", "40", "Percentage of calls answered within threshold"),
                                    ("Talk Time", "35", "Average talk time meets minimum duration"),
                                    ("Conversion", "25", "Call resulted in a positive outcome"),
                                ].into_iter().map(|(name, weight, desc)| {
                                    view! {
                                        <div class="space-y-2">
                                            <div class="flex items-center justify-between">
                                                <div>
                                                    <div class="text-sm font-medium text-gray-700">{name}</div>
                                                    <div class="text-xs text-gray-400">{desc}</div>
                                                </div>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-sm font-semibold text-iiz-dark">{weight}"%"</span>
                                                </div>
                                            </div>
                                            <input type="range" min="0" max="100" value=weight class="range range-sm range-primary w-full" />
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>

                            <div class="mt-6">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn TagsPage() -> impl IntoView {
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
                            <div class="space-y-3">
                                {[
                                    ("New Client", "bg-green-500", "1,245"),
                                    ("Returning Client", "bg-blue-500", "3,892"),
                                    ("Urgent", "bg-red-500", "456"),
                                    ("Follow-up Required", "bg-orange-500", "2,103"),
                                    ("Consultation Booked", "bg-purple-500", "892"),
                                    ("Spanish Speaker", "bg-yellow-500", "1,567"),
                                    ("After Hours", "bg-gray-500", "743"),
                                    ("VIP Client", "bg-pink-500", "189"),
                                ].into_iter().map(|(name, color, count)| {
                                    let dot_class = format!("w-3 h-3 rounded-full {}", color);
                                    view! {
                                        <div class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 cursor-pointer">
                                            <div class="flex items-center gap-3">
                                                <span class=dot_class></span>
                                                <span class="text-sm font-medium text-gray-700">{name}</span>
                                            </div>
                                            <div class="flex items-center gap-3">
                                                <span class="text-xs text-gray-400">{count}" uses"</span>
                                                <button class="btn btn-xs btn-ghost text-gray-400">
                                                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

