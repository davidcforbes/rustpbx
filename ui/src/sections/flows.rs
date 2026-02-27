use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

use crate::api::api_get;
use crate::api::types::{
    BulkMessageItem, ChatWidgetItem, FormReactorItem, ListResponse, PaginationMeta, QueueItem,
    ScheduleItem, SmartRouterItem, TriggerItem, VoiceMenuItem, WebhookItem,
};

// ---------------------------------------------------------------------------
// Flows side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn FlowsSideNav() -> impl IntoView {
    let location = use_location();
    let active = |href: &'static str| {
        move || {
            if location.pathname.get() == href { "side-nav-item active" } else { "side-nav-item" }
        }
    };

    view! {
        <div class="px-4 pt-4 pb-2">
            <div class="flex items-center gap-2 text-iiz-cyan">
                <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsArrowLeftRight /></span>
                <span class="text-lg font-light">"Flows"</span>
            </div>
        </div>

        <nav class="px-2 pb-4">
            // Routing group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsArrowLeftRight /></span>
                    "Routing"
                </h3>
                <a href="/flows/voice-menus" class=active("/flows/voice-menus")>"Voice Menus"</a>
                <a href="/flows/queues" class=active("/flows/queues")>"Queues"</a>
                <a href="/flows/smart-routers" class=active("/flows/smart-routers")>"Smart Routers"</a>
                <a href="/flows/geo-routers" class=active("/flows/geo-routers")>"Geo Routers"</a>
                <a href="/flows/schedules" class=active("/flows/schedules")>"Schedules"</a>
                <a href="/flows/agent-scripts" class=active("/flows/agent-scripts")>"Agent Scripts"</a>
                <a href="/flows/routing-tables" class=active("/flows/routing-tables")>"Routing Tables"</a>
                <a href="/flows/voicemails" class=active("/flows/voicemails")>"Voicemails"</a>
            </div>

            // Automation group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsLightningFill /></span>
                    "Automation"
                </h3>
                <a href="/flows/workflows" class=active("/flows/workflows")>"Workflows"</a>
                <a href="/flows/triggers" class=active("/flows/triggers")>"Triggers"</a>
                <a href="/flows/keyword-spotting" class=active("/flows/keyword-spotting")>"Keyword Spotting"</a>
                <a href="/flows/lambdas" class=active("/flows/lambdas")>"Lambdas"</a>
                <a href="/flows/api-logs" class=active("/flows/api-logs")>"API Logs"</a>
                <a href="/flows/global" class=active("/flows/global")>"Global"</a>
                <a href="/flows/webhooks" class=active("/flows/webhooks")>"Webhooks"</a>
            </div>

            // Engagement group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsChatDotsFill /></span>
                    "Engagement"
                </h3>
                <a href="/flows/bulk-messages" class=active("/flows/bulk-messages")>"Bulk Messages"</a>
                <a href="/flows/lead-reactor" class=active("/flows/lead-reactor")>"LeadReactor"</a>
                <a href="/flows/smart-dialers" class=active("/flows/smart-dialers")>"Smart Dialers"</a>
                <a href="/flows/form-reactor" class=active("/flows/form-reactor")>"FormReactor"</a>
                <a href="/flows/chat-widget" class=active("/flows/chat-widget")>"Chat Widget"</a>
                <a href="/flows/chat-ai" class=active("/flows/chat-ai")>
                    "ChatAI"
                    <span class="badge badge-xs bg-iiz-cyan text-white border-none ml-1">"BETA"</span>
                </a>
                <a href="/flows/dialogflow" class=active("/flows/dialogflow")>
                    "Dialogflow"
                    <span class="badge badge-xs bg-iiz-cyan text-white border-none ml-1">"BETA"</span>
                </a>
                <a href="/flows/reminders" class=active("/flows/reminders")>"Reminders"</a>
            </div>
        </nav>
    }
}

// ---------------------------------------------------------------------------
// Shared helpers (duplicated from numbers.rs to keep modules independent)
// ---------------------------------------------------------------------------

/// Format an ISO-8601 datetime string for display (just the first 19 chars).
fn fmt_date(iso: &str) -> String {
    iso.replace('T', " ")
        .trim_end_matches('Z')
        .chars()
        .take(19)
        .collect()
}

/// Format a NaiveTime string "HH:MM:SS" to "HH:MM AM/PM".
fn fmt_time(t: &str) -> String {
    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() >= 2 {
        if let Ok(h) = parts[0].parse::<u32>() {
            let m = parts[1];
            let (hour, ampm) = if h == 0 {
                (12, "AM")
            } else if h < 12 {
                (h, "AM")
            } else if h == 12 {
                (12, "PM")
            } else {
                (h - 12, "PM")
            };
            return format!("{}:{} {}", hour, m, ampm);
        }
    }
    t.to_string()
}

/// Render a pagination footer from real metadata.
fn pagination_footer(meta: &PaginationMeta) -> impl IntoView {
    let page = meta.page;
    let per_page = meta.per_page;
    let total_items = meta.total_items;
    let total_pages = meta.total_pages;
    let has_prev = meta.has_prev;
    let has_next = meta.has_next;

    let start = (page - 1) * per_page + 1;
    let end = std::cmp::min(page * per_page, total_items);
    let showing = format!("Showing {}-{} of {}", start, end, total_items);

    let mut pages: Vec<i64> = Vec::new();
    pages.push(1);
    if page > 3 {
        pages.push(-1);
    }
    for p in (page - 1)..=(page + 1) {
        if p > 1 && p < total_pages {
            pages.push(p);
        }
    }
    if page < total_pages - 2 {
        pages.push(-1);
    }
    if total_pages > 1 {
        pages.push(total_pages);
    }
    pages.dedup();

    view! {
        <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
            <span>{showing}</span>
            <div class="flex-1"></div>
            <div class="flex items-center gap-1">
                <button
                    class="btn btn-xs btn-ghost text-gray-400"
                    disabled=move || !has_prev
                >
                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span>
                </button>
                {pages.into_iter().map(|p| {
                    if p == -1 {
                        view! { <span class="text-xs text-gray-400">"..."</span> }.into_any()
                    } else if p == page {
                        let s = p.to_string();
                        view! { <button class="btn btn-xs bg-iiz-cyan text-white border-none">{s}</button> }.into_any()
                    } else {
                        let s = p.to_string();
                        view! { <button class="btn btn-xs btn-ghost">{s}</button> }.into_any()
                    }
                }).collect::<Vec<_>>()}
                <button
                    class="btn btn-xs btn-ghost text-gray-400"
                    disabled=move || !has_next
                >
                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span>
                </button>
            </div>
            <span class="text-xs text-gray-400 ml-2">"Per page:"</span>
            <select class="select select-xs select-bordered ml-1">
                <option selected>"25"</option>
                <option>"50"</option>
                <option>"100"</option>
            </select>
        </div>
    }
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

// (Mock structs for Triggers, Webhooks, BulkMessages, FormReactors removed — now using API types.)

// ---------------------------------------------------------------------------
// Voice Menus page
// ---------------------------------------------------------------------------

#[component]
pub fn VoiceMenusPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<VoiceMenuItem>>("/flows/voice-menus?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            // Info banner
            <div class="bg-blue-50 border border-blue-200 px-4 py-3 flex items-start gap-3 flex-shrink-0">
                <span class="w-5 h-5 inline-flex text-blue-500 mt-0.5"><Icon icon=icondata::BsInfoCircleFill /></span>
                <div class="flex-1 text-sm text-gray-700">
                    <span>"Voice Menus (IVR) allow callers to navigate options via keypress or speech recognition. "</span>
                    <a class="text-iiz-cyan hover:underline cursor-pointer">"Creating Voice Menus"</a>
                    <span>" · "</span>
                    <a class="text-iiz-cyan hover:underline cursor-pointer">"Overview"</a>
                    <span>" · "</span>
                    <a class="text-iiz-cyan hover:underline cursor-pointer">"Walkthrough Video"</a>
                </div>
                <button class="btn btn-xs btn-ghost text-gray-400">
                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsXLg /></span>
                </button>
            </div>

            // Title bar
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <h1 class="text-xl font-semibold text-iiz-dark">"Voice Menus"</h1>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Voice Menus", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Voice Menu"
                </button>
            </div>

            // Table header
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_1fr_80px_80px_100px_140px_140px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Name"</div>
                    <div class="col-header">"Greeting"</div>
                    <div class="col-header">"Speech Rec"</div>
                    <div class="col-header">"Speech Lang"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            // Table rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|m| {
                                    let has_greeting = m.greeting_type != "none";
                                    let speech_lang = m.speech_language.clone().unwrap_or_default();
                                    view! {
                                        <div class="activity-row grid grid-cols-[32px_1fr_80px_80px_100px_140px_140px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                            </button>
                                            <div class="text-sm font-medium text-iiz-blue-link">{m.name.clone()}</div>
                                            <div>
                                                {if has_greeting {
                                                    view! { <span class="w-4 h-4 inline-flex text-gray-400"><Icon icon=icondata::BsVolumeUpFill /></span> }.into_any()
                                                } else {
                                                    view! { <span class="text-xs text-gray-300">"-"</span> }.into_any()
                                                }}
                                            </div>
                                            <div>
                                                {if m.speech_recognition {
                                                    view! { <span class="text-green-500 text-sm">"Yes"</span> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </div>
                                            <div class="text-xs text-gray-500">{speech_lang}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&m.updated_at)}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&m.created_at)}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {pagination_footer(&meta)}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Queues page
// ---------------------------------------------------------------------------

#[component]
pub fn QueuesPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<QueueItem>>("/flows/queues?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Queues"</h1>
                    <p class="text-sm text-gray-500">"Allow callers to wait for the next available agent"</p>
                </div>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Queues", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Queue"
                </button>
            </div>

            // Table header
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_1fr_100px_80px_120px_80px_60px_140px_140px] gap-1 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Name"</div>
                    <div class="col-header">"Strategy"</div>
                    <div class="col-header">"Repeat"</div>
                    <div class="col-header">"Caller ID"</div>
                    <div class="col-header">"Max Wait"</div>
                    <div class="col-header">"Active"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            // Table rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|q| {
                                    let active_class = if q.is_active { "text-green-600 text-xs" } else { "text-gray-400 text-xs" };
                                    let active_text = if q.is_active { "Yes" } else { "No" };
                                    let caller_id = q.caller_id_display.clone().unwrap_or_else(|| "\u{2014}".to_string());
                                    let max_wait = format!("{}s", q.max_wait_secs);
                                    view! {
                                        <div class="activity-row grid grid-cols-[32px_1fr_100px_80px_120px_80px_60px_140px_140px] gap-1 px-4 py-2.5 items-center cursor-pointer">
                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                            </button>
                                            <div class="text-sm font-medium text-iiz-blue-link">{q.name.clone()}</div>
                                            <div class="text-xs text-gray-500">{q.strategy.clone()}</div>
                                            <div class="text-center">
                                                {if q.repeat_callers {
                                                    view! { <span class="text-green-500 text-sm">"Yes"</span> }.into_any()
                                                } else {
                                                    view! { <span class="text-gray-300 text-sm">"No"</span> }.into_any()
                                                }}
                                            </div>
                                            <div class="text-xs text-gray-500">{caller_id}</div>
                                            <div class="text-xs text-gray-500">{max_wait}</div>
                                            <div class=active_class>{active_text}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&q.updated_at)}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&q.created_at)}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {pagination_footer(&meta)}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Smart Routers page
// ---------------------------------------------------------------------------

#[component]
pub fn SmartRoutersPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<SmartRouterItem>>("/flows/smart-routers?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Smart Routers"</h1>
                    <p class="text-sm text-gray-500">"Conditionally route callers based on their specific properties"</p>
                </div>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Smart Routers", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Smart Router"
                </button>
            </div>

            // Table header
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_1fr_80px_60px_140px_140px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Name"</div>
                    <div class="col-header">"Priority"</div>
                    <div class="col-header">"Active"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            // Table rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|r| {
                                    let active_class = if r.is_active { "text-green-600 text-xs" } else { "text-gray-400 text-xs" };
                                    let active_text = if r.is_active { "Yes" } else { "No" };
                                    view! {
                                        <div class="activity-row grid grid-cols-[32px_1fr_80px_60px_140px_140px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                            </button>
                                            <div class="text-sm font-medium text-iiz-blue-link">{r.name.clone()}</div>
                                            <div class="text-xs text-gray-500">{r.priority.to_string()}</div>
                                            <div class=active_class>{active_text}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&r.updated_at)}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&r.created_at)}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {pagination_footer(&meta)}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Schedules page
// ---------------------------------------------------------------------------

#[component]
pub fn SchedulesPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ScheduleItem>>("/flows/schedules?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Schedules"</h1>
                    <p class="text-sm text-gray-500">"Recurring active times for agents, voice menus, call queues, and more"</p>
                </div>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Schedules", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Schedule"
                </button>
            </div>

            // Table header
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_1fr_180px_200px_180px_140px_140px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Name"</div>
                    <div class="col-header">"Time Zone"</div>
                    <div class="col-header">"Weekday Hours"</div>
                    <div class="col-header">"Weekend Hours"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            // Table rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|s| {
                                    // Build weekday hours display (Mon-Fri)
                                    let weekday_hours = {
                                        let days = [
                                            ("Mon", &s.monday_open, &s.monday_close),
                                            ("Tue", &s.tuesday_open, &s.tuesday_close),
                                            ("Wed", &s.wednesday_open, &s.wednesday_close),
                                            ("Thu", &s.thursday_open, &s.thursday_close),
                                            ("Fri", &s.friday_open, &s.friday_close),
                                        ];
                                        let mut lines: Vec<String> = Vec::new();
                                        for (label, open, close) in days {
                                            if let (Some(o), Some(c)) = (open, close) {
                                                lines.push(format!("{}: {} - {}", label, fmt_time(o), fmt_time(c)));
                                            }
                                        }
                                        if lines.is_empty() { vec!["Closed".to_string()] } else { lines }
                                    };
                                    // Build weekend hours display (Sat-Sun)
                                    let weekend_hours = {
                                        let days = [
                                            ("Sat", &s.saturday_open, &s.saturday_close),
                                            ("Sun", &s.sunday_open, &s.sunday_close),
                                        ];
                                        let mut lines: Vec<String> = Vec::new();
                                        for (label, open, close) in days {
                                            if let (Some(o), Some(c)) = (open, close) {
                                                lines.push(format!("{}: {} - {}", label, fmt_time(o), fmt_time(c)));
                                            }
                                        }
                                        if lines.is_empty() { vec!["Closed".to_string()] } else { lines }
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[32px_1fr_180px_200px_180px_140px_140px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                            </button>
                                            <div class="text-sm font-medium text-iiz-blue-link">{s.name.clone()}</div>
                                            <div class="text-xs text-gray-500">{s.timezone.clone()}</div>
                                            <div class="flex flex-col gap-0.5">
                                                {weekday_hours.into_iter().map(|t| view! { <span class="text-xs text-gray-600">{t}</span> }).collect::<Vec<_>>()}
                                            </div>
                                            <div class="flex flex-col gap-0.5">
                                                {weekend_hours.into_iter().map(|t| view! { <span class="text-xs text-gray-600">{t}</span> }).collect::<Vec<_>>()}
                                            </div>
                                            <div class="text-xs text-gray-500">{fmt_date(&s.updated_at)}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&s.created_at)}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {pagination_footer(&meta)}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Triggers page
// ---------------------------------------------------------------------------

#[component]
pub fn TriggersPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<TriggerItem>>("/flows/triggers?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Triggers"</h1>
                    <p class="text-sm text-gray-500">"Trigger actions on your activities"</p>
                </div>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Triggers", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Visual Workflows"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"API Logs"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Trigger"
                </button>
            </div>

            // Table header
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_180px_180px_140px_80px_80px_120px_120px] gap-1 px-4 py-2 items-center min-w-max">
                    <div class="col-header"></div>
                    <div class="col-header">"Name"</div>
                    <div class="col-header">"Trigger Event"</div>
                    <div class="col-header">"Run On"</div>
                    <div class="col-header text-right">"Runs (7d)"</div>
                    <div class="col-header">"Status"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            // Table rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|t| {
                                    let run_on = t.run_on.clone().unwrap_or_default();
                                    let status_class = match t.status.as_str() {
                                        "active" => "badge badge-sm bg-green-500 text-white border-none",
                                        "paused" => "badge badge-sm bg-orange-400 text-white border-none",
                                        _ => "badge badge-sm bg-gray-400 text-white border-none",
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[32px_180px_180px_140px_80px_80px_120px_120px] gap-1 px-4 py-3 items-center cursor-pointer min-w-max">
                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                            </button>
                                            <div class="text-sm font-medium text-iiz-blue-link">{t.name.clone()}</div>
                                            <div class="text-xs text-gray-500">{t.trigger_event.clone()}</div>
                                            <div class="text-xs text-gray-500">{run_on}</div>
                                            <div class="text-sm font-bold text-gray-700 text-right">{t.runs_7d.to_string()}</div>
                                            <div><span class=status_class>{t.status.clone()}</span></div>
                                            <div class="text-xs text-gray-500">{fmt_date(&t.updated_at)}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&t.created_at)}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {pagination_footer(&meta)}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Webhooks page
// ---------------------------------------------------------------------------

#[component]
pub fn WebhooksPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<WebhookItem>>("/flows/webhooks?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Webhooks"</h1>
                    <p class="text-sm text-gray-500">"Send data to your servers"</p>
                </div>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Webhooks", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Global Webhooks"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"API Logs"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-xs bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ Start"
                </button>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Webhook"
                </button>
            </div>

            // Table header
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_160px_180px_200px_60px_80px_80px_120px_120px_100px] gap-1 px-4 py-2 items-center min-w-max">
                    <div class="col-header"></div>
                    <div class="col-header">"Name"</div>
                    <div class="col-header">"Trigger Event"</div>
                    <div class="col-header">"Callback URL"</div>
                    <div class="col-header">"Method"</div>
                    <div class="col-header">"Body Type"</div>
                    <div class="col-header">"Status"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                    <div class="col-header">"Actions"</div>
                </div>
            </div>

            // Table rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|h| {
                                    let trigger_event = h.trigger_event.clone().unwrap_or_default();
                                    let status_class = match h.status.as_str() {
                                        "active" => "badge badge-sm bg-green-500 text-white border-none",
                                        "paused" => "badge badge-sm bg-orange-400 text-white border-none",
                                        _ => "badge badge-sm bg-gray-400 text-white border-none",
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[32px_160px_180px_200px_60px_80px_80px_120px_120px_100px] gap-1 px-4 py-2.5 items-center cursor-pointer min-w-max">
                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                            </button>
                                            <div class="text-sm font-medium text-iiz-blue-link">{h.name.clone()}</div>
                                            <div class="text-xs text-gray-500">{trigger_event}</div>
                                            <div class="text-xs text-gray-400 truncate max-w-[200px]">{h.callback_url.clone()}</div>
                                            <div class="text-xs text-gray-500">{h.method.clone()}</div>
                                            <div class="text-xs text-gray-500">{h.body_type.clone()}</div>
                                            <div><span class=status_class>{h.status.clone()}</span></div>
                                            <div class="text-xs text-gray-500">{fmt_date(&h.updated_at)}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&h.created_at)}</div>
                                            <div class="flex items-center gap-1">
                                                <button class="btn btn-xs bg-iiz-cyan text-white border-none">"Test"</button>
                                                <button class="btn btn-xs btn-ghost text-gray-400">"Deact."</button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {pagination_footer(&meta)}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Bulk Messages page
// ---------------------------------------------------------------------------

#[component]
pub fn BulkMessagesPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<BulkMessageItem>>("/flows/bulk-messages?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            // Warning banner
            <div class="bg-red-50 border border-red-300 px-4 py-3 flex items-center gap-3 flex-shrink-0">
                <span class="w-5 h-5 inline-flex text-red-500"><Icon icon=icondata::BsExclamationTriangleFill /></span>
                <span class="text-sm text-red-700">
                    "You have Bulk Messages that need attention. To ensure delivery, please deactivate the existing messages and recreate them."
                </span>
            </div>

            // Title bar
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Bulk Messages"</h1>
                    <p class="text-sm text-gray-500">"Send text messages to a group of recipients"</p>
                </div>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Bulk Messages", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Bulk Message"
                </button>
            </div>

            // Status filter tabs
            <div class="bg-white border-b border-gray-200 px-6 py-2 flex items-center gap-2 flex-shrink-0">
                <button class="btn btn-xs bg-iiz-cyan text-white border-none">"All"</button>
                <button class="btn btn-xs btn-ghost">"Sending"</button>
                <button class="btn btn-xs btn-ghost">"Pending"</button>
                <button class="btn btn-xs btn-ghost">"Completed"</button>
                <button class="btn btn-xs btn-ghost">"Cancelled"</button>
                <button class="btn btn-xs btn-ghost">"Failed"</button>
                <button class="btn btn-xs btn-ghost">"Export..."</button>
            </div>

            // Table header
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_200px_120px_200px_70px_70px_70px_70px_80px_120px_120px] gap-1 px-4 py-2 items-center min-w-max">
                    <div class="col-header"></div>
                    <div class="col-header">"Label"</div>
                    <div class="col-header">"Phone"</div>
                    <div class="col-header">"Body"</div>
                    <div class="col-header text-center">"Recipients"</div>
                    <div class="col-header text-center">"Sent"</div>
                    <div class="col-header text-center">"Delivered"</div>
                    <div class="col-header text-center">"Failed"</div>
                    <div class="col-header">"Status"</div>
                    <div class="col-header">"Scheduled"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            // Table rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|m| {
                                    let label = m.label.clone().unwrap_or_default();
                                    let phone = m.sender_phone.clone().unwrap_or_default();
                                    let scheduled = m.scheduled_at.as_deref().map(fmt_date).unwrap_or_default();
                                    let status_class = match m.status.as_str() {
                                        "completed" | "Completed" => "badge badge-sm bg-green-500 text-white border-none",
                                        "failed" | "Failed" => "badge badge-sm bg-red-500 text-white border-none",
                                        "sending" | "Sending" => "badge badge-sm bg-blue-500 text-white border-none",
                                        "pending" | "Pending" => "badge badge-sm bg-orange-400 text-white border-none",
                                        _ => "badge badge-sm bg-gray-400 text-white border-none",
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[32px_200px_120px_200px_70px_70px_70px_70px_80px_120px_120px] gap-1 px-4 py-2.5 items-center cursor-pointer min-w-max">
                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsArrowRepeat /></span>
                                            </button>
                                            <div class="text-sm font-medium text-iiz-blue-link truncate max-w-[200px]">{label}</div>
                                            <div class="text-xs text-gray-500 whitespace-nowrap">{phone}</div>
                                            <div class="text-xs text-gray-400 truncate max-w-[200px]">{m.message_body.clone()}</div>
                                            <div class="text-sm font-bold text-gray-600 text-center">{m.recipient_count.to_string()}</div>
                                            <div class="text-sm text-gray-600 text-center">{m.sent_count.to_string()}</div>
                                            <div class="text-sm text-green-600 text-center">{m.delivered_count.to_string()}</div>
                                            <div class="text-sm text-red-600 text-center">{m.failed_count.to_string()}</div>
                                            <div><span class=status_class>{m.status.clone()}</span></div>
                                            <div class="text-xs text-gray-500">{scheduled}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&m.created_at)}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {pagination_footer(&meta)}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// FormReactor page
// ---------------------------------------------------------------------------

#[component]
pub fn FormReactorPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<FormReactorItem>>("/flows/form-reactor?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"FormReactors"</h1>
                    <p class="text-sm text-gray-500">"Embeddable forms that can trigger phone calls, text messages, and emails"</p>
                </div>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} FormReactors", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New FormReactor"
                </button>
            </div>

            // Table header
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_200px_1fr_80px_120px_120px_60px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Form Name"</div>
                    <div class="col-header">"Fields"</div>
                    <div class="col-header">"Status"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                    <div class="col-header text-center">"Calls"</div>
                </div>
            </div>

            // Table rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|f| {
                                    let fields = f.form_fields.clone().unwrap_or_default();
                                    let status_class = match f.status.as_str() {
                                        "active" => "badge badge-sm bg-green-500 text-white border-none",
                                        "draft" => "badge badge-sm bg-orange-400 text-white border-none",
                                        _ => "badge badge-sm bg-gray-400 text-white border-none",
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[32px_200px_1fr_80px_120px_120px_60px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                            </button>
                                            <div class="text-sm font-medium text-iiz-blue-link whitespace-nowrap">{f.name.clone()}</div>
                                            <div class="bg-gray-50 rounded px-3 py-1.5 text-xs text-gray-600">{fields}</div>
                                            <div><span class=status_class>{f.status.clone()}</span></div>
                                            <div class="text-xs text-gray-500">{fmt_date(&f.updated_at)}</div>
                                            <div class="text-xs text-gray-500">{fmt_date(&f.created_at)}</div>
                                            <div class="text-sm font-bold text-gray-600 text-center">{f.call_count.to_string()}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {pagination_footer(&meta)}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Geo Routers page
// ---------------------------------------------------------------------------

#[component]
pub fn GeoRoutersPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Geo Routers"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Name"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter geo router name" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span><span class="label-text-alt text-gray-400">"Optional"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this geo router"></textarea>
                            </div>
                            <div class="mt-4">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                        </div>
                    </div>
                    // Prompts card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Prompts"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Options"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option selected>"Silently route the call to the default action"</option>
                                    <option>"Prompt the caller to enter their zip code"</option>
                                    <option>"Prompt the caller to say their location"</option>
                                </select>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Handle non-geo caller"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option selected>"Hang Up"</option>
                                    <option>"Route to default"</option>
                                    <option>"Prompt for zip code"</option>
                                </select>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Multiple matches"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option selected>"Simultaneous (Ring All)"</option>
                                    <option>"Sequential (Round Robin)"</option>
                                    <option>"Random"</option>
                                </select>
                            </div>
                        </div>
                    </div>
                    // Routing card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Routing"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"How will you be routing?"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option selected>"Zip Code"</option>
                                    <option>"Area Code"</option>
                                    <option>"State"</option>
                                    <option>"Country"</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Agent Scripts page
// ---------------------------------------------------------------------------

#[component]
pub fn AgentScriptsPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Agent Scripts"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label">
                                    <span class="label-text font-medium">"Name"</span>
                                    <span class="text-red-500 ml-1">"*"</span>
                                </label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter script name" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span><span class="label-text-alt text-gray-400">"Optional"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this script"></textarea>
                            </div>
                            <div class="mt-3">
                                <div class="collapse collapse-arrow border border-gray-200 rounded-lg bg-gray-50">
                                    <input type="checkbox" />
                                    <div class="collapse-title text-sm font-medium">"Advanced scripting options"</div>
                                    <div class="collapse-content text-sm text-gray-500">
                                        <p>"Configure advanced script behaviors and conditional logic."</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    // Contents card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Contents"</h2>
                            <div class="mt-4 border border-gray-200 rounded-lg overflow-hidden">
                                <div class="flex items-center gap-1 bg-gray-50 border-b border-gray-200 px-3 py-2">
                                    <button class="btn btn-xs btn-ghost font-bold">"B"</button>
                                    <button class="btn btn-xs btn-ghost italic">"I"</button>
                                    <button class="btn btn-xs btn-ghost underline">"U"</button>
                                    <div class="divider divider-horizontal mx-0 h-4"></div>
                                    <button class="btn btn-xs btn-ghost">
                                        <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsLink45deg /></span>
                                    </button>
                                </div>
                                <textarea class="w-full p-3 min-h-[200px] text-sm resize-y border-none focus:outline-none" placeholder="Script Markup. Add script to show your agents when an activity is connected."></textarea>
                            </div>
                            <div class="mt-3">
                                <div role="tablist" class="tabs tabs-bordered">
                                    <a role="tab" class="tab tab-active">"Code"</a>
                                    <a role="tab" class="tab">"Input Fields"</a>
                                    <a role="tab" class="tab">"Output Fields"</a>
                                </div>
                            </div>
                        </div>
                    </div>
                    // Workflow card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Workflow"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"When a user completes a call script which panel should we load next"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"None"</option>
                                    <option>"Next script panel"</option>
                                    <option>"Summary panel"</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Routing Tables page
// ---------------------------------------------------------------------------

#[component]
pub fn RoutingTablesPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Routing Tables"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Name"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter routing table name" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span><span class="label-text-alt text-gray-400">"Optional"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this routing table"></textarea>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Default Route"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option selected>"A default route if no matches found in your table"</option>
                                    <option>"Hang Up"</option>
                                    <option>"Voicemail"</option>
                                </select>
                                <label class="label"><span class="label-text-alt text-gray-400">"If no matches are found in your table - contacts will be routed here."</span></label>
                            </div>
                            <div class="mt-4">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                            <div class="mt-3 bg-blue-50 border border-blue-200 rounded-lg px-4 py-3">
                                <p class="text-sm text-blue-700">"Save changes to add mappings"</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Voicemails page
// ---------------------------------------------------------------------------

#[component]
pub fn VoicemailsPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Voicemail"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            // Info banner
            <div class="bg-blue-50 border-b border-blue-200 px-4 py-3 flex items-start gap-3 flex-shrink-0">
                <span class="w-5 h-5 inline-flex text-blue-500 mt-0.5"><Icon icon=icondata::BsInfoCircleFill /></span>
                <div class="flex-1 text-sm text-blue-700">
                    <span class="font-semibold">"What's New: "</span>
                    <span>"Notifications now support multiple recipients. Add additional email addresses or user groups to receive voicemail notifications."</span>
                </div>
                <button class="btn btn-xs btn-ghost text-gray-400">
                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsXLg /></span>
                </button>
            </div>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label">
                                    <span class="label-text font-medium">"Name"</span>
                                    <span class="text-red-500 ml-1">"*"</span>
                                </label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter voicemail name" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Tags"</span></label>
                                <div class="flex items-center gap-2">
                                    <span class="badge bg-gray-100 text-gray-700 border-gray-300 gap-1">
                                        "voicemail"
                                        <button class="btn btn-xs btn-ghost btn-circle text-gray-400">"x"</button>
                                    </span>
                                    <input type="text" class="input input-bordered input-xs flex-1" placeholder="Add tag..." />
                                </div>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Greeting (TTS)"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" value="Please leave a message after the beep" />
                                <div class="flex items-center gap-3 mt-2">
                                    <div class="flex items-center gap-1">
                                        <span class="text-xs text-gray-500">"Language:"</span>
                                        <select class="select select-bordered select-xs">
                                            <option selected>"English"</option>
                                            <option>"Spanish"</option>
                                            <option>"French"</option>
                                        </select>
                                    </div>
                                    <div class="flex items-center gap-1">
                                        <span class="text-xs text-gray-500">"Voice:"</span>
                                        <select class="select select-bordered select-xs">
                                            <option selected>"Awesome"</option>
                                            <option>"Professional"</option>
                                            <option>"Friendly"</option>
                                        </select>
                                    </div>
                                </div>
                            </div>
                            <div class="divider my-3"></div>
                            <div class="flex items-center justify-between">
                                <div>
                                    <span class="text-sm font-medium">"Email"</span>
                                    <p class="text-xs text-gray-400">"Send voicemail recording via email"</p>
                                </div>
                                <input type="checkbox" class="toggle toggle-sm toggle-info" checked />
                            </div>
                            <div class="flex items-center justify-between mt-3">
                                <div>
                                    <span class="text-sm font-medium">"Transcribe"</span>
                                    <p class="text-xs text-gray-400">"Transcribe voicemail audio to text"</p>
                                </div>
                                <input type="checkbox" class="toggle toggle-sm toggle-info" />
                            </div>
                        </div>
                    </div>
                    // Notifications card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Notifications"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"User Emails"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"Select users to notify..."</option>
                                </select>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"User Groups"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"Select user groups..."</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Keyword Spotting page
// ---------------------------------------------------------------------------

#[component]
pub fn KeywordSpottingPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Keyword Spotting"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Name"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter keyword spotting name" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span><span class="label-text-alt text-gray-400">"Optional"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this keyword spotter"></textarea>
                            </div>
                            <div class="flex items-center gap-2 mt-4">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                                <button class="btn btn-sm btn-ghost text-gray-500">"Copy Section"</button>
                            </div>
                        </div>
                    </div>
                    // Workflow card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Workflow"</h2>
                            <div class="mt-4 space-y-4">
                                <div class="bg-gray-50 rounded-lg p-4">
                                    <p class="text-sm font-medium text-gray-700">"If any of the following keywords are found:"</p>
                                    <p class="text-sm text-gray-400 mt-1">"No keywords added."</p>
                                    <button class="btn btn-xs bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none mt-2">"+ Add Keyword"</button>
                                </div>
                                <div class="bg-gray-50 rounded-lg p-4">
                                    <p class="text-sm font-medium text-gray-700">"Then perform the following actions:"</p>
                                    <p class="text-sm text-gray-400 mt-1">"No actions added."</p>
                                    <button class="btn btn-xs bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none mt-2">"Add Action"</button>
                                </div>
                                <div class="bg-yellow-50 border border-yellow-200 rounded-lg px-4 py-3">
                                    <p class="text-xs text-yellow-700">"NOTE: Keyword triggers fire once per activity when any keyword in this section is detected."</p>
                                </div>
                            </div>
                        </div>
                    </div>
                    // Spot Activity Types card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Spot Activity Types"</h2>
                            <div class="mt-4 space-y-3">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <span class="text-sm font-medium">"Calls"</span>
                                        <p class="text-xs text-gray-400">"Spot keywords in phone calls"</p>
                                    </div>
                                    <input type="checkbox" class="toggle toggle-sm toggle-info" checked />
                                </div>
                                <div class="flex items-center justify-between">
                                    <div>
                                        <span class="text-sm font-medium">"Chats"</span>
                                        <p class="text-xs text-gray-400">"Spot keywords in chat messages"</p>
                                    </div>
                                    <input type="checkbox" class="toggle toggle-sm toggle-info" />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Lambdas page
// ---------------------------------------------------------------------------

#[component]
pub fn LambdasPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Lambdas"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label">
                                    <span class="label-text font-medium">"Name"</span>
                                    <span class="text-red-500 ml-1">"*"</span>
                                </label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter lambda name" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this lambda function"></textarea>
                            </div>
                        </div>
                    </div>
                    // Code card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Code"</h2>
                            <div class="mt-4">
                                <textarea class="w-full bg-gray-900 text-green-400 font-mono text-sm p-4 rounded-lg min-h-[300px] resize-y" placeholder="// Write your JavaScript function here
exports.handler = async (event, context) => {
  const { caller, callee, callData } = event;

  // Your custom logic here
  return {
    statusCode: 200,
    body: JSON.stringify({ action: 'continue' })
  };
};"></textarea>
                            </div>
                            <div class="mt-3">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save & Test"</button>
                            </div>
                        </div>
                    </div>
                    // Trigger card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Trigger"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Select event"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"Select an event..."</option>
                                    <option>"On call start"</option>
                                    <option>"On call end"</option>
                                    <option>"On call answered"</option>
                                    <option>"On form submission"</option>
                                    <option>"On chat message"</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// API Logs page
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct ApiLogEntry {
    source: &'static str,
    request_url: &'static str,
    response_code: u16,
    date: &'static str,
    activity: &'static str,
}

fn mock_api_logs() -> Vec<ApiLogEntry> {
    vec![
        ApiLogEntry { source: "Webhook: 4iiz AI Outbound SMS", request_url: "POST https://gfphuh6fxq...lambda-url.us-east-1.on.aws/", response_code: 200, date: "2026-02-24 10:42 AM", activity: "Call #284719" },
        ApiLogEntry { source: "Webhook: #Na Calls to Tickets", request_url: "POST https://hooks.zapier.com/hooks/catch/123...", response_code: 200, date: "2026-02-24 10:38 AM", activity: "Call #284718" },
        ApiLogEntry { source: "Trigger: FKM Tagging - Outbound", request_url: "POST https://api.4iiz.com/v1/triggers/execute", response_code: 200, date: "2026-02-24 10:35 AM", activity: "Call #284717" },
        ApiLogEntry { source: "Webhook: AnswerHero", request_url: "POST https://hooks.zapier.com/hooks/catch/456...", response_code: 404, date: "2026-02-24 10:30 AM", activity: "Call #284716" },
        ApiLogEntry { source: "Webhook: Offline-Comms-test1", request_url: "POST https://abc123.ngrok.io/webhook", response_code: 404, date: "2026-02-24 10:25 AM", activity: "Call #284715" },
        ApiLogEntry { source: "Webhook: 4iiz AI SMS hook", request_url: "POST https://xyz789.lambda-url.us-east-1.on.aws/", response_code: 200, date: "2026-02-24 10:20 AM", activity: "Text #91024" },
    ]
}

#[component]
pub fn ApiLogsPage() -> impl IntoView {
    let logs = mock_api_logs();

    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Settings"</span></li>
                        <li><span class="text-gray-500">"Integrations"</span></li>
                        <li><span class="text-gray-500">"Webhooks"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"API Logs"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
            </header>
            // Filters row
            <div class="bg-white border-b border-gray-200 px-4 py-3 flex items-center gap-3 flex-shrink-0 flex-wrap">
                <select class="select select-bordered select-sm">
                    <option>"Status Code"</option>
                    <option>"200"</option>
                    <option>"404"</option>
                    <option>"500"</option>
                </select>
                <input type="text" class="input input-bordered input-sm w-48" placeholder="Source" />
                <select class="select select-bordered select-sm">
                    <option>"Select Call"</option>
                </select>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Search"</button>
                <button class="btn btn-sm btn-ghost text-gray-500">"Reset"</button>
            </div>
            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
                    <div class="grid grid-cols-[32px_220px_1fr_100px_160px_120px_60px] gap-1 px-4 py-2 border-b border-gray-200 min-w-max">
                        <div class="col-header"></div>
                        <div class="col-header">"Source"</div>
                        <div class="col-header">"Request URL"</div>
                        <div class="col-header">"Response Code"</div>
                        <div class="col-header">"Date"</div>
                        <div class="col-header">"Activity"</div>
                        <div class="col-header"></div>
                    </div>

                    {logs.into_iter().map(|l| {
                        let badge_class = if l.response_code == 200 {
                            "badge badge-sm bg-green-500 text-white border-none"
                        } else {
                            "badge badge-sm bg-red-500 text-white border-none"
                        };
                        view! {
                            <div class="activity-row grid grid-cols-[32px_220px_1fr_100px_160px_120px_60px] gap-1 px-4 py-2.5 items-center cursor-pointer min-w-max">
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronDown /></span>
                                </button>
                                <div class="text-sm text-gray-700">{l.source}</div>
                                <div class="text-xs text-gray-400 truncate">{l.request_url}</div>
                                <div><span class=badge_class>{l.response_code}</span></div>
                                <div class="text-xs text-gray-500">{l.date}</div>
                                <div class="text-xs text-iiz-blue-link">{l.activity}</div>
                                <div>
                                    <button class="btn btn-xs btn-ghost text-iiz-cyan">"Retry"</button>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex items-center justify-center mt-4">
                    <button class="btn btn-sm btn-ghost text-iiz-cyan">"Load More"</button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Global page
// ---------------------------------------------------------------------------

#[component]
pub fn GlobalPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex-shrink-0">
                <h1 class="text-xl font-semibold text-iiz-dark">"Global Settings"</h1>
                <p class="text-sm text-gray-500">"Configure account-wide automation settings and global variables"</p>
            </div>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // Account Variables card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Account Variables"</h2>
                            <div class="mt-4 overflow-x-auto">
                                <table class="table table-sm">
                                    <thead>
                                        <tr>
                                            <th class="text-xs text-gray-500 uppercase">"Key"</th>
                                            <th class="text-xs text-gray-500 uppercase">"Value"</th>
                                            <th class="w-16"></th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <tr>
                                            <td class="font-mono text-sm">"COMPANY_NAME"</td>
                                            <td class="text-sm text-gray-600">"Diener Law Group"</td>
                                            <td>
                                                <button class="btn btn-xs btn-ghost text-gray-400">
                                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                                </button>
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="font-mono text-sm">"SUPPORT_EMAIL"</td>
                                            <td class="text-sm text-gray-600">"support@dienerlaw.com"</td>
                                            <td>
                                                <button class="btn btn-xs btn-ghost text-gray-400">
                                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                                </button>
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="font-mono text-sm">"DEFAULT_TIMEZONE"</td>
                                            <td class="text-sm text-gray-600">"America/New_York"</td>
                                            <td>
                                                <button class="btn btn-xs btn-ghost text-gray-400">
                                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                                </button>
                                            </td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                    // Global Webhooks card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Global Webhooks"</h2>
                            <div class="mt-4 text-center py-8">
                                <p class="text-sm text-gray-400">"No global webhooks configured."</p>
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none mt-3">"Add Webhook"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Workflows page
// ---------------------------------------------------------------------------

#[component]
pub fn WorkflowsPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            // Top bar
            <div class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Workflows"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm btn-ghost text-gray-500">"Revisions"</button>
                <button class="btn btn-sm btn-ghost text-gray-500">"Feedback"</button>
            </div>
            // Content: left panel + canvas
            <div class="flex-1 flex overflow-hidden">
                // Left panel - events
                <div class="w-64 bg-white border-r border-gray-200 overflow-y-auto flex-shrink-0">
                    <div class="p-4">
                        <h2 class="text-sm font-semibold text-gray-700 mb-3">"Workflow Events"</h2>

                        <div class="mb-4">
                            <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">"Activity Events"</h3>
                            <div class="space-y-1">
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Call Started"</div>
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Call Answered"</div>
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Call Ended"</div>
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Text Received"</div>
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Form Submitted"</div>
                            </div>
                        </div>

                        <div class="mb-4">
                            <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">"Completion Events"</h3>
                            <div class="space-y-1">
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Call Completed"</div>
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Voicemail Left"</div>
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Chat Ended"</div>
                            </div>
                        </div>

                        <div class="mb-4">
                            <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">"Conversion Events"</h3>
                            <div class="space-y-1">
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Lead Converted"</div>
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Appointment Set"</div>
                            </div>
                        </div>

                        <div class="mb-4">
                            <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">"Other Events"</h3>
                            <div class="space-y-1">
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Tag Added"</div>
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Score Changed"</div>
                                <div class="text-sm text-gray-600 px-2 py-1 hover:bg-gray-50 rounded cursor-pointer">"Contact Updated"</div>
                            </div>
                        </div>
                    </div>
                </div>
                // Canvas area
                <div class="flex-1 bg-gray-100 flex items-center justify-center">
                    <div class="text-center">
                        <span class="w-16 h-16 inline-flex text-gray-300 mx-auto mb-4">
                            <Icon icon=icondata::BsDiagram3Fill />
                        </span>
                        <p class="text-gray-500 text-sm mb-3">"Click to add a workflow trigger"</p>
                        <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"+ Add Trigger"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// LeadReactor page
// ---------------------------------------------------------------------------

#[component]
pub fn LeadReactorPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"LeadReactor"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Name"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter lead reactor name" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this lead reactor"></textarea>
                            </div>
                        </div>
                    </div>
                    // Response card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Response"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"How to respond"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option selected>"Call"</option>
                                    <option>"Text"</option>
                                    <option>"Email"</option>
                                </select>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Response delay"</span></label>
                                <div class="flex items-center gap-2">
                                    <input type="number" class="input input-bordered input-sm w-24" value="0" />
                                    <span class="text-sm text-gray-500">"seconds"</span>
                                </div>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Message template"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=3 placeholder="Enter the message to send when responding to a new lead..."></textarea>
                            </div>
                        </div>
                    </div>
                    // Tracking card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Tracking"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Assign to tracking number"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"Select a tracking number..."</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Smart Dialers page
// ---------------------------------------------------------------------------

#[component]
pub fn SmartDialersPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Smart Dialers"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Name"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter dialer campaign name" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this dialer campaign"></textarea>
                            </div>
                        </div>
                    </div>
                    // Dialing card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Dialing"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Dial mode"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"Preview"</option>
                                    <option selected>"Progressive"</option>
                                    <option>"Predictive"</option>
                                </select>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Calls per agent"</span></label>
                                <input type="number" class="input input-bordered input-sm w-24" value="1" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Retry attempts"</span></label>
                                <input type="number" class="input input-bordered input-sm w-24" value="3" />
                            </div>
                        </div>
                    </div>
                    // Contact List card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Contact List"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Select list"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"Select a contact list..."</option>
                                </select>
                            </div>
                            <div class="mt-3 text-center py-6">
                                <p class="text-sm text-gray-400">"No list selected"</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Chat Widget page
// ---------------------------------------------------------------------------

// (ChatWidgetRow mock struct removed — now using ChatWidgetItem from API types.)

#[component]
pub fn ChatWidgetPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ChatWidgetItem>>("/flows/chat-widgets?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            // Title bar
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <h1 class="text-xl font-semibold text-iiz-dark">"Chat Widget"</h1>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Chat Widgets", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <button class="btn btn-xs btn-ghost text-gray-500">"User licenses"</button>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Chat Widget"
                </button>
            </div>

            <div class="flex-1 overflow-y-auto">
                // KPI summary cards (static — needs aggregation queries later)
                <div class="grid grid-cols-4 gap-3 p-4">
                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Active Widgets"</div>
                        <div class="text-2xl font-bold text-green-600 mt-1">"--"</div>
                        <div class="text-xs text-gray-400">"of -- total"</div>
                    </div>
                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Total Chats"</div>
                        <div class="text-2xl font-bold text-iiz-dark mt-1">"--"</div>
                        <div class="text-xs text-gray-400">"Aggregation pending"</div>
                    </div>
                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Avg Response Time"</div>
                        <div class="text-2xl font-bold text-iiz-dark mt-1">"--"</div>
                        <div class="text-xs text-gray-400">"Seconds"</div>
                    </div>
                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Satisfaction"</div>
                        <div class="text-2xl font-bold text-iiz-cyan mt-1">"--"</div>
                        <div class="text-xs text-gray-400">"Aggregation pending"</div>
                    </div>
                </div>

                // Widget config table
                <div class="px-4 pb-4">
                    <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
                        <table class="table table-sm w-full">
                            <thead>
                                <tr class="border-b border-gray-200">
                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Chat Widget"</th>
                                    <th class="text-xs text-gray-500 font-semibold uppercase text-center">"Fields"</th>
                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Status"</th>
                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Routing"</th>
                                    <th class="text-xs text-gray-500 font-semibold uppercase text-center">"Agents"</th>
                                    <th class="text-xs text-gray-500 font-semibold uppercase text-center">"Chats"</th>
                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Updated"</th>
                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Created"</th>
                                    <th class="text-xs text-gray-500 font-semibold uppercase text-center">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || match data.get() {
                                    None => view! {
                                        <tr>
                                            <td colspan="9" class="text-center py-4">
                                                <span class="loading loading-spinner loading-md text-iiz-cyan"></span>
                                                <span class="ml-2 text-gray-500">"Loading..."</span>
                                            </td>
                                        </tr>
                                    }.into_any(),
                                    Some(Err(e)) => view! {
                                        <tr>
                                            <td colspan="9" class="text-center py-4 text-red-500 text-sm">{e}</td>
                                        </tr>
                                    }.into_any(),
                                    Some(Ok(resp)) => {
                                        let items = resp.items.clone();
                                        view! {
                                            <>
                                                {items.into_iter().map(|w| {
                                                    let routing = w.routing_type.clone().unwrap_or_default();
                                                    let status_class = match w.status.as_str() {
                                                        "active" | "live" => "badge badge-sm badge-success",
                                                        "draft" => "badge badge-sm badge-warning",
                                                        _ => "badge badge-sm bg-gray-400 text-white border-none",
                                                    };
                                                    view! {
                                                        <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                            <td class="text-sm font-medium text-iiz-dark">{w.name.clone()}</td>
                                                            <td class="text-sm text-center">{w.custom_fields_count.to_string()}</td>
                                                            <td><span class=status_class>{w.status.clone()}</span></td>
                                                            <td class="text-sm text-gray-600">{routing}</td>
                                                            <td class="text-sm text-center">{w.agent_count.to_string()}</td>
                                                            <td class="text-sm text-center font-medium">{w.chat_count.to_string()}</td>
                                                            <td class="text-xs text-gray-500">{fmt_date(&w.updated_at)}</td>
                                                            <td class="text-xs text-gray-500">{fmt_date(&w.created_at)}</td>
                                                            <td class="text-center">
                                                                <div class="flex items-center justify-center gap-1">
                                                                    <button class="btn btn-xs btn-ghost text-iiz-cyan">"Edit"</button>
                                                                    <button class="btn btn-xs btn-ghost text-gray-400">"Code"</button>
                                                                </div>
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

                // Widget preview + embed code section
                <div class="grid grid-cols-2 gap-4 px-4 pb-4">
                    // Widget preview
                    <div class="bg-white rounded-lg border border-gray-200 p-4">
                        <h3 class="text-sm font-semibold text-iiz-dark mb-3">"Widget Preview"</h3>
                        <div class="border border-gray-200 rounded-lg p-4 bg-gray-50">
                            // Mini chat widget mockup
                            <div class="flex justify-end">
                                <div class="w-72">
                                    // Chat header
                                    <div class="bg-iiz-cyan text-white rounded-t-lg px-4 py-3 flex items-center gap-2">
                                        <div class="w-8 h-8 bg-white/20 rounded-full flex items-center justify-center">
                                            <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsChatDots /></span>
                                        </div>
                                        <div>
                                            <div class="text-sm font-semibold">"Diener Law"</div>
                                            <div class="text-xs opacity-80">"We typically reply in minutes"</div>
                                        </div>
                                    </div>
                                    // Chat body
                                    <div class="bg-white border-x border-gray-200 p-3 space-y-2" style="min-height: 120px;">
                                        <div class="flex gap-2">
                                            <div class="w-6 h-6 bg-iiz-cyan/20 rounded-full flex items-center justify-center flex-shrink-0">
                                                <span class="text-xs text-iiz-cyan">"D"</span>
                                            </div>
                                            <div class="bg-gray-100 rounded-lg px-3 py-2 text-sm text-gray-700 max-w-[200px]">
                                                "Hi! How can we help you today?"
                                            </div>
                                        </div>
                                    </div>
                                    // Chat input
                                    <div class="bg-white border border-gray-200 rounded-b-lg px-3 py-2 flex items-center gap-2">
                                        <input type="text" placeholder="Type a message..." class="text-sm flex-1 outline-none" />
                                        <button class="w-6 h-6 bg-iiz-cyan rounded-full flex items-center justify-center">
                                            <span class="w-3 h-3 inline-flex text-white"><Icon icon=icondata::BsSendFill /></span>
                                        </button>
                                    </div>
                                </div>
                            </div>
                            // Floating button
                            <div class="flex justify-end mt-3">
                                <div class="w-14 h-14 bg-iiz-cyan rounded-full flex items-center justify-center shadow-lg cursor-pointer">
                                    <span class="w-6 h-6 inline-flex text-white"><Icon icon=icondata::BsChatDotsFill /></span>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Embed code
                    <div class="bg-white rounded-lg border border-gray-200 p-4">
                        <h3 class="text-sm font-semibold text-iiz-dark mb-3">"Embed Code"</h3>
                        <p class="text-xs text-gray-500 mb-3">"Add this code to your website before the closing &lt;/body&gt; tag."</p>
                        <div class="bg-gray-900 rounded-lg p-4 font-mono text-xs text-green-400 overflow-x-auto">
                            <pre class="whitespace-pre-wrap">
                                {"<script>\n  (function(w,d,s,c){\n    var f=d.getElementsByTagName(s)[0];\n    var j=d.createElement(s);\n    j.async=true;\n    j.src='https://chat.4iiz.com/widget/'+c+'.js';\n    f.parentNode.insertBefore(j,f);\n  })(window,document,'script','wgt_abc123');\n</script>"}
                            </pre>
                        </div>
                        <div class="flex gap-2 mt-3">
                            <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                                <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsClipboard /></span>
                                "Copy Code"
                            </button>
                            <button class="btn btn-sm btn-ghost">
                                <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsEnvelope /></span>
                                "Email to Developer"
                            </button>
                        </div>
                        <div class="mt-4 p-3 bg-blue-50 rounded-lg border border-blue-100">
                            <h4 class="text-xs font-semibold text-blue-800 mb-1">"Installation Options"</h4>
                            <ul class="text-xs text-blue-700 space-y-1">
                                <li>"- Direct HTML embed (shown above)"</li>
                                <li>"- Google Tag Manager container"</li>
                                <li>"- WordPress plugin"</li>
                                <li>"- React / Vue component"</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// ChatAI page (Flows section, BETA)
// ---------------------------------------------------------------------------

#[component]
pub fn FlowsChatAIPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"ChatAI's"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <span class="badge badge-sm bg-iiz-cyan text-white border-none">"BETA"</span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label">
                                    <span class="label-text font-medium">"Name"</span>
                                </label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter ChatAI name" />
                                <label class="label"><span class="label-text-alt text-gray-400">"This name is user facing"</span></label>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this ChatAI"></textarea>
                            </div>
                        </div>
                    </div>
                    // Knowledge Banks card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Knowledge Banks"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Choose a Knowledge Bank"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"Select a knowledge bank..."</option>
                                </select>
                            </div>
                            <div class="flex items-center justify-between mt-3">
                                <div>
                                    <span class="text-sm font-medium">"Include Source"</span>
                                    <p class="text-xs text-gray-400">"Include source references in AI responses"</p>
                                </div>
                                <input type="checkbox" class="toggle toggle-sm toggle-info" checked />
                            </div>
                            <div class="mt-3">
                                <button class="btn btn-sm btn-ghost text-iiz-cyan">"Manage Knowledge Banks"</button>
                            </div>
                        </div>
                    </div>
                    // Instructions card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Instructions"</h2>
                            <div class="form-control mt-4">
                                <textarea class="textarea textarea-bordered w-full min-h-[200px]" placeholder="Enter instructions for the AI to follow when responding to chat messages. Be specific about tone, topics to cover, and any limitations."></textarea>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Dialogflow page (BETA)
// ---------------------------------------------------------------------------

#[component]
pub fn DialogflowPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Dialogflow"</span></li>
                        <li><span class="text-gray-500">"New Dialogflow Agent"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"Pre-requisites"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <span class="badge badge-sm bg-iiz-cyan text-white border-none">"BETA"</span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // Pre-requisites card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Pre-requisites"</h2>
                            <ol class="mt-4 space-y-3 list-decimal list-inside text-sm text-gray-700">
                                <li>"Create a Google Cloud Platform (GCP) project with billing enabled."</li>
                                <li>"Enable the Dialogflow API in your GCP project."</li>
                                <li>"Create a Dialogflow CX agent or ES agent in the Google Cloud Console."</li>
                                <li>"Create a Conversation Profile linked to your Dialogflow agent."</li>
                                <li>"Generate a service account key (JSON) and upload it to 4iiz for authentication."</li>
                            </ol>
                        </div>
                    </div>
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Name"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter a unique name" />
                                <label class="label"><span class="label-text-alt text-gray-400">"Must be unique across all Dialogflow agents"</span></label>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this Dialogflow agent"></textarea>
                            </div>
                        </div>
                    </div>
                    // Google Cloud Configuration card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Google Cloud Configuration"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Project ID"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="my-gcp-project-id" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Conversation Profile ID"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter conversation profile ID" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Agent Location"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option selected>"Select a location"</option>
                                    <option>"us-central1"</option>
                                    <option>"us-east1"</option>
                                    <option>"europe-west1"</option>
                                    <option>"asia-east1"</option>
                                    <option>"global"</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Reminders page
// ---------------------------------------------------------------------------

#[component]
pub fn RemindersPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Reminders"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"New"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Reminder Settings"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"My Reminders"</a>
            </header>
            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Name"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter reminder name" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Description"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=2 placeholder="Describe this reminder"></textarea>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Timezone"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option selected>"(GMT-05:00) Eastern Time"</option>
                                    <option>"(GMT-06:00) Central Time"</option>
                                    <option>"(GMT-07:00) Mountain Time"</option>
                                    <option>"(GMT-08:00) Pacific Time"</option>
                                </select>
                            </div>
                        </div>
                    </div>
                    // Scheduling card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Scheduling"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"Remind at"</span></label>
                                <input type="datetime-local" class="input input-bordered input-sm w-full" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label cursor-pointer justify-start gap-2">
                                    <input type="checkbox" class="checkbox checkbox-sm" />
                                    <span class="label-text font-medium">"Recurring"</span>
                                </label>
                            </div>
                        </div>
                    </div>
                    // Who To Invite card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Who To Invite"</h2>
                            <div class="mt-4 flex items-center gap-1">
                                <button class="btn btn-sm bg-iiz-cyan text-white border-none">"Recent Calls"</button>
                                <button class="btn btn-sm btn-ghost text-gray-500">"Contact List"</button>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Choose a Contact"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"Select a contact..."</option>
                                </select>
                            </div>
                        </div>
                    </div>
                    // Getting Connected card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Getting Connected"</h2>
                            <div class="form-control mt-4">
                                <label class="label"><span class="label-text font-medium">"How to Remind"</span></label>
                                <select class="select select-bordered select-sm w-full">
                                    <option>"Call"</option>
                                    <option>"Text"</option>
                                    <option>"Email"</option>
                                </select>
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Who to Remind"</span></label>
                                <input type="text" class="input input-bordered input-sm w-full" placeholder="Enter phone number or email" />
                            </div>
                            <div class="form-control mt-3">
                                <label class="label"><span class="label-text font-medium">"Message"</span></label>
                                <textarea class="textarea textarea-bordered textarea-sm w-full" rows=3 placeholder="Enter reminder message..."></textarea>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
