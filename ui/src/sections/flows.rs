use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

use crate::api::api_get;
use crate::api::types::{
    AgentScriptItem, ApiLogEntryItem, BulkMessageItem, ChatWidgetItem, DialogflowItem,
    FormReactorItem, GeoRouterItem, KeywordSpottingItem, LambdaItem, LeadReactorItem,
    ListResponse, PaginationMeta, QueueItem, ReminderItem, RoutingTableItem, ScheduleItem,
    SmartDialerItem, SmartRouterItem, TriggerItem, VoicemailBoxItem, VoiceMenuItem, WebhookItem,
    WorkflowItem,
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
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<GeoRouterItem>>("/flows/geo-routers?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Geo Routers"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Geo Router"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Description"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td>
                                                                    {if item.is_active {
                                                                        view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-500 border-gray-200">"Inactive"</span> }.into_any()
                                                                    }}
                                                                </td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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

// ---------------------------------------------------------------------------
// Agent Scripts page
// ---------------------------------------------------------------------------

#[component]
pub fn AgentScriptsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<AgentScriptItem>>("/flows/agent-scripts?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Agent Scripts"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Script"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Description"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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

// ---------------------------------------------------------------------------
// Routing Tables page
// ---------------------------------------------------------------------------

#[component]
pub fn RoutingTablesPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<RoutingTableItem>>("/flows/routing-tables?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Routing Tables"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Routing Table"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Description"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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

// ---------------------------------------------------------------------------
// Voicemails page
// ---------------------------------------------------------------------------

#[component]
pub fn VoicemailsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<VoicemailBoxItem>>("/flows/voicemails?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Voicemail Boxes"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Voicemail Box"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Greeting Type"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.greeting_type.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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

// ---------------------------------------------------------------------------
// Keyword Spotting page
// ---------------------------------------------------------------------------

#[component]
pub fn KeywordSpottingPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<KeywordSpottingItem>>("/flows/keyword-spotting?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Keyword Spotting"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Config"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Description"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td>
                                                                    {if item.is_active {
                                                                        view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-500 border-gray-200">"Inactive"</span> }.into_any()
                                                                    }}
                                                                </td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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

// ---------------------------------------------------------------------------
// Lambdas page
// ---------------------------------------------------------------------------

#[component]
pub fn LambdasPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<LambdaItem>>("/flows/lambdas?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Lambdas"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Lambda"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Runtime"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.runtime.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td class="text-sm text-gray-600">{item.status.clone()}</td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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

// ---------------------------------------------------------------------------
// API Logs page
// ---------------------------------------------------------------------------

#[component]
pub fn ApiLogsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ApiLogEntryItem>>("/activities/api-logs?page=1&per_page=25").await
    });

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
                {move || match data.get() {
                    None => loading_view().into_any(),
                    Some(Err(e)) => error_view(e).into_any(),
                    Some(Ok(resp)) => {
                        let items = resp.items.clone();
                        let meta = resp.pagination.clone();
                        view! {
                            <>
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

                                    {items.into_iter().map(|l| {
                                        let source = l.source.clone().unwrap_or_default();
                                        let request_url = format!("{} {}", l.method, l.endpoint);
                                        let response_code = l.response_code.unwrap_or(0);
                                        let activity = l.activity_description.clone().unwrap_or_default();
                                        let date = fmt_date(&l.timestamp);
                                        let time = fmt_time(&l.timestamp);
                                        let date_str = format!("{} {}", date, time);
                                        let badge_class = if response_code >= 200 && response_code < 300 {
                                            "badge badge-sm bg-green-500 text-white border-none"
                                        } else {
                                            "badge badge-sm bg-red-500 text-white border-none"
                                        };
                                        view! {
                                            <div class="activity-row grid grid-cols-[32px_220px_1fr_100px_160px_120px_60px] gap-1 px-4 py-2.5 items-center cursor-pointer min-w-max">
                                                <button class="btn btn-xs btn-ghost text-gray-400">
                                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronDown /></span>
                                                </button>
                                                <div class="text-sm text-gray-700">{source}</div>
                                                <div class="text-xs text-gray-400 truncate">{request_url}</div>
                                                <div><span class=badge_class>{response_code}</span></div>
                                                <div class="text-xs text-gray-500">{date_str}</div>
                                                <div class="text-xs text-iiz-blue-link">{activity}</div>
                                                <div>
                                                    <button class="btn btn-xs btn-ghost text-iiz-cyan">"Retry"</button>
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
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<WorkflowItem>>("/flows/workflows?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Workflows"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Workflow"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Description"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td>
                                                                    {if item.is_active {
                                                                        view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-500 border-gray-200">"Inactive"</span> }.into_any()
                                                                    }}
                                                                </td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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

// ---------------------------------------------------------------------------
// LeadReactor page
// ---------------------------------------------------------------------------

#[component]
pub fn LeadReactorPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<LeadReactorItem>>("/flows/lead-reactor?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Lead Reactor"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Config"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Description"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td>
                                                                    {if item.is_active {
                                                                        view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-500 border-gray-200">"Inactive"</span> }.into_any()
                                                                    }}
                                                                </td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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

// ---------------------------------------------------------------------------
// Smart Dialers page
// ---------------------------------------------------------------------------

#[component]
pub fn SmartDialersPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<SmartDialerItem>>("/flows/smart-dialers?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Smart Dialers"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Smart Dialer"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Description"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td>
                                                                    {if item.is_active {
                                                                        view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-500 border-gray-200">"Inactive"</span> }.into_any()
                                                                    }}
                                                                </td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<DialogflowItem>>("/flows/dialogflow?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Dialogflow"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Config"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Project ID"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.project_id.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td>
                                                                    {if item.is_active {
                                                                        view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-500 border-gray-200">"Inactive"</span> }.into_any()
                                                                    }}
                                                                </td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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

// ---------------------------------------------------------------------------
// Reminders page
// ---------------------------------------------------------------------------

#[component]
pub fn RemindersPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ReminderItem>>("/flows/reminders?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Reminders"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Reminder"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Description"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
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
                                                    {items.into_iter().map(|item| {
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                                <td class="text-sm font-medium text-iiz-cyan">{item.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "\u{2014}".to_string())}</td>
                                                                <td>
                                                                    {if item.is_active {
                                                                        view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-500 border-gray-200">"Inactive"</span> }.into_any()
                                                                    }}
                                                                </td>
                                                                <td class="text-sm text-gray-600">{fmt_date(&item.created_at)}</td>
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
