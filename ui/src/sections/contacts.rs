use leptos::prelude::*;
use leptos_icons::Icon;

use crate::api::api_get;
use crate::api::types::{
    BlockedNumberItem, ContactListItem, DncEntryItem, DntEntryItem, ListResponse, PaginationMeta,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Format an ISO-8601 datetime string for display (just the first 19 chars).
fn fmt_date(iso: &str) -> String {
    // "2026-02-24T01:15:32Z" -> "2026-02-24 01:15:32"
    iso.replace('T', " ")
        .trim_end_matches('Z')
        .chars()
        .take(19)
        .collect()
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
    // Build a small set of page buttons around the current page.
    let mut pages: Vec<i64> = Vec::new();
    pages.push(1);
    if page > 3 {
        pages.push(-1); // sentinel for "..."
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
    // Deduplicate consecutive sentinels
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

// ---------------------------------------------------------------------------
// Contact Lists page
// ---------------------------------------------------------------------------

#[component]
pub fn ContactListsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ContactListItem>>("/contacts/lists?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Contact Lists"</h1>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Lists", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <div class="join">
                    <input type="text" placeholder="Search lists..." class="input input-sm input-bordered join-item w-48" />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Lists"</button>
            </header>

            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[60px_1fr_100px_180px_180px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Name"</div>
                    <div class="col-header">"Members"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
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
                                {items.into_iter().map(|l| {
                                    let desc = l.description.clone().unwrap_or_default();
                                    let updated = fmt_date(&l.updated_at);
                                    let created = fmt_date(&l.created_at);
                                    view! {
                                        <div class="activity-row grid grid-cols-[60px_1fr_100px_180px_180px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Edit"</a>
                                            <div>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-sm font-medium">{l.name.clone()}</span>
                                                    <button class="text-gray-400 hover:text-gray-600">
                                                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                                    </button>
                                                </div>
                                                <div class="text-xs text-gray-500">{desc}</div>
                                            </div>
                                            <div>
                                                <a class="text-sm text-iiz-cyan hover:underline cursor-pointer">{l.member_count}</a>
                                            </div>
                                            <div class="text-xs text-gray-500">{updated}</div>
                                            <div class="text-xs text-gray-500">{created}</div>
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
// Blocked Numbers page
// ---------------------------------------------------------------------------

#[component]
pub fn BlockedNumbersPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<BlockedNumberItem>>("/contacts/blocked?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Blocked Numbers"</h1>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Import"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Blocked Numbers", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <div class="join">
                    <input type="text" placeholder="Search..." class="input input-sm input-bordered join-item w-40" />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Blocked Number"</button>
            </header>

            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[32px_140px_120px_100px_160px_160px_160px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Blocked Number"</div>
                    <div class="col-header">"CNAM"</div>
                    <div class="col-header">"Calls Blocked"</div>
                    <div class="col-header">"Last Blocked"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
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
                                {items.into_iter().map(|n| {
                                    let cnam = n.cnam.clone().unwrap_or_default();
                                    let cnam_display = if cnam.is_empty() { "\u{2014}".to_string() } else { cnam };
                                    let last_display = n.last_blocked_at
                                        .as_deref()
                                        .map(|s| fmt_date(s))
                                        .unwrap_or_else(|| "\u{2014}".to_string());
                                    let updated = fmt_date(&n.updated_at);
                                    let created = fmt_date(&n.created_at);
                                    view! {
                                        <div class="activity-row grid grid-cols-[32px_140px_120px_100px_160px_160px_160px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <button class="btn btn-xs btn-ghost text-gray-400">
                                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                            </button>
                                            <div class="text-sm font-medium">{n.number.clone()}</div>
                                            <div class="text-xs text-gray-500">{cnam_display}</div>
                                            <div class="text-sm text-center">{n.calls_blocked}</div>
                                            <div class="text-xs text-gray-500">{last_display}</div>
                                            <div class="text-xs text-gray-500">{updated}</div>
                                            <div class="text-xs text-gray-500">{created}</div>
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
// Do Not Call page
// ---------------------------------------------------------------------------

#[component]
pub fn DoNotCallPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<DncEntryItem>>("/contacts/dnc?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Do Not Call List"</h1>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Do Not Calls", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Numbers"</button>
            </header>

            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[60px_160px_200px_200px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Number"</div>
                    <div class="col-header">"Added By"</div>
                    <div class="col-header">"Created at"</div>
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
                                {items.into_iter().map(|e| {
                                    let added_by = e.added_by_id.clone().unwrap_or_else(|| "\u{2014}".to_string());
                                    let created = fmt_date(&e.created_at);
                                    view! {
                                        <div class="activity-row grid grid-cols-[60px_160px_200px_200px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Remove"</a>
                                            <div class="text-sm font-medium">{e.number.clone()}</div>
                                            <div class="text-xs text-gray-600">{added_by}</div>
                                            <div class="text-xs text-gray-500">{created}</div>
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
// Do Not Text page
// ---------------------------------------------------------------------------

#[component]
pub fn DoNotTextPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<DntEntryItem>>("/contacts/dnt?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Do Not Text List"</h1>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Do Not Texts", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Do Not Text"</button>
            </header>

            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[60px_180px_100px_140px_120px_180px] gap-2 px-4 py-2 items-center">
                    <div class="col-header"></div>
                    <div class="col-header">"Number"</div>
                    <div class="col-header">"Rejected"</div>
                    <div class="col-header">"Last Rejected"</div>
                    <div class="col-header">"Added by"</div>
                    <div class="col-header">"Created at"</div>
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
                                {items.into_iter().map(|e| {
                                    let last_rejected = e.last_rejected_at
                                        .as_deref()
                                        .map(|s| fmt_date(s))
                                        .unwrap_or_else(|| "\u{2014}".to_string());
                                    let added_by = e.added_by_id.clone().unwrap_or_else(|| "\u{2014}".to_string());
                                    let created = fmt_date(&e.created_at);
                                    view! {
                                        <div class="activity-row grid grid-cols-[60px_180px_100px_140px_120px_180px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Remove"</a>
                                            <div>
                                                <div class="text-sm font-medium">{e.number.clone()}</div>
                                                <div class="text-xs text-gray-400">{e.e164.clone()}</div>
                                            </div>
                                            <div class="text-sm text-center">{e.rejected_count}</div>
                                            <div class="text-xs text-gray-500">{last_rejected}</div>
                                            <div class="text-xs text-gray-600">{added_by}</div>
                                            <div class="text-xs text-gray-500">{created}</div>
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
