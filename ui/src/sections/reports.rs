use leptos::prelude::*;
use leptos_icons::Icon;

// ---------------------------------------------------------------------------
// Reports side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn ReportsSideNav() -> impl IntoView {
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
                <a href="/reports/activity" class="side-nav-item active">"Activity Reports"</a>
                <a href="/reports/roi" class="side-nav-item">"ROI Reports"</a>
                <a href="/reports/accuracy" class="side-nav-item">"Accuracy Reports"</a>
                <a href="/reports/map" class="side-nav-item">"Activity Map"</a>
                <a href="/reports/overview" class="side-nav-item">"Overview"</a>
                <a href="/reports/todays-missed" class="side-nav-item">"Today's Missed Calls"</a>
                <a href="/reports/positive-daily" class="side-nav-item">"Positive Daily Reports"</a>
                <a href="/reports/google-ca" class="side-nav-item">"Google CA Report"</a>
                <a href="/reports/saturday-calls" class="side-nav-item">"saturday calls"</a>
                <a href="/reports/daily-calls" class="side-nav-item">"Daily Calls"</a>
                <a href="/reports/weekly-missed" class="side-nav-item">"Weekly Missed Calls"</a>
                <a href="/reports/priming" class="side-nav-item">"Priming Calls"</a>
                <a href="/reports/missed" class="side-nav-item">"Missed Calls"</a>
                <a href="/reports/missed-daily-1st" class="side-nav-item">"Missed Calls Daily - 1st"</a>
                <a href="/reports/cs-daily-missed" class="side-nav-item">"CS Daily Missed Calls"</a>
                <a href="/reports/cs-daily-missed-2" class="side-nav-item">"CS Daily Missed 2.0"</a>
                <a href="/reports/priming-missed" class="side-nav-item">"Priming Missed Calls"</a>
                <a href="/reports/daily-collection" class="side-nav-item">"Daily Collection Calls"</a>
                <a href="/reports/power-bi" class="side-nav-item">"Power BI - Total Inbound"</a>
                <a href="/reports/realtime" class="side-nav-item">"real time"</a>
                <a href="/reports/appointments" class="side-nav-item">"Appointments"</a>
            </div>

            // Connect group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsPeopleFill /></span>
                    "Connect"
                </h3>
                <a href="/reports/realtime-agents" class="side-nav-item">"Real-time Agents"</a>
                <a href="/reports/coaching" class="side-nav-item">"Coaching"</a>
                <a href="/reports/queue-report" class="side-nav-item">"Queue Report"</a>
                <a href="/reports/agent-activity" class="side-nav-item">"Agent Activity"</a>
            </div>

            // Usage group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsSpeedometer /></span>
                    "Usage"
                </h3>
                <a href="/reports/agency-usage" class="side-nav-item">"Agency Usage"</a>
            </div>

            // Report Settings group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsGearFill /></span>
                    "Report Settings"
                </h3>
                <a href="/reports/custom-reports" class="side-nav-item">"Custom Reports"</a>
                <a href="/reports/notifications" class="side-nav-item">"Notifications"</a>
                <a href="/reports/scoring" class="side-nav-item">"Scoring"</a>
                <a href="/reports/tags" class="side-nav-item">"Tags"</a>
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
// Placeholder for report pages
// ---------------------------------------------------------------------------

#[component]
pub fn ReportsPlaceholderPage(
    #[prop(into)] title: String,
    #[prop(into, optional)] description: Option<String>,
) -> impl IntoView {
    let desc = description.unwrap_or_else(|| format!("The {} report will display here with charts and data tables.", title));
    view! {
        <div class="flex flex-col h-full">
            <div class="flex-1 flex items-center justify-center bg-iiz-gray-bg">
                <div class="text-center max-w-md">
                    <div class="w-16 h-16 rounded-full bg-iiz-cyan-light flex items-center justify-center mx-auto mb-4">
                        <span class="w-8 h-8 inline-flex text-iiz-cyan"><Icon icon=icondata::BsBarChartFill /></span>
                    </div>
                    <h2 class="text-xl font-semibold text-gray-700">{title}</h2>
                    <p class="text-sm text-gray-500 mt-2">{desc}</p>
                </div>
            </div>
        </div>
    }
}
