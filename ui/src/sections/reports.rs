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
// ReusableReportPage - shared layout for Analytics sub-reports
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct SimpleSourceRow {
    name: &'static str,
    badge_pct: &'static str,
    badge_color: &'static str,
    total: &'static str,
    period_unique: &'static str,
    avg_talk: &'static str,
}

fn simple_source_rows() -> Vec<SimpleSourceRow> {
    vec![
        SimpleSourceRow { name: "Google Organic", badge_pct: "73%", badge_color: "bg-green-500", total: "80,374", period_unique: "19,988", avg_talk: "2:22" },
        SimpleSourceRow { name: "Customer Service", badge_pct: "20%", badge_color: "bg-orange-500", total: "22,270", period_unique: "6,809", avg_talk: "2:05" },
        SimpleSourceRow { name: "TikTok Organic", badge_pct: "2%", badge_color: "bg-red-500", total: "2,526", period_unique: "1,746", avg_talk: "2:10" },
        SimpleSourceRow { name: "Facebook Paid", badge_pct: "3%", badge_color: "bg-red-500", total: "1,942", period_unique: "1,178", avg_talk: "2:15" },
        SimpleSourceRow { name: "Direct", badge_pct: "1%", badge_color: "bg-green-500", total: "625", period_unique: "326", avg_talk: "2:30" },
    ]
}

#[component]
pub fn ReusableReportPage(#[prop(into)] title: String, #[prop(into)] subtitle: String) -> impl IntoView {
    let rows = simple_source_rows();

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

            // Title + subtitle row
            <div class="bg-white border-b border-gray-200 px-4 py-3 flex-shrink-0">
                <h2 class="text-lg font-semibold text-iiz-dark">{title}</h2>
                <p class="text-xs text-gray-500">{subtitle}</p>
            </div>

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
                    <div class="h-[100px] bg-gray-50 rounded-lg border border-gray-200 flex items-center justify-center gap-2">
                        <span class="w-6 h-6 inline-flex text-gray-400"><Icon icon=icondata::BsBarChartFill /></span>
                        <span class="text-sm text-gray-400">"Chart data loading..."</span>
                    </div>
                </div>

                // Data table
                <div class="overflow-x-auto">
                    // Column headers
                    <div class="grid grid-cols-[200px_100px_100px_100px] gap-1 px-4 py-2 bg-gray-50 border-b border-gray-200 min-w-max">
                        <div class="col-header">"Source"</div>
                        <div class="col-header text-right">"Total"</div>
                        <div class="col-header text-right">"Period Unique"</div>
                        <div class="col-header text-right">"Avg Talk Time"</div>
                    </div>

                    // Source rows
                    {rows.into_iter().map(|s| {
                        let badge_class = format!("inline-block w-10 text-center text-[10px] text-white rounded px-1 py-0.5 {}", s.badge_color);
                        view! {
                            <div class="activity-row grid grid-cols-[200px_100px_100px_100px] gap-1 px-4 py-2 items-center border-b border-gray-100 min-w-max">
                                <div class="flex items-center gap-2">
                                    <span class=badge_class>{s.badge_pct}</span>
                                    <span class="text-sm truncate">{s.name}</span>
                                </div>
                                <div class="text-right text-sm">{s.total}</div>
                                <div class="text-right text-sm">{s.period_unique}</div>
                                <div class="text-right text-sm">{s.avg_talk}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}

                    // Totals row
                    <div class="grid grid-cols-[200px_100px_100px_100px] gap-1 px-4 py-2 bg-gray-50 border-t border-gray-300 font-semibold min-w-max">
                        <div class="text-sm">"Total"</div>
                        <div class="text-right text-sm">"107,737"</div>
                        <div class="text-right text-sm">"30,047"</div>
                        <div class="text-right text-sm">"2:16"</div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// 17 Analytics pages (reuse ReusableReportPage)
// ---------------------------------------------------------------------------

#[component]
pub fn ROIReportPage() -> impl IntoView {
    view! { <ReusableReportPage title="ROI Reports" subtitle="Revenue attribution per tracking source" /> }
}

#[component]
pub fn AccuracyReportPage() -> impl IntoView {
    view! { <ReusableReportPage title="Accuracy Reports" subtitle="Call scoring accuracy and quality metrics" /> }
}

#[component]
pub fn ActivityMapPage() -> impl IntoView {
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
                <p class="text-xs text-gray-500">"Geographic visualization of call origins"</p>
            </div>

            // Map placeholder
            <div class="flex-1 flex items-center justify-center bg-iiz-gray-bg">
                <div class="text-center">
                    <div class="w-64 h-48 bg-gray-100 rounded-lg border border-gray-200 flex items-center justify-center mx-auto">
                        <div class="text-center">
                            <span class="w-12 h-12 inline-flex text-gray-300 mx-auto"><Icon icon=icondata::BsGeoAltFill /></span>
                            <p class="text-sm text-gray-400 mt-2">"Map loading..."</p>
                            <p class="text-xs text-gray-300 mt-1">"Geographic call distribution"</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn OverviewPage() -> impl IntoView {
    view! { <ReusableReportPage title="Overview" subtitle="Call distribution overview" /> }
}

#[component]
pub fn TodaysMissedPage() -> impl IntoView {
    view! { <ReusableReportPage title="Today's Missed Calls" subtitle="Real-time missed calls for today" /> }
}

#[component]
pub fn PositiveDailyPage() -> impl IntoView {
    view! { <ReusableReportPage title="Positive Daily Reports" subtitle="Calls meeting positive outcome criteria" /> }
}

#[component]
pub fn GoogleCAPage() -> impl IntoView {
    view! { <ReusableReportPage title="Google CA Report" subtitle="Google call analytics integration" /> }
}

#[component]
pub fn SaturdayCallsPage() -> impl IntoView {
    view! { <ReusableReportPage title="saturday calls" subtitle="Weekend call volume" /> }
}

#[component]
pub fn DailyCallsPage() -> impl IntoView {
    view! { <ReusableReportPage title="Daily Calls" subtitle="Day-by-day call volume" /> }
}

#[component]
pub fn WeeklyMissedPage() -> impl IntoView {
    view! { <ReusableReportPage title="Weekly Missed Calls" subtitle="Aggregated weekly missed call trends" /> }
}

#[component]
pub fn PrimingCallsPage() -> impl IntoView {
    view! { <ReusableReportPage title="Priming Calls" subtitle="Pre-qualification call tracking" /> }
}

#[component]
pub fn MissedCallsPage() -> impl IntoView {
    view! { <ReusableReportPage title="Missed Calls" subtitle="Missed call analysis" /> }
}

#[component]
pub fn MissedDaily1stPage() -> impl IntoView {
    view! { <ReusableReportPage title="Missed Calls Daily - First Contact" subtitle="First-contact missed calls" /> }
}

#[component]
pub fn CSDailyMissedPage() -> impl IntoView {
    view! { <ReusableReportPage title="CS Daily Missed Calls" subtitle="Customer service queue missed calls" /> }
}

#[component]
pub fn CSDailyMissed2Page() -> impl IntoView {
    view! { <ReusableReportPage title="CS Daily Missed 2.0" subtitle="Customer service v2" /> }
}

#[component]
pub fn PrimingMissedPage() -> impl IntoView {
    view! { <ReusableReportPage title="Priming Missed Calls" subtitle="Priming missed call tracking" /> }
}

#[component]
pub fn DailyCollectionPage() -> impl IntoView {
    view! { <ReusableReportPage title="Daily Collection Calls" subtitle="Collections department tracking" /> }
}

// ---------------------------------------------------------------------------
// 3 more Analytics pages (simpler, also ReusableReportPage)
// ---------------------------------------------------------------------------

#[component]
pub fn PowerBIPage() -> impl IntoView {
    view! { <ReusableReportPage title="Power BI - Total Inbound" subtitle="External BI integration" /> }
}

#[component]
pub fn RealTimePage() -> impl IntoView {
    view! { <ReusableReportPage title="real time" subtitle="Live call dashboard" /> }
}

#[component]
pub fn AppointmentsPage() -> impl IntoView {
    view! { <ReusableReportPage title="Appointments" subtitle="Appointment scheduling conversions" /> }
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

// ---------------------------------------------------------------------------
// Placeholder for report pages (kept for compatibility)
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
