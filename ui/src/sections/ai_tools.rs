use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

use crate::api::api_get;
use crate::api::types::{AskAiConfigItem, ChatAiAgentItem, KnowledgeBankItem, ListResponse, PaginationMeta, VoiceAiAgentItem};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Format an ISO-8601 datetime string for display (date only).
fn fmt_date(iso: &str) -> String {
    if iso.len() >= 10 { iso[..10].to_string() } else { iso.to_string() }
}

/// Format byte count for human-readable display.
fn fmt_bytes(bytes: i64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
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

/// Map a knowledge bank category to a DaisyUI badge class.
fn category_badge(cat: &str) -> &'static str {
    match cat {
        "General" => "badge-info",
        "Support" => "badge-warning",
        "Legal" => "badge-secondary",
        "Training" => "badge-primary",
        _ => "badge-ghost",
    }
}

/// Map a knowledge bank status to a (dot-class, badge-class) pair.
fn status_classes(status: &str) -> (&'static str, &'static str) {
    match status {
        "Ready" => ("bg-green-500", "badge-success"),
        "Indexing" => ("", "badge-warning"),
        "Error" => ("bg-red-500", "badge-error"),
        _ => ("bg-gray-400", "badge-ghost"),
    }
}

// ---------------------------------------------------------------------------
// AI Tools side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn AIToolsSideNav() -> impl IntoView {
    let location = use_location();
    let active = |href: &'static str| {
        move || {
            if location.pathname.get() == href { "side-nav-item active" } else { "side-nav-item" }
        }
    };

    view! {
        <div class="px-4 pt-4 pb-2">
            <div class="flex items-center gap-2 text-iiz-cyan">
                <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsStars /></span>
                <span class="text-lg font-light">"AI Tools"</span>
            </div>
        </div>

        <nav class="px-2 pb-4">
            // AI Insights group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsDiamond /></span>
                    "AI Insights"
                </h3>
                <a href="/ai-tools/askai" class=active("/ai-tools/askai")>"AskAI"</a>
                <a href="/ai-tools/summaries" class=active("/ai-tools/summaries")>"Summaries"</a>
            </div>

            // AI Agents group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsRobot /></span>
                    "AI Agents"
                </h3>
                <a href="/ai-tools/chatai" class=active("/ai-tools/chatai")>
                    "ChatAI"
                    <span class="ml-1 text-[10px] text-gray-400 uppercase">"BETA"</span>
                </a>
                <a href="/ai-tools/voiceai" class=active("/ai-tools/voiceai")>"VoiceAI"</a>
                <a href="/ai-tools/knowledge-banks" class=active("/ai-tools/knowledge-banks")>
                    "Knowledge Banks"
                    <span class="ml-1 text-[10px] text-gray-400 uppercase">"BETA"</span>
                </a>
            </div>
        </nav>
    }
}

// ---------------------------------------------------------------------------
// AskAI page - list of existing AskAI configurations
// ---------------------------------------------------------------------------

#[component]
pub fn AskAIPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<AskAiConfigItem>>("/ai-tools/ask-ai?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="mr-auto">
                    <h1 class="text-lg font-semibold text-gray-800">"AskAI Configurations"</h1>
                    <p class="text-xs text-gray-500">"Manage AI-powered question answering configurations"</p>
                </div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"+ New AskAI"</button>
            </header>

            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let meta = resp.pagination.clone();
                    view! {
                        <div class="flex flex-col flex-1 overflow-hidden">
                            <div class="flex-1 overflow-y-auto">
                                <table class="table table-sm w-full">
                                    <thead>
                                        <tr class="text-xs text-gray-500 uppercase">
                                            <th>"Name"</th>
                                            <th>"Provider"</th>
                                            <th>"Active"</th>
                                            <th>"Created"</th>
                                            <th>"Updated"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {resp.items.into_iter().map(|item| {
                                            view! {
                                                <tr class="hover:bg-gray-50 cursor-pointer">
                                                    <td>
                                                        <div class="font-medium text-gray-800">{item.name.clone()}</div>
                                                        <div class="text-xs text-gray-400">{item.description.clone().unwrap_or_else(|| "-".to_string())}</div>
                                                    </td>
                                                    <td class="text-sm text-gray-600">{item.model_provider.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                    <td>
                                                        {if item.is_active {
                                                            view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                        } else {
                                                            view! { <span class="badge badge-sm bg-gray-100 text-gray-500">"Inactive"</span> }.into_any()
                                                        }}
                                                    </td>
                                                    <td class="text-xs text-gray-500">{fmt_date(&item.created_at)}</td>
                                                    <td class="text-xs text-gray-500">{fmt_date(&item.updated_at)}</td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </div>
                            {pagination_footer(&meta)}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Summaries page - conversation analysis settings with toggles
// ---------------------------------------------------------------------------

#[component]
pub fn SummariesPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Account Settings"</span></li>
                        <li><span class="text-gray-500">"Conversation Analysis Setting"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"Channel Filter"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGearFill /></span>
                    "Versions"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // Info banner
                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 flex items-start gap-3">
                        <span class="w-5 h-5 inline-flex text-blue-500 flex-shrink-0 mt-0.5"><Icon icon=icondata::BsInfoCircleFill /></span>
                        <div class="flex-1">
                            <p class="text-sm text-gray-600">
                                "Enhance your insights with AskAI Summaries, which lets you select from various topic types to tailor the summaries for maximum impact."
                            </p>
                        </div>
                        <div class="flex items-center gap-3 flex-shrink-0">
                            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Close"</a>
                            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"More Info"</a>
                        </div>
                    </div>

                    // Channel Filter card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Channel Filter"</h2>
                            <p class="text-sm text-gray-500 mt-1">"What type of activities do you want to analyze?"</p>
                            <p class="text-sm text-orange-600 mt-2">"Toggle one of the options in order to select the Summary Type below."</p>

                            <div class="space-y-3 mt-4">
                                <label class="flex items-center gap-3 cursor-pointer">
                                    <input type="checkbox" class="toggle toggle-sm" />
                                    <span class="text-sm">"Analyze phone calls"</span>
                                </label>
                                <label class="flex items-center gap-3 cursor-pointer">
                                    <input type="checkbox" class="toggle toggle-sm" />
                                    <span class="text-sm">"Analyze video calls"</span>
                                </label>
                            </div>

                            <p class="text-xs text-gray-500 mt-3">
                                "Have you completed the "
                                <a class="text-iiz-cyan hover:underline cursor-pointer">"Zoom Integration"</a>
                                " which is required to summarize video calls?"
                            </p>

                            <div class="mt-4">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                        </div>
                    </div>

                    // Summary Type Selector card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <div class="flex items-center justify-between">
                                <h2 class="card-title text-lg font-semibold">"Summary Type"</h2>
                                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"We Want Your Feedback!"</a>
                            </div>
                            <p class="text-sm text-gray-500 mt-1">"Choose one or more summaries to get key information from a conversation."</p>
                            <p class="text-sm text-orange-600 mt-1">"*Note: Summaries carry an additional fee per summarized activity."</p>

                            <div class="grid grid-cols-2 gap-4 mt-4">
                                {[
                                    ("Classic Summary", "Narrative recap of the meeting from beginning to end"),
                                    ("Customer Success", "Experiences, challenges, and goals discussed on the call"),
                                    ("Key Insights", "Core conversation takeaways and insights"),
                                    ("Project Kick-Off", "Project vision, target goals, and resources"),
                                    ("Question-Answer", "Questions asked and their answers"),
                                    ("Action Items", "List of next steps based on the conversation"),
                                    ("Sales", "The prospect's needs, challenges, and buying journey"),
                                    ("Pain Points", "Issues the prospect is facing and wants to resolve"),
                                    ("Demo", "Demo overview and success rating"),
                                ].into_iter().map(|(name, desc)| {
                                    view! {
                                        <label class="flex items-start gap-3 p-3 rounded-lg border border-gray-100 hover:bg-gray-50 cursor-pointer">
                                            <input type="checkbox" class="toggle toggle-sm mt-0.5" />
                                            <div>
                                                <div class="text-sm font-medium">{name}</div>
                                                <div class="text-xs text-gray-500">{desc}</div>
                                            </div>
                                        </label>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>

                            <div class="mt-4">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                        </div>
                    </div>

                    // Summary Transcription Rules card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Summary Transcription Rules"</h2>
                            <p class="text-sm text-gray-500 mt-1">"Select which call settings you'd like to use to run AskAI summaries."</p>
                            <p class="text-xs text-gray-500 mt-2">"To analyze your conversations, please enable call transcriptions and select which activity types you'd like analyzed."</p>
                            <div class="mt-4">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Edit Assigned Call Settings"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Knowledge Banks page - API-driven data table
// ---------------------------------------------------------------------------

#[component]
pub fn KnowledgeBanksPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<KnowledgeBankItem>>("/ai-tools/knowledge-banks?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-lg font-semibold text-iiz-dark">"Knowledge Banks"</h1>
                    <p class="text-xs text-gray-400 -mt-0.5">"Knowledge Banks for ChatAI and VoiceAI"</p>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Knowledge Bank"</button>
            </header>

            // Search bar
            <div class="bg-white border-b border-gray-200 px-4 py-2">
                <div class="flex items-center gap-3">
                    <div class="join">
                        <input type="text" placeholder="Search knowledge banks..." class="input input-sm input-bordered join-item w-64" />
                        <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                            <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                        </button>
                    </div>
                    <select class="select select-sm select-bordered">
                        <option selected>"All Categories"</option>
                        <option>"General"</option>
                        <option>"Support"</option>
                        <option>"Legal"</option>
                        <option>"Training"</option>
                    </select>
                    <select class="select select-sm select-bordered">
                        <option selected>"All Statuses"</option>
                        <option>"Ready"</option>
                        <option>"Indexing"</option>
                        <option>"Error"</option>
                    </select>
                </div>
            </div>

            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                // Summary cards (static placeholders — need aggregation queries)
                                <div class="grid grid-cols-4 gap-3 p-4">
                                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Knowledge Banks"</div>
                                        <div class="text-2xl font-bold text-iiz-dark mt-1">"—"</div>
                                        <div class="text-xs text-gray-400">"Aggregation pending"</div>
                                    </div>
                                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Total Documents"</div>
                                        <div class="text-2xl font-bold text-iiz-dark mt-1">"—"</div>
                                        <div class="text-xs text-gray-400">"Across all banks"</div>
                                    </div>
                                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Storage Used"</div>
                                        <div class="text-2xl font-bold text-iiz-dark mt-1">"—"</div>
                                        <div class="text-xs text-gray-400">"of 1 GB limit"</div>
                                    </div>
                                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                                        <div class="text-xs text-gray-500 uppercase tracking-wide">"AI Agents Using"</div>
                                        <div class="text-2xl font-bold text-iiz-cyan mt-1">"—"</div>
                                        <div class="text-xs text-gray-400">"Aggregation pending"</div>
                                    </div>
                                </div>

                                // Knowledge bank table
                                <div class="px-4 pb-4">
                                    <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
                                        <table class="table table-sm w-full">
                                            <thead>
                                                <tr class="border-b border-gray-200">
                                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Name"</th>
                                                    <th class="text-xs text-gray-500 font-semibold uppercase text-center">"Documents"</th>
                                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Category"</th>
                                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Status"</th>
                                                    <th class="text-xs text-gray-500 font-semibold uppercase text-right">"Size"</th>
                                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Last Import"</th>
                                                    <th class="text-xs text-gray-500 font-semibold uppercase">"Used By"</th>
                                                    <th class="text-xs text-gray-500 font-semibold uppercase text-center">"Actions"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {items.into_iter().map(|b| {
                                                    let cat_badge = category_badge(&b.category);
                                                    let (dot_cls, status_badge) = status_classes(&b.status);
                                                    let is_indexing = b.status == "Indexing";
                                                    let size_str = fmt_bytes(b.total_size_bytes);
                                                    let last_import_str = b.last_import_at.as_deref().map(fmt_date).unwrap_or_else(|| "\u{2014}".to_string());
                                                    let used_by_str = b.used_by.as_deref().unwrap_or("\u{2014}").to_string();
                                                    view! {
                                                        <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                                            <td>
                                                                <div class="flex items-center gap-2">
                                                                    <span class="w-8 h-8 bg-iiz-cyan/10 rounded flex items-center justify-center flex-shrink-0">
                                                                        <span class="w-4 h-4 inline-flex text-iiz-cyan"><Icon icon=icondata::BsDatabase /></span>
                                                                    </span>
                                                                    <div>
                                                                        <div class="text-sm font-medium text-iiz-dark">{b.name.clone()}</div>
                                                                        <div class="text-xs text-gray-400">{b.category.clone()}</div>
                                                                    </div>
                                                                </div>
                                                            </td>
                                                            <td class="text-sm text-center font-medium">{b.document_count.to_string()}</td>
                                                            <td><span class=format!("badge badge-sm {}", cat_badge)>{b.category.clone()}</span></td>
                                                            <td>
                                                                <div class="flex items-center gap-1">
                                                                    {if is_indexing {
                                                                        view! { <span class="loading loading-spinner loading-xs text-warning"></span> }.into_any()
                                                                    } else {
                                                                        view! { <span class=format!("w-2 h-2 {} rounded-full inline-block", dot_cls)></span> }.into_any()
                                                                    }}
                                                                    <span class=format!("badge badge-sm {}", status_badge)>{b.status.clone()}</span>
                                                                </div>
                                                            </td>
                                                            <td class="text-sm text-right text-gray-600">{size_str}</td>
                                                            <td class="text-xs text-gray-500">{last_import_str}</td>
                                                            <td class="text-xs text-gray-500">{used_by_str}</td>
                                                            <td class="text-center">
                                                                <div class="flex items-center justify-center gap-1">
                                                                    <button class="btn btn-xs btn-ghost text-iiz-cyan">"View"</button>
                                                                    <button class="btn btn-xs btn-ghost text-gray-500">"Import"</button>
                                                                    <button class="btn btn-xs btn-ghost text-red-400">"Remove"</button>
                                                                </div>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </tbody>
                                        </table>
                                    </div>
                                </div>

                                // Upload / Import section
                                <div class="grid grid-cols-2 gap-4 px-4 pb-4">
                                    // Upload area
                                    <div class="bg-white rounded-lg border border-gray-200 p-4">
                                        <h3 class="text-sm font-semibold text-iiz-dark mb-3">"Import Documents"</h3>
                                        <div class="border-2 border-dashed border-gray-300 rounded-lg p-8 text-center hover:border-iiz-cyan transition-colors cursor-pointer">
                                            <span class="w-10 h-10 inline-flex text-gray-300 mx-auto"><Icon icon=icondata::BsCloudUpload /></span>
                                            <p class="text-sm text-gray-600 mt-2">"Drag files here or click to upload"</p>
                                            <p class="text-xs text-gray-400 mt-1">"PDF, DOCX, TXT, CSV, HTML \u{2014} Max 50MB per file"</p>
                                            <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none mt-3">"Choose Files"</button>
                                        </div>
                                        <div class="mt-3 space-y-2">
                                            <h4 class="text-xs font-semibold text-gray-500 uppercase">"Other Import Methods"</h4>
                                            <div class="flex gap-2">
                                                <button class="btn btn-sm btn-outline gap-1">
                                                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGlobe /></span>
                                                    "Import from URL"
                                                </button>
                                                <button class="btn btn-sm btn-outline gap-1">
                                                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsLink45deg /></span>
                                                    "Crawl Website"
                                                </button>
                                                <button class="btn btn-sm btn-outline gap-1">
                                                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsCodeSlash /></span>
                                                    "API Sync"
                                                </button>
                                            </div>
                                        </div>
                                    </div>

                                    // Indexing status + content preview
                                    <div class="bg-white rounded-lg border border-gray-200 p-4">
                                        <h3 class="text-sm font-semibold text-iiz-dark mb-3">"Indexing Status"</h3>
                                        <div class="space-y-3">
                                            // Indexing progress placeholder
                                            <div class="p-3 bg-yellow-50 rounded-lg border border-yellow-100">
                                                <div class="flex items-center justify-between mb-1">
                                                    <span class="text-sm font-medium text-yellow-800">"No active indexing"</span>
                                                    <span class="text-xs text-yellow-600">"—"</span>
                                                </div>
                                                <div class="w-full bg-yellow-200 rounded-full h-2">
                                                    <div class="bg-yellow-500 h-2 rounded-full" style="width: 0%"></div>
                                                </div>
                                                <p class="text-xs text-yellow-600 mt-1">"All banks up to date"</p>
                                            </div>

                                            // Recently indexed
                                            <h4 class="text-xs font-semibold text-gray-500 uppercase">"Recently Indexed"</h4>
                                            <div class="space-y-1">
                                                <div class="flex items-center gap-2 p-2 rounded hover:bg-gray-50">
                                                    <span class="w-4 h-4 inline-flex text-gray-400"><Icon icon=icondata::BsFileText /></span>
                                                    <span class="text-sm text-gray-400 flex-1">"No recent activity"</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
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
// VoiceAI page - list of existing Voice AI agents
// ---------------------------------------------------------------------------

#[component]
pub fn VoiceAIPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<VoiceAiAgentItem>>("/ai-tools/voice-ai?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="mr-auto">
                    <h1 class="text-lg font-semibold text-gray-800">"Voice AI Agents"</h1>
                    <p class="text-xs text-gray-500">"Manage AI-powered voice agents for call handling"</p>
                </div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"+ New Voice AI"</button>
            </header>

            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let meta = resp.pagination.clone();
                    view! {
                        <div class="flex flex-col flex-1 overflow-hidden">
                            <div class="flex-1 overflow-y-auto">
                                <table class="table table-sm w-full">
                                    <thead>
                                        <tr class="text-xs text-gray-500 uppercase">
                                            <th>"Name"</th>
                                            <th>"Description"</th>
                                            <th>"Voice"</th>
                                            <th>"Active"</th>
                                            <th>"Created"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {resp.items.into_iter().map(|item| {
                                            view! {
                                                <tr class="hover:bg-gray-50 cursor-pointer">
                                                    <td class="font-medium text-gray-800">{item.name.clone()}</td>
                                                    <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                    <td class="text-sm text-gray-600">{item.voice_name.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                    <td>
                                                        {if item.is_active {
                                                            view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                        } else {
                                                            view! { <span class="badge badge-sm bg-gray-100 text-gray-500">"Inactive"</span> }.into_any()
                                                        }}
                                                    </td>
                                                    <td class="text-xs text-gray-500">{fmt_date(&item.created_at)}</td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </div>
                            {pagination_footer(&meta)}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// ChatAI page - list of existing Chat AI agents
// ---------------------------------------------------------------------------

#[component]
pub fn ChatAIPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ChatAiAgentItem>>("/ai-tools/chat-ai?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="mr-auto">
                    <h1 class="text-lg font-semibold text-gray-800">"Chat AI Agents"</h1>
                    <p class="text-xs text-gray-500">"Manage AI-powered chat agents"</p>
                </div>
                <span class="badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200">"BETA"</span>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"+ New Chat AI"</button>
            </header>

            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let meta = resp.pagination.clone();
                    view! {
                        <div class="flex flex-col flex-1 overflow-hidden">
                            <div class="flex-1 overflow-y-auto">
                                <table class="table table-sm w-full">
                                    <thead>
                                        <tr class="text-xs text-gray-500 uppercase">
                                            <th>"Name"</th>
                                            <th>"Description"</th>
                                            <th>"Active"</th>
                                            <th>"Created"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {resp.items.into_iter().map(|item| {
                                            view! {
                                                <tr class="hover:bg-gray-50 cursor-pointer">
                                                    <td class="font-medium text-gray-800">{item.name.clone()}</td>
                                                    <td class="text-sm text-gray-600">{item.description.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                    <td>
                                                        {if item.is_active {
                                                            view! { <span class="badge badge-sm bg-green-100 text-green-700 border-green-200">"Active"</span> }.into_any()
                                                        } else {
                                                            view! { <span class="badge badge-sm bg-gray-100 text-gray-500">"Inactive"</span> }.into_any()
                                                        }}
                                                    </td>
                                                    <td class="text-xs text-gray-500">{fmt_date(&item.created_at)}</td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </div>
                            {pagination_footer(&meta)}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

