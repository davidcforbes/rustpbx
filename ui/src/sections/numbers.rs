use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

// ---------------------------------------------------------------------------
// Numbers side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn NumbersSideNav() -> impl IntoView {
    let location = use_location();
    let active = |href: &'static str| {
        move || {
            if location.pathname.get() == href { "side-nav-item active" } else { "side-nav-item" }
        }
    };

    view! {
        <div class="px-4 pt-4 pb-2">
            <div class="flex items-center gap-2 text-iiz-cyan">
                <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsGrid3x3GapFill /></span>
                <span class="text-lg font-light">"Numbers"</span>
            </div>
        </div>

        <nav class="px-2 pb-4">
            // Management group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsTelephoneFill /></span>
                    "Management"
                </h3>
                <a href="/numbers/buy" class=active("/numbers/buy")>"Buy Numbers"</a>
                <a href="/numbers/tracking" class=active("/numbers/tracking")>"Tracking Numbers"</a>
                <a href="/numbers/receiving" class=active("/numbers/receiving")>"Receiving Numbers"</a>
                <a href="/numbers/text" class=active("/numbers/text")>"Text Numbers"</a>
                <a href="/numbers/port" class=active("/numbers/port")>"Port Numbers"</a>
                <a href="/numbers/call-settings" class=active("/numbers/call-settings")>"Call Settings"</a>
            </div>

            // Dynamic Numbers group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsArrowRepeat /></span>
                    "Dynamic Numbers"
                </h3>
                <a href="/numbers/pools" class=active("/numbers/pools")>"Number Pools"</a>
                <a href="/numbers/targets" class=active("/numbers/targets")>"Target Numbers"</a>
                <a href="/numbers/sources" class=active("/numbers/sources")>"Tracking Sources"</a>
                <a href="/numbers/code" class=active("/numbers/code")>"Tracking Code"</a>
            </div>
        </nav>
    }
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct TrackingNumber {
    number: &'static str,
    source: &'static str,
    routing: &'static str,
    routing_type: &'static str,
    text_enabled: bool,
    target: &'static str,
    config: &'static str,
    billing_date: &'static str,
    active: bool,
    number_type: &'static str,
}

#[derive(Clone, Debug)]
struct ReceivingNumber {
    number: &'static str,
    description: &'static str,
    tracking_count: u32,
    total_calls: &'static str,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct CallSetting {
    name: &'static str,
    is_default: bool,
    greeting: bool,
    whisper: bool,
    inbound_rec: bool,
    outbound_rec: bool,
    transcribe: bool,
    caller_id: bool,
    enhanced_id: bool,
    override_id: bool,
    spam: bool,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct TrackingSource {
    name: &'static str,
    source_type: &'static str,
    position: u32,
    numbers: &'static str,
    last_touch: bool,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct BuyNumber {
    number: &'static str,
    e164: &'static str,
    rate_center: &'static str,
    features: Vec<&'static str>,
    monthly: &'static str,
}

// ---------------------------------------------------------------------------
// Mock data
// ---------------------------------------------------------------------------

fn mock_tracking_numbers() -> Vec<TrackingNumber> {
    vec![
        TrackingNumber { number: "(910) 991-0047", source: "Test source", routing: "SYSTANGO TESTING", routing_type: "Queue", text_enabled: true, target: "Account Level", config: "Inbound Rec, Outbound Rec, Caller ID", billing_date: "2026-03-23", active: true, number_type: "Offsite Static" },
        TrackingNumber { number: "(980) 553-2289", source: "Facebook Paid", routing: "Check if New Lead or Current Client", routing_type: "Smart Router", text_enabled: true, target: "(855) 563-5818", config: "Inbound Rec, Outbound Rec, Caller ID", billing_date: "2026-03-13", active: true, number_type: "Onsite Dynamic" },
        TrackingNumber { number: "(855) 614-1888", source: "Customer Service Line", routing: "Rescue Team", routing_type: "Queue", text_enabled: true, target: "Account Level", config: "Inbound Rec, Outbound Rec, Caller ID", billing_date: "2026-03-01", active: true, number_type: "Offsite Static" },
        TrackingNumber { number: "(919) 290-4449", source: "WhatsApp", routing: "Customer Service Queue (Official)", routing_type: "Queue", text_enabled: true, target: "Account Level", config: "Inbound Rec, Outbound Rec, Caller ID", billing_date: "2026-03-01", active: true, number_type: "Offsite Static" },
        TrackingNumber { number: "(832) 558-3313", source: "Facebook West Houston Office", routing: "Check if New Lead or...", routing_type: "Smart Router", text_enabled: true, target: "(855) 563-5818", config: "Inbound Rec, Outbound Rec, Caller ID", billing_date: "2026-03-01", active: true, number_type: "Onsite Dynamic" },
        TrackingNumber { number: "(276) 201-0001", source: "Google Organic", routing: "Main IVR", routing_type: "Queue", text_enabled: false, target: "Account Level", config: "Inbound Rec, Caller ID", billing_date: "2026-03-15", active: true, number_type: "Offsite Static" },
        TrackingNumber { number: "(276) 201-0002", source: "YouTube Ads", routing: "Sales Queue", routing_type: "Queue", text_enabled: true, target: "(888) 361-3349", config: "Inbound Rec, Outbound Rec, Caller ID", billing_date: "2026-03-15", active: false, number_type: "Offsite Static" },
        TrackingNumber { number: "(276) 201-0003", source: "TikTok Organic", routing: "CS Smart Router", routing_type: "Smart Router", text_enabled: true, target: "(855) 563-5818", config: "Inbound Rec, Outbound Rec, Transcribe", billing_date: "2026-03-15", active: true, number_type: "Onsite Dynamic" },
    ]
}

fn mock_receiving_numbers() -> Vec<ReceivingNumber> {
    vec![
        ReceivingNumber { number: "(252) 235-4100", description: "CASE MANAGERS QUEUE", tracking_count: 2, total_calls: "182", updated: "2025-12-15", created: "2024-06-10" },
        ReceivingNumber { number: "(844) 707-4320", description: "RMS-FKM-PAYMENTS", tracking_count: 1, total_calls: "0", updated: "2025-11-20", created: "2024-08-05" },
        ReceivingNumber { number: "(252) 351-2397", description: "CS QUEUE", tracking_count: 5, total_calls: "4,119", updated: "2025-12-20", created: "2024-03-15" },
        ReceivingNumber { number: "(888) 361-3349", description: "SALES QUEUE", tracking_count: 8, total_calls: "13,228", updated: "2025-12-22", created: "2023-11-01" },
        ReceivingNumber { number: "(888) 399-8387", description: "ANSWERHERO", tracking_count: 12, total_calls: "73,549", updated: "2025-12-22", created: "2023-05-20" },
        ReceivingNumber { number: "(855) 563-5818", description: "MAIN CS LINE", tracking_count: 15, total_calls: "50,417", updated: "2025-12-22", created: "2023-01-15" },
    ]
}

fn mock_call_settings() -> Vec<CallSetting> {
    vec![
        CallSetting { name: "Account Level", is_default: true, greeting: true, whisper: false, inbound_rec: true, outbound_rec: true, transcribe: false, caller_id: true, enhanced_id: false, override_id: false, spam: false, updated: "2025-12-01", created: "2023-01-15" },
        CallSetting { name: "No Call Recording", is_default: false, greeting: false, whisper: false, inbound_rec: true, outbound_rec: false, transcribe: false, caller_id: true, enhanced_id: false, override_id: false, spam: false, updated: "2025-10-15", created: "2024-06-01" },
    ]
}

fn mock_tracking_sources() -> Vec<TrackingSource> {
    vec![
        TrackingSource { name: "Direct", source_type: "Onsite Dynamic", position: 1, numbers: "13 Assigned", last_touch: false, updated: "2025-12-20", created: "2023-01-15" },
        TrackingSource { name: "Woosender", source_type: "Offsite Static", position: 2, numbers: "3 Assigned", last_touch: true, updated: "2025-12-18", created: "2023-03-10" },
        TrackingSource { name: "Email Marketing", source_type: "Offsite Static", position: 3, numbers: "2 Assigned", last_touch: false, updated: "2025-12-15", created: "2023-05-20" },
        TrackingSource { name: "Facebook Paid", source_type: "Offsite Static", position: 4, numbers: "5 Assigned", last_touch: false, updated: "2025-12-10", created: "2023-06-01" },
        TrackingSource { name: "Google Organic", source_type: "Offsite Static", position: 5, numbers: "8 Assigned", last_touch: false, updated: "2025-12-10", created: "2023-06-15" },
        TrackingSource { name: "YouTube Ads", source_type: "Offsite Static", position: 6, numbers: "2 Assigned", last_touch: false, updated: "2025-11-30", created: "2024-01-10" },
        TrackingSource { name: "TikTok Organic", source_type: "Offsite Static", position: 7, numbers: "4 Assigned", last_touch: true, updated: "2025-11-25", created: "2024-03-01" },
        TrackingSource { name: "Google Paid", source_type: "Offsite Static", position: 8, numbers: "6 Assigned", last_touch: false, updated: "2025-11-20", created: "2024-04-15" },
        TrackingSource { name: "Bing Organic", source_type: "Offsite Static", position: 9, numbers: "1 Assigned", last_touch: false, updated: "2025-11-15", created: "2024-06-01" },
        TrackingSource { name: "Referral", source_type: "Offsite Static", position: 10, numbers: "3 Assigned", last_touch: true, updated: "2025-11-10", created: "2024-07-20" },
    ]
}

fn mock_buy_numbers() -> Vec<BuyNumber> {
    vec![
        BuyNumber { number: "(276) 201-0001", e164: "+12762010001", rate_center: "GALAX, VA", features: vec!["SMS", "MMS", "Fax"], monthly: "$1.26" },
        BuyNumber { number: "(276) 201-0002", e164: "+12762010002", rate_center: "GALAX, VA", features: vec!["SMS", "MMS"], monthly: "$1.26" },
        BuyNumber { number: "(276) 201-0003", e164: "+12762010003", rate_center: "GALAX, VA", features: vec!["SMS", "MMS", "HIPAA"], monthly: "$1.26" },
        BuyNumber { number: "(276) 201-0004", e164: "+12762010004", rate_center: "WYTHEVILLE, VA", features: vec!["SMS", "MMS", "Fax", "e911"], monthly: "$1.26" },
        BuyNumber { number: "(276) 201-0005", e164: "+12762010005", rate_center: "PULASKI, VA", features: vec!["SMS"], monthly: "$1.26" },
        BuyNumber { number: "(276) 201-0006", e164: "+12762010006", rate_center: "RADFORD, VA", features: vec!["SMS", "MMS", "Fax"], monthly: "$1.26" },
        BuyNumber { number: "(276) 201-0007", e164: "+12762010007", rate_center: "MARION, VA", features: vec!["SMS", "MMS"], monthly: "$1.26" },
        BuyNumber { number: "(276) 201-0008", e164: "+12762010008", rate_center: "ABINGDON, VA", features: vec!["SMS", "MMS", "HIPAA", "Fax", "e911"], monthly: "$1.26" },
    ]
}

// ---------------------------------------------------------------------------
// Tracking Numbers page (main page)
// ---------------------------------------------------------------------------

#[component]
pub fn TrackingNumbersPage() -> impl IntoView {
    let numbers = mock_tracking_numbers();

    view! {
        <div class="flex flex-col h-full">
            // Top bar
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Released Numbers"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Number Log"</a>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">"254 Tracking Numbers"</span>
                <div class="join">
                    <input type="text" placeholder="Search numbers..." class="input input-sm input-bordered join-item w-48" />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "Buy Numbers"
                </button>
            </header>

            // Table headers
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_120px_140px_180px_80px_110px_160px_100px_80px_80px_100px] gap-1 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Number"</div>
                    <div class="col-header">"Source"</div>
                    <div class="col-header">"Call Routing"</div>
                    <div class="col-header">"Text"</div>
                    <div class="col-header">"Target"</div>
                    <div class="col-header">"Config"</div>
                    <div class="col-header">"Billing"</div>
                    <div class="col-header">"Active"</div>
                    <div class="col-header">"Actions"</div>
                    <div class="col-header">"Type"</div>
                </div>
            </div>

            // Table rows
            <div class="flex-1 overflow-y-auto">
                {numbers.into_iter().map(|n| view! { <TrackingNumberRow number=n /> }).collect::<Vec<_>>()}
            </div>

            // Pagination bar
            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-8 of 254"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span></button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost">"3"</button>
                    <span class="text-xs text-gray-400">"..."</span>
                    <button class="btn btn-xs btn-ghost">"26"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                </div>
                <select class="select select-xs select-bordered ml-2">
                    <option selected>"10"</option>
                    <option>"25"</option>
                    <option>"50"</option>
                    <option>"100"</option>
                </select>
            </div>
        </div>
    }
}

#[component]
fn TrackingNumberRow(number: TrackingNumber) -> impl IntoView {
    let type_class = if number.number_type == "Onsite Dynamic" {
        "badge badge-sm bg-purple-100 text-purple-700 border-none"
    } else {
        "badge badge-sm bg-gray-100 text-gray-600 border-none"
    };
    let routing_badge = if number.routing_type == "Smart Router" {
        "badge badge-xs bg-blue-100 text-blue-700 border-none"
    } else {
        "badge badge-xs bg-green-100 text-green-700 border-none"
    };
    let active_class = if number.active { "text-green-600 text-xs" } else { "text-gray-400 text-xs" };
    let active_text = if number.active { "Yes" } else { "No" };

    view! {
        <div class="activity-row grid grid-cols-[32px_120px_140px_180px_80px_110px_160px_100px_80px_80px_100px] gap-1 px-4 py-2.5 items-center cursor-pointer">
            <button class="btn btn-xs btn-ghost text-gray-400">
                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
            </button>
            <div>
                <div class="text-sm font-medium text-iiz-blue-link">{number.number}</div>
            </div>
            <div class="text-xs text-gray-600 truncate">{number.source}</div>
            <div>
                <div class="text-xs truncate">{number.routing}</div>
                <span class=routing_badge>{number.routing_type}</span>
            </div>
            <div>
                {if number.text_enabled {
                    view! { <span class="w-4 h-4 inline-flex text-green-500"><Icon icon=icondata::BsShieldCheck /></span> }.into_any()
                } else {
                    view! { <span class="text-xs text-gray-400">"-"</span> }.into_any()
                }}
            </div>
            <div class="text-xs text-gray-600 truncate">{number.target}</div>
            <div class="text-xs text-gray-500 truncate">{number.config}</div>
            <div class="text-xs text-gray-500">{number.billing_date}</div>
            <div class=active_class>{active_text}</div>
            <div class="flex items-center gap-0.5">
                <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsCalendar /></span></button>
                <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsEnvelope /></span></button>
                <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsTelephone /></span></button>
                <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsClipboard /></span></button>
            </div>
            <div><span class=type_class>{number.number_type}</span></div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Buy Numbers page
// ---------------------------------------------------------------------------

#[component]
pub fn BuyNumbersPage() -> impl IntoView {
    let numbers = mock_buy_numbers();

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <select class="select select-sm select-bordered">
                    <option selected>"US +1"</option>
                    <option>"CA +1"</option>
                    <option>"GB +44"</option>
                </select>
                <button class="btn btn-sm btn-outline border-iiz-cyan text-iiz-cyan">"Regulations"</button>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm btn-outline">"New Number Pool"</button>
                <button class="btn btn-sm btn-outline">"Buy Bulk"</button>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Request a Number..."</button>
            </header>

            // Search tabs
            <div class="bg-white border-b border-gray-200 px-4">
                <div class="flex gap-4 text-sm">
                    <button class="py-2 border-b-2 border-iiz-cyan text-iiz-cyan font-medium">"Local Number"</button>
                    <button class="py-2 text-gray-500 hover:text-gray-700">"Toll-Free"</button>
                    <button class="py-2 text-gray-500 hover:text-gray-700">"Address"</button>
                    <button class="py-2 text-gray-500 hover:text-gray-700">"Near Number"</button>
                </div>
            </div>

            // Search filters
            <div class="bg-white border-b border-gray-200 px-4 py-3 flex items-center gap-3">
                <select class="select select-sm select-bordered">
                    <option>"Any"</option>
                    <option>"Area Code"</option>
                    <option>"City"</option>
                    <option>"State"</option>
                </select>
                <select class="select select-sm select-bordered">
                    <option>"contains"</option>
                    <option>"starts with"</option>
                    <option>"ends with"</option>
                </select>
                <input type="text" placeholder="276" class="input input-sm input-bordered w-32" />
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    "Search"
                </button>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Additional Filters"</a>
            </div>

            <div class="flex flex-1 overflow-hidden">
                // Results table
                <div class="flex-1 overflow-y-auto">
                    // Column headers
                    <div class="grid grid-cols-[1fr_1fr_100px_80px] gap-2 px-4 py-2 bg-gray-50 border-b border-gray-200">
                        <div class="col-header">"Phone Number"</div>
                        <div class="col-header">"Rate Center & Features"</div>
                        <div class="col-header">"Monthly Fee"</div>
                        <div class="col-header"></div>
                    </div>

                    {numbers.into_iter().map(|n| {
                        let feats = n.features.clone();
                        view! {
                            <div class="activity-row grid grid-cols-[1fr_1fr_100px_80px] gap-2 px-4 py-3 items-center">
                                <div>
                                    <div class="text-sm font-medium">{n.number}</div>
                                    <div class="text-xs text-gray-400">{n.e164}</div>
                                </div>
                                <div>
                                    <span class="text-xs text-gray-600">{n.rate_center}</span>
                                    <div class="flex flex-wrap gap-1 mt-1">
                                        {feats.iter().map(|f| {
                                            let badge_class = match *f {
                                                "HIPAA" => "badge badge-xs bg-green-100 text-green-700 border-none",
                                                "e911" => "badge badge-xs bg-orange-100 text-orange-700 border-none",
                                                _ => "badge badge-xs bg-gray-100 text-gray-600 border-none",
                                            };
                                            view! { <span class=badge_class>{*f}</span> }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                                <div class="text-sm">{n.monthly}</div>
                                <div>
                                    <button class="btn btn-xs bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPlus /></span>
                                        "Add"
                                    </button>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                // Cart sidebar
                <div class="w-56 border-l border-gray-200 bg-white p-4 flex-shrink-0">
                    <h3 class="text-sm font-semibold text-gray-700 mb-4">"Cart"</h3>
                    <div class="text-center py-6">
                        <div class="text-2xl font-bold text-gray-300">"0"</div>
                        <div class="text-xs text-gray-400">"numbers"</div>
                    </div>
                    <div class="border-t border-gray-200 pt-3">
                        <div class="flex justify-between text-sm">
                            <span class="text-gray-500">"Monthly"</span>
                            <span class="font-medium">"$0.00"</span>
                        </div>
                    </div>
                    <button class="btn btn-sm w-full mt-4 bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none" disabled>"Purchase"</button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Receiving Numbers page
// ---------------------------------------------------------------------------

#[component]
pub fn ReceivingNumbersPage() -> impl IntoView {
    let numbers = mock_receiving_numbers();

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">"6 Receiving Numbers"</span>
                <div class="join">
                    <input type="text" placeholder="Search..." class="input input-sm input-bordered join-item w-40" />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Receiving Number"</button>
            </header>

            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_160px_120px_40px_80px_100px_100px_100px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Number"</div>
                    <div class="col-header">"Tracking #s"</div>
                    <div class="col-header"></div>
                    <div class="col-header">"Geo"</div>
                    <div class="col-header">"Total Calls"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            <div class="flex-1 overflow-y-auto">
                {numbers.into_iter().map(|n| {
                    let tracking_text = format!("{} Numbers", n.tracking_count);
                    view! {
                        <div class="activity-row grid grid-cols-[32px_160px_120px_40px_80px_100px_100px_100px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <button class="btn btn-xs btn-ghost text-gray-400">
                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                            </button>
                            <div>
                                <div class="text-sm font-medium text-iiz-blue-link">{n.number}</div>
                                <div class="text-xs text-gray-500">{n.description}</div>
                            </div>
                            <div>
                                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">{tracking_text}</a>
                            </div>
                            <div>
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsVolumeUpFill /></span>
                                </button>
                            </div>
                            <div class="text-xs text-gray-500">"-"</div>
                            <div class="text-sm">{n.total_calls}</div>
                            <div class="text-xs text-gray-500">{n.updated}</div>
                            <div class="text-xs text-gray-500">{n.created}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-6 of 6"</span>
                <div class="flex-1"></div>
                <select class="select select-xs select-bordered">
                    <option selected>"10"</option>
                    <option>"25"</option>
                    <option>"50"</option>
                </select>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Call Settings page
// ---------------------------------------------------------------------------

#[component]
pub fn CallSettingsPage() -> impl IntoView {
    let settings = mock_call_settings();

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">"2 Call Settings"</span>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Call Settings"</button>
            </header>

            <div class="sticky top-0 bg-white border-b border-gray-200 z-10 overflow-x-auto">
                <div class="grid grid-cols-[32px_140px_60px_60px_60px_60px_60px_60px_60px_60px_60px_60px_90px_90px] gap-1 px-4 py-2 items-center min-w-max">
                    <div class="col-header"></div>
                    <div class="col-header">"Name"</div>
                    <div class="col-header text-center">"Default"</div>
                    <div class="col-header text-center">"Greeting"</div>
                    <div class="col-header text-center">"Whisper"</div>
                    <div class="col-header text-center">"In Rec"</div>
                    <div class="col-header text-center">"Out Rec"</div>
                    <div class="col-header text-center">"Transcr"</div>
                    <div class="col-header text-center">"Caller ID"</div>
                    <div class="col-header text-center">"Enh ID"</div>
                    <div class="col-header text-center">"Ovr ID"</div>
                    <div class="col-header text-center">"Spam"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            <div class="flex-1 overflow-y-auto overflow-x-auto">
                {settings.into_iter().map(|s| {
                    let check = |enabled: bool| {
                        if enabled {
                            view! { <span class="text-green-500 text-sm">"✓"</span> }.into_any()
                        } else {
                            view! { <span class="text-gray-300 text-sm">"-"</span> }.into_any()
                        }
                    };
                    view! {
                        <div class="activity-row grid grid-cols-[32px_140px_60px_60px_60px_60px_60px_60px_60px_60px_60px_60px_90px_90px] gap-1 px-4 py-2.5 items-center cursor-pointer min-w-max">
                            <button class="btn btn-xs btn-ghost text-gray-400">
                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                            </button>
                            <div class="text-sm font-medium">{s.name}</div>
                            <div class="text-center">{check(s.is_default)}</div>
                            <div class="text-center">{check(s.greeting)}</div>
                            <div class="text-center">{check(s.whisper)}</div>
                            <div class="text-center">{check(s.inbound_rec)}</div>
                            <div class="text-center">{check(s.outbound_rec)}</div>
                            <div class="text-center">{check(s.transcribe)}</div>
                            <div class="text-center">{check(s.caller_id)}</div>
                            <div class="text-center">{check(s.enhanced_id)}</div>
                            <div class="text-center">{check(s.override_id)}</div>
                            <div class="text-center">{check(s.spam)}</div>
                            <div class="text-xs text-gray-500">{s.updated}</div>
                            <div class="text-xs text-gray-500">{s.created}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Tracking Sources page
// ---------------------------------------------------------------------------

#[component]
pub fn TrackingSourcesPage() -> impl IntoView {
    let sources = mock_tracking_sources();

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Export"</a>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">"46 Tracking Sources"</span>
                <div class="join">
                    <input type="text" placeholder="Search..." class="input input-sm input-bordered join-item w-40" />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Tracking Source"</button>
            </header>

            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_1fr_120px_60px_100px_70px_90px_90px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Source Name"</div>
                    <div class="col-header">"Type"</div>
                    <div class="col-header">"Pos"</div>
                    <div class="col-header">"Numbers"</div>
                    <div class="col-header">"Last Touch"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            <div class="flex-1 overflow-y-auto">
                {sources.into_iter().map(|s| {
                    let type_class = if s.source_type == "Onsite Dynamic" {
                        "badge badge-sm bg-purple-100 text-purple-700 border-none"
                    } else {
                        "badge badge-sm bg-gray-100 text-gray-600 border-none"
                    };
                    view! {
                        <div class="activity-row grid grid-cols-[32px_1fr_120px_60px_100px_70px_90px_90px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <button class="btn btn-xs btn-ghost text-gray-400">
                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                            </button>
                            <div class="text-sm font-medium">{s.name}</div>
                            <div><span class=type_class>{s.source_type}</span></div>
                            <div class="text-sm text-center">{s.position}</div>
                            <div><a class="text-xs text-iiz-cyan hover:underline cursor-pointer">{s.numbers}</a></div>
                            <div class="text-center">
                                {if s.last_touch {
                                    view! { <span class="text-green-500 text-sm">"✓"</span> }.into_any()
                                } else {
                                    view! { <span class="text-gray-300 text-sm">"-"</span> }.into_any()
                                }}
                            </div>
                            <div class="text-xs text-gray-500">{s.updated}</div>
                            <div class="text-xs text-gray-500">{s.created}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-10 of 46"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span></button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost">"3"</button>
                    <button class="btn btn-xs btn-ghost">"4"</button>
                    <button class="btn btn-xs btn-ghost">"5"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                </div>
                <select class="select select-xs select-bordered ml-2">
                    <option selected>"10"</option>
                    <option>"25"</option>
                    <option>"50"</option>
                </select>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Additional data types for new pages
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct TextNumber {
    number: &'static str,
    e164: &'static str,
}

#[derive(Clone, Debug)]
struct TargetNumber {
    number: &'static str,
    description: &'static str,
    target_type: &'static str,
    tracking_numbers: Vec<&'static str>,
    more_count: u32,
    updated: &'static str,
    created: &'static str,
}

// ---------------------------------------------------------------------------
// Additional mock data for new pages
// ---------------------------------------------------------------------------

fn mock_text_available() -> Vec<TextNumber> {
    vec![
        TextNumber { number: "(276) 201-0001", e164: "+12762010001" },
        TextNumber { number: "(276) 201-0002", e164: "+12762010002" },
        TextNumber { number: "(276) 201-0004", e164: "+12762010004" },
        TextNumber { number: "(276) 201-0005", e164: "+12762010005" },
        TextNumber { number: "(276) 201-0006", e164: "+12762010006" },
        TextNumber { number: "(276) 201-0007", e164: "+12762010007" },
        TextNumber { number: "(276) 201-0008", e164: "+12762010008" },
        TextNumber { number: "(910) 991-0047", e164: "+19109910047" },
        TextNumber { number: "(919) 290-4449", e164: "+19192904449" },
    ]
}

fn mock_text_assigned() -> Vec<TextNumber> {
    vec![
        TextNumber { number: "(980) 553-2289", e164: "+19805532289" },
        TextNumber { number: "(855) 614-1888", e164: "+18556141888" },
        TextNumber { number: "(832) 558-3313", e164: "+18325583313" },
        TextNumber { number: "(252) 351-2397", e164: "+12523512397" },
        TextNumber { number: "(888) 361-3349", e164: "+18883613349" },
        TextNumber { number: "(888) 399-8387", e164: "+18883998387" },
        TextNumber { number: "(855) 563-5818", e164: "+18555635818" },
    ]
}

fn mock_long_text_available() -> Vec<TextNumber> {
    vec![
        TextNumber { number: "(276) 201-0001", e164: "+12762010001" },
        TextNumber { number: "(276) 201-0004", e164: "+12762010004" },
        TextNumber { number: "(276) 201-0006", e164: "+12762010006" },
        TextNumber { number: "(276) 201-0008", e164: "+12762010008" },
        TextNumber { number: "(910) 991-0047", e164: "+19109910047" },
        TextNumber { number: "(919) 290-4449", e164: "+19192904449" },
        TextNumber { number: "(252) 235-4100", e164: "+12522354100" },
    ]
}

fn mock_target_numbers() -> Vec<TargetNumber> {
    vec![
        TargetNumber { number: "(252) 351-2397", description: "Description", target_type: "Phone Match", tracking_numbers: vec!["+12523512397", "+19199180047"], more_count: 0, updated: "2025-12-20", created: "2024-03-15" },
        TargetNumber { number: "(888) 361-3349", description: "Website", target_type: "Phone Match", tracking_numbers: vec!["+18883613349", "+19802231234"], more_count: 9, updated: "2025-12-18", created: "2023-11-01" },
        TargetNumber { number: "(888) 359-4517", description: "Google Adwords NC", target_type: "Phone Match", tracking_numbers: vec!["+18883594517"], more_count: 0, updated: "2025-11-30", created: "2023-08-20" },
        TargetNumber { number: "(855) 563-5818", description: "Google Adwords", target_type: "Phone Match", tracking_numbers: vec!["+18555635818"], more_count: 15, updated: "2025-11-25", created: "2023-01-15" },
    ]
}

// ---------------------------------------------------------------------------
// Text Numbers page - Dual-list picker
// ---------------------------------------------------------------------------

#[component]
fn TextNumberDualList(
    title: &'static str,
    available: Vec<TextNumber>,
    assigned: Vec<TextNumber>,
    available_total: u32,
    assigned_total: u32,
) -> impl IntoView {
    let avail_count = available.len();
    let assign_count = assigned.len();
    let avail_label = format!("Available: {} of {}", avail_count, available_total);
    let assign_label = format!("Assigned: {} of {}", assign_count, assigned_total);

    view! {
        <div class="bg-white rounded-lg border border-gray-200 p-4 mb-6">
            <h3 class="text-sm font-semibold text-gray-700 mb-3">{title}</h3>

            <div class="flex items-start gap-3">
                // Available panel
                <div class="flex-1 border border-gray-200 rounded-lg">
                    <div class="bg-gray-50 px-3 py-2 border-b border-gray-200 rounded-t-lg">
                        <span class="text-xs font-medium text-gray-600">{avail_label}</span>
                    </div>
                    <div class="p-2">
                        <input type="text" placeholder="Search numbers..." class="input input-xs input-bordered w-full mb-2" />
                        <div class="h-40 overflow-y-auto space-y-0.5">
                            {available.into_iter().map(|n| {
                                view! {
                                    <label class="flex items-center gap-2 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">
                                        <input type="checkbox" class="checkbox checkbox-xs checkbox-primary" />
                                        <span class="text-xs text-gray-700">{n.number}</span>
                                    </label>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                    <div class="bg-gray-50 px-3 py-1.5 border-t border-gray-200 rounded-b-lg flex gap-2">
                        <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Select All"</a>
                        <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Unselect All"</a>
                    </div>
                </div>

                // Arrow buttons
                <div class="flex flex-col items-center gap-1 pt-16">
                    <button class="btn btn-xs btn-outline border-gray-300 text-gray-500 w-8">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span>
                    </button>
                    <button class="btn btn-xs btn-outline border-gray-300 text-gray-500 w-8">
                        <span class="text-[10px]">">>"</span>
                    </button>
                    <button class="btn btn-xs btn-outline border-gray-300 text-gray-500 w-8">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span>
                    </button>
                    <button class="btn btn-xs btn-outline border-gray-300 text-gray-500 w-8">
                        <span class="text-[10px]">"<<"</span>
                    </button>
                </div>

                // Assigned panel
                <div class="flex-1 border border-gray-200 rounded-lg">
                    <div class="bg-gray-50 px-3 py-2 border-b border-gray-200 rounded-t-lg">
                        <span class="text-xs font-medium text-gray-600">{assign_label}</span>
                    </div>
                    <div class="p-2">
                        <input type="text" placeholder="Search numbers..." class="input input-xs input-bordered w-full mb-2" />
                        <div class="h-40 overflow-y-auto space-y-0.5">
                            {assigned.into_iter().map(|n| {
                                view! {
                                    <label class="flex items-center gap-2 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">
                                        <input type="checkbox" class="checkbox checkbox-xs checkbox-primary" />
                                        <span class="text-xs text-gray-700">{n.number}</span>
                                    </label>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                    <div class="bg-gray-50 px-3 py-1.5 border-t border-gray-200 rounded-b-lg flex gap-2">
                        <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Select All"</a>
                        <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Unselect All"</a>
                    </div>
                </div>
            </div>

            <div class="mt-3 flex justify-end">
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Settings"</button>
            </div>
        </div>
    }
}

#[component]
pub fn TextNumbersPage() -> impl IntoView {
    let available = mock_text_available();
    let assigned = mock_text_assigned();
    let long_available = mock_long_text_available();
    // Build long text assigned list (reuse some numbers)
    let long_assigned: Vec<TextNumber> = vec![
        TextNumber { number: "(980) 553-2289", e164: "+19805532289" },
        TextNumber { number: "(855) 614-1888", e164: "+18556141888" },
        TextNumber { number: "(832) 558-3313", e164: "+18325583313" },
        TextNumber { number: "(252) 351-2397", e164: "+12523512397" },
        TextNumber { number: "(888) 361-3349", e164: "+18883613349" },
        TextNumber { number: "(888) 399-8387", e164: "+18883998387" },
        TextNumber { number: "(855) 563-5818", e164: "+18555635818" },
        TextNumber { number: "(276) 201-0002", e164: "+12762010002" },
    ];

    view! {
        <div class="flex flex-col h-full">
            // Header
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-lg font-semibold text-gray-800">"Text Message Numbers"</h1>
                    <p class="text-xs text-gray-500">"Choose which numbers can send and receive text messages"</p>
                </div>
            </header>

            // Tabs
            <div class="bg-white border-b border-gray-200 px-4">
                <div class="flex gap-6 text-sm">
                    <button class="py-2.5 border-b-2 border-iiz-cyan text-iiz-cyan font-medium">"Incoming Messages"</button>
                    <button class="py-2.5 text-gray-500 hover:text-gray-700">"Outgoing Messages"</button>
                </div>
            </div>

            // Content
            <div class="flex-1 overflow-y-auto p-4">
                <TextNumberDualList
                    title="Allow Text Messages"
                    available=available
                    assigned=assigned
                    available_total=250
                    assigned_total=250
                />

                <TextNumberDualList
                    title="Outgoing Long Text Messages"
                    available=long_available
                    assigned=long_assigned
                    available_total=250
                    assigned_total=250
                />

                <div class="bg-blue-50 border border-blue-200 rounded-lg p-3 text-xs text-blue-700">
                    <span class="font-semibold">"SMS Segmentation: "</span>
                    "Standard SMS messages are limited to 160 characters. Long text messages can contain up to 1,600 characters and will be split into multiple segments for delivery. Each segment is billed separately."
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Port Numbers page - Multi-step wizard form
// ---------------------------------------------------------------------------

#[component]
pub fn PortNumbersPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            // Breadcrumb header
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-2 flex-shrink-0">
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Port Numbers"</a>
                <span class="text-xs text-gray-400">">"</span>
                <span class="text-xs text-gray-500">"New"</span>
                <span class="text-xs text-gray-400">">"</span>
                <span class="text-xs font-medium text-gray-700">"General"</span>
            </header>

            // Form content
            <div class="flex-1 overflow-y-auto p-6">
                <div class="max-w-2xl">
                    // Name field
                    <div class="mb-6">
                        <label class="text-sm font-medium text-gray-700 block mb-1">"Name"</label>
                        <input type="text" placeholder="Enter port request name" class="input input-bordered w-full" />
                        <p class="text-xs text-gray-400 mt-1">"Give this port request a descriptive name for easy identification."</p>
                    </div>

                    // User Details section (expanded)
                    <div class="bg-white border border-gray-200 rounded-lg mb-4">
                        <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 cursor-pointer">
                            <h3 class="text-sm font-semibold text-gray-700">"User Details"</h3>
                            <span class="w-4 h-4 inline-flex text-gray-400 rotate-180">
                                <Icon icon=icondata::BsChevronDown />
                            </span>
                        </div>
                        <div class="p-4 space-y-4">
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">
                                        "First Name"
                                        <span class="text-red-500 ml-0.5">"*"</span>
                                    </label>
                                    <input type="text" class="input input-bordered w-full" />
                                </div>
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">
                                        "Last Name"
                                        <span class="text-red-500 ml-0.5">"*"</span>
                                    </label>
                                    <input type="text" class="input input-bordered w-full" />
                                </div>
                            </div>
                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">
                                    "Business Name"
                                    <span class="text-red-500 ml-0.5">"*"</span>
                                </label>
                                <input type="text" class="input input-bordered w-full" />
                            </div>
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Service Account Number"</label>
                                    <input type="text" class="input input-bordered w-full" />
                                </div>
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Account PIN Number"</label>
                                    <input type="text" class="input input-bordered w-full" />
                                </div>
                            </div>
                        </div>
                    </div>

                    // Billing Details section (collapsed)
                    <div class="bg-white border border-gray-200 rounded-lg mb-6">
                        <div class="flex items-center justify-between px-4 py-3 cursor-pointer">
                            <h3 class="text-sm font-semibold text-gray-700">"Billing Details"</h3>
                            <span class="w-4 h-4 inline-flex text-gray-400">
                                <Icon icon=icondata::BsChevronDown />
                            </span>
                        </div>
                        <div class="hidden p-4 space-y-4 border-t border-gray-200">
                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">
                                    "Street Address"
                                    <span class="text-red-500 ml-0.5">"*"</span>
                                </label>
                                <input type="text" class="input input-bordered w-full" />
                            </div>
                            <div class="grid grid-cols-3 gap-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">
                                        "City"
                                        <span class="text-red-500 ml-0.5">"*"</span>
                                    </label>
                                    <input type="text" class="input input-bordered w-full" />
                                </div>
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">
                                        "State"
                                        <span class="text-red-500 ml-0.5">"*"</span>
                                    </label>
                                    <input type="text" class="input input-bordered w-full" />
                                </div>
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">
                                        "ZIP"
                                        <span class="text-red-500 ml-0.5">"*"</span>
                                    </label>
                                    <input type="text" class="input input-bordered w-full" />
                                </div>
                            </div>
                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">
                                    "Country"
                                    <span class="text-red-500 ml-0.5">"*"</span>
                                </label>
                                <select class="select select-bordered w-full">
                                    <option selected>"United States"</option>
                                    <option>"Canada"</option>
                                    <option>"United Kingdom"</option>
                                </select>
                            </div>
                        </div>
                    </div>

                    // Continue button
                    <div class="flex justify-end">
                        <button class="btn bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Continue"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Number Pools page - Form-based configuration
// ---------------------------------------------------------------------------

#[component]
pub fn NumberPoolsPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            // Breadcrumb header
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-2 flex-shrink-0">
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Number Pools"</a>
                <span class="text-xs text-gray-400">">"</span>
                <span class="text-xs text-gray-500">"New"</span>
                <span class="text-xs text-gray-400">">"</span>
                <span class="text-xs font-medium text-gray-700">"General"</span>
            </header>

            // Form content
            <div class="flex-1 overflow-y-auto p-6">
                <div class="max-w-2xl space-y-4">
                    // Card 1: General
                    <div class="bg-white border border-gray-200 rounded-lg">
                        <div class="px-4 py-3 border-b border-gray-200">
                            <h3 class="text-sm font-semibold text-gray-700">"General"</h3>
                        </div>
                        <div class="p-4 space-y-4">
                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">
                                    "Name"
                                    <span class="text-red-500 ml-0.5">"*"</span>
                                </label>
                                <input type="text" class="input input-bordered w-full" />
                            </div>
                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">"Description"</label>
                                <textarea class="textarea textarea-bordered w-full h-20" placeholder="Optional description..."></textarea>
                            </div>
                        </div>
                    </div>

                    // Card 2: Tracking
                    <div class="bg-white border border-gray-200 rounded-lg">
                        <div class="px-4 py-3 border-b border-gray-200">
                            <h3 class="text-sm font-semibold text-gray-700">"Tracking"</h3>
                        </div>
                        <div class="p-4 space-y-4">
                            <div class="flex items-center justify-between">
                                <label class="text-sm text-gray-700">"Custom Tracking Source"</label>
                                <input type="checkbox" class="toggle toggle-sm" />
                            </div>
                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">"Visitor Type"</label>
                                <select class="select select-bordered w-full">
                                    <option selected>"All Visitors"</option>
                                    <option>"New Visitors"</option>
                                    <option>"Returning Visitors"</option>
                                </select>
                            </div>
                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">"Estimated Visitor Count"</label>
                                <input type="number" value="1" class="input input-bordered w-32" />
                            </div>
                        </div>
                    </div>

                    // Card 3: Numbers Management
                    <div class="bg-white border border-gray-200 rounded-lg">
                        <div class="px-4 py-3 border-b border-gray-200">
                            <h3 class="text-sm font-semibold text-gray-700">"Numbers Management"</h3>
                        </div>
                        <div class="p-4 space-y-4">
                            <div class="flex items-center justify-between">
                                <label class="text-sm text-gray-700">"Auto Management"</label>
                                <input type="checkbox" class="toggle toggle-sm toggle-success" checked />
                            </div>

                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">"Target Accuracy"</label>
                                <div class="flex items-center gap-3">
                                    <span class="text-lg font-bold text-iiz-cyan">"99%"</span>
                                    <input type="range" min="90" max="100" value="99" class="range range-xs range-primary flex-1" />
                                </div>
                            </div>

                            <div class="bg-blue-50 border border-blue-200 rounded-lg p-3 text-xs text-blue-700">
                                "Based on your visitor count, we recommend "
                                <span class="font-bold">"1"</span>
                                " tracking number(s) to maintain the target accuracy level."
                            </div>

                            <div class="text-sm text-gray-500">
                                "Cost: "
                                <span class="font-medium text-gray-700">"$1.26/mo per number"</span>
                            </div>

                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">"Country"</label>
                                <select class="select select-bordered w-full">
                                    <option selected>"US +1"</option>
                                    <option>"CA +1"</option>
                                    <option>"GB +44"</option>
                                </select>
                            </div>

                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-2">"Number Type"</label>
                                <div class="flex items-center gap-4">
                                    <label class="flex items-center gap-2 cursor-pointer">
                                        <input type="radio" name="number_type" class="radio radio-sm radio-primary" checked />
                                        <span class="text-sm text-gray-700">"Local"</span>
                                    </label>
                                    <label class="flex items-center gap-2 cursor-pointer">
                                        <input type="radio" name="number_type" class="radio radio-sm radio-primary" />
                                        <span class="text-sm text-gray-700">"Toll Free"</span>
                                    </label>
                                </div>
                            </div>

                            <div>
                                <label class="text-sm font-medium text-gray-700 block mb-1">"Area Code"</label>
                                <select class="select select-bordered w-32">
                                    <option selected>"205"</option>
                                    <option>"212"</option>
                                    <option>"276"</option>
                                    <option>"310"</option>
                                    <option>"404"</option>
                                    <option>"512"</option>
                                    <option>"702"</option>
                                    <option>"919"</option>
                                </select>
                            </div>

                            <div class="flex items-center justify-between">
                                <label class="text-sm text-gray-700">"Allow Overlay"</label>
                                <input type="checkbox" class="toggle toggle-sm" />
                            </div>
                        </div>
                    </div>

                    // Save button
                    <div class="flex justify-end">
                        <button class="btn bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Target Numbers page - Data table
// ---------------------------------------------------------------------------

#[component]
pub fn TargetNumbersPage() -> impl IntoView {
    let targets = mock_target_numbers();

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="mr-auto">
                    <h1 class="text-lg font-semibold text-gray-800">"Target Numbers"</h1>
                    <p class="text-xs text-gray-500">"Numbers to replace with tracking numbers on your website"</p>
                </div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Target Number"</button>
            </header>

            // Search bar
            <div class="bg-white border-b border-gray-200 px-4 py-2 flex items-center gap-3">
                <div class="join">
                    <input type="text" placeholder="Search..." class="input input-sm input-bordered join-item w-48" />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                <span class="text-sm text-gray-500">"4 Target Numbers"</span>
            </div>

            // Table headers
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_1fr_100px_1fr_100px_100px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Name"</div>
                    <div class="col-header">"Type"</div>
                    <div class="col-header">"Tracking Numbers"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            // Table rows
            <div class="flex-1 overflow-y-auto">
                {targets.into_iter().map(|t| {
                    let tracking_display = t.tracking_numbers.join(", ");
                    let more_text = if t.more_count > 0 {
                        format!("... {} more", t.more_count)
                    } else {
                        String::new()
                    };
                    view! {
                        <div class="activity-row grid grid-cols-[32px_1fr_100px_1fr_100px_100px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <button class="btn btn-xs btn-ghost text-gray-400">
                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                            </button>
                            <div>
                                <div class="text-sm font-medium text-iiz-blue-link">{t.number}</div>
                                <div class="text-xs text-gray-500">{t.description}</div>
                            </div>
                            <div>
                                <span class="badge badge-sm bg-gray-100 text-gray-600 border-none">{t.target_type}</span>
                            </div>
                            <div>
                                <span class="text-xs text-gray-600">{tracking_display}</span>
                                {if !more_text.is_empty() {
                                    view! { <span class="text-xs text-iiz-cyan ml-1">{more_text}</span> }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </div>
                            <div class="text-xs text-gray-500">{t.updated}</div>
                            <div class="text-xs text-gray-500">{t.created}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Pagination bar
            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-4 of 4"</span>
                <div class="flex-1"></div>
                <span class="text-xs text-gray-400 mr-2">"Per page:"</span>
                <select class="select select-xs select-bordered">
                    <option selected>"10"</option>
                    <option>"25"</option>
                    <option>"50"</option>
                    <option>"100"</option>
                </select>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Tracking Code page - Installation guide
// ---------------------------------------------------------------------------

#[component]
pub fn TrackingCodePage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            // Breadcrumb header
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-2 flex-shrink-0">
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Tracking Code"</a>
                <span class="text-xs text-gray-400">">"</span>
                <span class="text-xs font-medium text-gray-700">"Tracking Code Installation"</span>
                <div class="flex-1"></div>
                <button class="btn btn-sm btn-outline border-iiz-cyan text-iiz-cyan">"Refresh Tracking Code"</button>
            </header>

            // Content
            <div class="flex-1 overflow-y-auto p-6">
                <div class="max-w-3xl space-y-6">
                    // Info panel
                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 text-sm text-blue-700">
                        "Dynamic Number Insertion (DNI) works by placing a small JavaScript snippet on your website. "
                        "When a visitor arrives, the script automatically replaces your target phone numbers with unique tracking numbers, "
                        "allowing you to attribute each call to the correct marketing source."
                    </div>

                    // Important Setup Notes
                    <div class="bg-white border border-gray-200 rounded-lg p-4">
                        <h3 class="text-sm font-semibold text-gray-700 mb-2">"Important Setup Notes"</h3>
                        <ul class="space-y-2 text-sm text-gray-600">
                            <li class="flex items-start gap-2">
                                <span class="w-1.5 h-1.5 rounded-full bg-gray-400 mt-1.5 flex-shrink-0"></span>
                                "Avoid conflicts with other call tracking scripts. Only one DNI provider should be active on a page at a time."
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="w-1.5 h-1.5 rounded-full bg-gray-400 mt-1.5 flex-shrink-0"></span>
                                "Hardcoded phone numbers in images, JavaScript variables, or CSS content properties will not be swapped. Use plain HTML text."
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="w-1.5 h-1.5 rounded-full bg-gray-400 mt-1.5 flex-shrink-0"></span>
                                "Numbers inside iframes from different domains cannot be replaced due to cross-origin restrictions."
                            </li>
                        </ul>
                    </div>

                    // Code snippet
                    <div class="bg-white border border-gray-200 rounded-lg">
                        <div class="px-4 py-3 border-b border-gray-200 flex items-center justify-between">
                            <h3 class="text-sm font-semibold text-gray-700">"Tracking Code Snippet"</h3>
                            <button class="btn btn-xs btn-outline border-iiz-cyan text-iiz-cyan">
                                <span class="w-3 h-3 inline-flex mr-1"><Icon icon=icondata::BsClipboard /></span>
                                "Copy to Clipboard"
                            </button>
                        </div>
                        <div class="bg-gray-900 p-4 rounded-b-lg">
                            <code class="text-green-400 font-mono text-sm">
                                "<script async src=\"//155169.tctm.co/t.js\"></script>"
                            </code>
                        </div>
                    </div>

                    // Email Developer section
                    <div class="bg-white border border-gray-200 rounded-lg p-4">
                        <h3 class="text-sm font-semibold text-gray-700 mb-3">"Email Developer"</h3>
                        <div class="flex items-center gap-3">
                            <input type="email" placeholder="developer@example.com" class="input input-bordered flex-1" />
                            <button class="btn bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Send Instructions"</button>
                        </div>
                    </div>

                    // Platform tabs
                    <div class="bg-white border border-gray-200 rounded-lg">
                        <div class="border-b border-gray-200 px-4">
                            <div class="flex gap-4 text-sm">
                                <button class="py-2.5 border-b-2 border-iiz-cyan text-iiz-cyan font-medium">"STANDARD"</button>
                                <button class="py-2.5 text-gray-500 hover:text-gray-700">"DEVELOPER RESOURCES"</button>
                                <button class="py-2.5 text-gray-500 hover:text-gray-700">"TESTING"</button>
                            </div>
                        </div>

                        <div class="p-4">
                            // Platform logos as buttons
                            <h4 class="text-sm font-medium text-gray-700 mb-3">"Install on a Specific Platform"</h4>
                            <div class="flex flex-wrap gap-2 mb-6">
                                <button class="btn btn-sm btn-outline border-gray-300 text-gray-600">"AMP"</button>
                                <button class="btn btn-sm btn-outline border-gray-300 text-gray-600">"Google Tag Manager"</button>
                                <button class="btn btn-sm btn-outline border-gray-300 text-gray-600">"Magento"</button>
                                <button class="btn btn-sm btn-outline border-gray-300 text-gray-600">"Wix"</button>
                                <button class="btn btn-sm btn-outline border-gray-300 text-gray-600">"WordPress"</button>
                            </div>

                            // Manual install note
                            <div class="bg-gray-50 border border-gray-200 rounded-lg p-3">
                                <h4 class="text-sm font-medium text-gray-700">"Not Using One of These Platforms?"</h4>
                                <p class="text-xs text-gray-500 mt-1">
                                    "Paste the tracking code snippet above into your website's HTML, just before the closing "
                                    <code class="bg-gray-200 px-1 rounded text-xs">"</body>"</code>
                                    " tag on every page you want to track."
                                </p>
                            </div>
                        </div>
                    </div>

                    // Advanced Options (expandable)
                    <div class="bg-white border border-gray-200 rounded-lg">
                        <div class="flex items-center justify-between px-4 py-3 cursor-pointer">
                            <h3 class="text-sm font-semibold text-gray-700">"Advanced Options"</h3>
                            <span class="w-4 h-4 inline-flex text-gray-400">
                                <Icon icon=icondata::BsChevronDown />
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
