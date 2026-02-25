use leptos::prelude::*;
use leptos_icons::Icon;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct ContactList {
    name: &'static str,
    description: &'static str,
    members: u32,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct BlockedNumber {
    number: &'static str,
    cnam: &'static str,
    calls_blocked: u32,
    last_blocked: &'static str,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct DncEntry {
    number: &'static str,
    added_by: &'static str,
    created_at: &'static str,
}

#[derive(Clone, Debug)]
struct DntEntry {
    number: &'static str,
    e164: &'static str,
    rejected_count: u32,
    last_rejected: &'static str,
    added_by: &'static str,
    created_at: &'static str,
}

// ---------------------------------------------------------------------------
// Mock data
// ---------------------------------------------------------------------------

fn mock_contact_lists() -> Vec<ContactList> {
    vec![
        ContactList { name: "User 12345 Activity Contacts", description: "luis.barba@company.com", members: 4930, updated: "2026-02-24 01:15:32 PM", created: "2024-06-10 09:00:00 AM" },
        ContactList { name: "User 12346 Activity Contacts", description: "carlos.diaz@company.com", members: 3821, updated: "2026-02-24 12:45:11 PM", created: "2024-06-15 10:30:00 AM" },
        ContactList { name: "User 12347 Activity Contacts", description: "armando@company.com", members: 2915, updated: "2026-02-23 05:20:00 PM", created: "2024-07-01 08:00:00 AM" },
        ContactList { name: "User 12348 Activity Contacts", description: "ramon.acosta@company.com", members: 1847, updated: "2026-02-23 04:10:33 PM", created: "2024-07-20 11:15:00 AM" },
        ContactList { name: "User 12349 Activity Contacts", description: "rafael.martindelcampo@company.com", members: 1203, updated: "2026-02-23 02:00:00 PM", created: "2024-08-05 09:45:00 AM" },
        ContactList { name: "User 12350 Activity Contacts", description: "ruben.rodriguez@company.com", members: 956, updated: "2026-02-22 06:30:00 PM", created: "2024-09-01 10:00:00 AM" },
        ContactList { name: "User 12351 Activity Contacts", description: "magaly.almaraz@company.com", members: 712, updated: "2026-02-22 03:45:22 PM", created: "2024-09-15 08:30:00 AM" },
        ContactList { name: "User 12352 Activity Contacts", description: "cecilia.arrezola@company.com", members: 548, updated: "2026-02-21 11:00:00 AM", created: "2024-10-01 09:00:00 AM" },
        ContactList { name: "VIP Clients", description: "High-priority client contacts", members: 403, updated: "2026-02-20 09:15:00 AM", created: "2024-10-20 14:00:00 PM" },
        ContactList { name: "New Leads Q1 2026", description: "Leads from Q1 campaigns", members: 1587, updated: "2026-02-24 10:30:00 AM", created: "2026-01-02 08:00:00 AM" },
    ]
}

fn mock_blocked_numbers() -> Vec<BlockedNumber> {
    vec![
        BlockedNumber { number: "(919) 553-4064", cnam: "", calls_blocked: 256, last_blocked: "2026-02-23 03:15:00 PM", updated: "2026-02-23 03:15:00 PM", created: "2019-08-15 10:00:00 AM" },
        BlockedNumber { number: "(800) 555-0100", cnam: "SPAM LIKELY", calls_blocked: 42, last_blocked: "2026-02-22 11:30:00 AM", updated: "2026-02-22 11:30:00 AM", created: "2024-03-10 09:00:00 AM" },
        BlockedNumber { number: "(888) 555-0199", cnam: "", calls_blocked: 18, last_blocked: "2026-02-20 02:45:00 PM", updated: "2026-02-20 02:45:00 PM", created: "2024-06-20 14:30:00 PM" },
        BlockedNumber { number: "(404) 555-0123", cnam: "TELEMARKETER", calls_blocked: 7, last_blocked: "2026-02-18 09:00:00 AM", updated: "2026-02-18 09:00:00 AM", created: "2025-01-05 11:00:00 AM" },
        BlockedNumber { number: "(213) 555-0456", cnam: "", calls_blocked: 3, last_blocked: "2026-02-15 04:20:00 PM", updated: "2026-02-15 04:20:00 PM", created: "2025-06-12 08:45:00 AM" },
        BlockedNumber { number: "(305) 555-0789", cnam: "", calls_blocked: 0, last_blocked: "", updated: "2026-01-10 10:00:00 AM", created: "2026-01-10 10:00:00 AM" },
        BlockedNumber { number: "(702) 555-0321", cnam: "", calls_blocked: 0, last_blocked: "", updated: "2026-02-01 09:30:00 AM", created: "2026-02-01 09:30:00 AM" },
        BlockedNumber { number: "(512) 555-0654", cnam: "ROBOCALL", calls_blocked: 15, last_blocked: "2026-02-24 08:10:00 AM", updated: "2026-02-24 08:10:00 AM", created: "2024-11-20 15:00:00 PM" },
    ]
}

fn mock_dnc_entries() -> Vec<DncEntry> {
    vec![
        DncEntry { number: "(919) 555-0101", added_by: "luis.barba@company.com", created_at: "2026-02-24 10:15:00 UTC" },
        DncEntry { number: "(704) 555-0202", added_by: "carlos.diaz@company.com", created_at: "2026-02-23 14:30:00 UTC" },
        DncEntry { number: "(252) 555-0303", added_by: "armando@company.com", created_at: "2026-02-22 09:45:00 UTC" },
        DncEntry { number: "(336) 555-0404", added_by: "ramon.acosta@company.com", created_at: "2026-02-21 16:20:00 UTC" },
        DncEntry { number: "(910) 555-0505", added_by: "rafael.martindelcampo@company.com", created_at: "2026-02-20 11:00:00 UTC" },
        DncEntry { number: "(828) 555-0606", added_by: "ruben.rodriguez@company.com", created_at: "2026-02-19 08:30:00 UTC" },
        DncEntry { number: "(980) 555-0707", added_by: "luis.barba@company.com", created_at: "2026-02-18 13:15:00 UTC" },
        DncEntry { number: "(704) 555-0808", added_by: "carlos.diaz@company.com", created_at: "2026-02-17 10:45:00 UTC" },
        DncEntry { number: "(919) 555-0909", added_by: "armando@company.com", created_at: "2026-01-15 09:00:00 UTC" },
        DncEntry { number: "(336) 555-1010", added_by: "ramon.acosta@company.com", created_at: "2026-01-10 14:00:00 UTC" },
    ]
}

fn mock_dnt_entries() -> Vec<DntEntry> {
    vec![
        DntEntry { number: "(919) 555-1111", e164: "+19195551111", rejected_count: 12, last_rejected: "2026-02-24 09:30:00", added_by: "system", created_at: "2025-03-15 08:00:00 UTC" },
        DntEntry { number: "(704) 555-2222", e164: "+17045552222", rejected_count: 8, last_rejected: "2026-02-23 14:15:00", added_by: "system", created_at: "2025-04-20 10:30:00 UTC" },
        DntEntry { number: "(252) 555-3333", e164: "+12525553333", rejected_count: 5, last_rejected: "2026-02-22 11:45:00", added_by: "system", created_at: "2025-05-10 09:15:00 UTC" },
        DntEntry { number: "(336) 555-4444", e164: "+13365554444", rejected_count: 3, last_rejected: "2026-02-20 16:00:00", added_by: "system", created_at: "2025-06-01 14:30:00 UTC" },
        DntEntry { number: "(910) 555-5555", e164: "+19105555555", rejected_count: 0, last_rejected: "Never", added_by: "system", created_at: "2025-07-15 08:45:00 UTC" },
        DntEntry { number: "(828) 555-6666", e164: "+18285556666", rejected_count: 15, last_rejected: "2026-02-24 10:00:00", added_by: "system", created_at: "2025-08-20 11:00:00 UTC" },
        DntEntry { number: "(980) 555-7777", e164: "+19805557777", rejected_count: 1, last_rejected: "2026-01-15 09:30:00", added_by: "luis.barba@company.com", created_at: "2025-09-05 13:00:00 UTC" },
        DntEntry { number: "(704) 555-8888", e164: "+17045558888", rejected_count: 22, last_rejected: "2026-02-24 08:15:00", added_by: "system", created_at: "2025-10-10 10:15:00 UTC" },
        DntEntry { number: "(919) 555-9999", e164: "+19195559999", rejected_count: 0, last_rejected: "Never", added_by: "system", created_at: "2025-11-20 09:00:00 UTC" },
        DntEntry { number: "(336) 555-0011", e164: "+13365550011", rejected_count: 7, last_rejected: "2026-02-21 15:30:00", added_by: "system", created_at: "2025-12-01 08:30:00 UTC" },
    ]
}

// ---------------------------------------------------------------------------
// Contact Lists page
// ---------------------------------------------------------------------------

#[component]
pub fn ContactListsPage() -> impl IntoView {
    let lists = mock_contact_lists();

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Contact Lists"</h1>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <span class="text-sm text-gray-500">"838 Lists"</span>
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

            <div class="flex-1 overflow-y-auto">
                {lists.into_iter().map(|l| {
                    view! {
                        <div class="activity-row grid grid-cols-[60px_1fr_100px_180px_180px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Edit"</a>
                            <div>
                                <div class="flex items-center gap-2">
                                    <span class="text-sm font-medium">{l.name}</span>
                                    <button class="text-gray-400 hover:text-gray-600">
                                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                    </button>
                                </div>
                                <div class="text-xs text-gray-500">{l.description}</div>
                            </div>
                            <div>
                                <a class="text-sm text-iiz-cyan hover:underline cursor-pointer">{l.members}</a>
                            </div>
                            <div class="text-xs text-gray-500">{l.updated}</div>
                            <div class="text-xs text-gray-500">{l.created}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-10 of 838"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span></button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost">"3"</button>
                    <span class="text-xs text-gray-400">"..."</span>
                    <button class="btn btn-xs btn-ghost">"84"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                </div>
                <span class="text-xs text-gray-400 ml-2">"Per page:"</span>
                <select class="select select-xs select-bordered ml-1">
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
// Blocked Numbers page
// ---------------------------------------------------------------------------

#[component]
pub fn BlockedNumbersPage() -> impl IntoView {
    let numbers = mock_blocked_numbers();

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Blocked Numbers"</h1>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Import"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <span class="text-sm text-gray-500">"30 Blocked Numbers"</span>
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

            <div class="flex-1 overflow-y-auto">
                {numbers.into_iter().map(|n| {
                    let cnam_display = if n.cnam.is_empty() { "\u{2014}" } else { n.cnam };
                    let last_display = if n.last_blocked.is_empty() { "\u{2014}" } else { n.last_blocked };
                    view! {
                        <div class="activity-row grid grid-cols-[32px_140px_120px_100px_160px_160px_160px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <button class="btn btn-xs btn-ghost text-gray-400">
                                <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                            </button>
                            <div class="text-sm font-medium">{n.number}</div>
                            <div class="text-xs text-gray-500">{cnam_display}</div>
                            <div class="text-sm text-center">{n.calls_blocked}</div>
                            <div class="text-xs text-gray-500">{last_display}</div>
                            <div class="text-xs text-gray-500">{n.updated}</div>
                            <div class="text-xs text-gray-500">{n.created}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Do Not Call page
// ---------------------------------------------------------------------------

#[component]
pub fn DoNotCallPage() -> impl IntoView {
    let entries = mock_dnc_entries();

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Do Not Call List"</h1>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">"174 Do Not Calls"</span>
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

            <div class="flex-1 overflow-y-auto">
                {entries.into_iter().map(|e| {
                    view! {
                        <div class="activity-row grid grid-cols-[60px_160px_200px_200px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Remove"</a>
                            <div class="text-sm font-medium">{e.number}</div>
                            <div class="text-xs text-gray-600">{e.added_by}</div>
                            <div class="text-xs text-gray-500">{e.created_at}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-10 of 174"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span></button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Do Not Text page
// ---------------------------------------------------------------------------

#[component]
pub fn DoNotTextPage() -> impl IntoView {
    let entries = mock_dnt_entries();

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Do Not Text List"</h1>
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">"5,910 Do Not Texts"</span>
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

            <div class="flex-1 overflow-y-auto">
                {entries.into_iter().map(|e| {
                    view! {
                        <div class="activity-row grid grid-cols-[60px_180px_100px_140px_120px_180px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Remove"</a>
                            <div>
                                <div class="text-sm font-medium">{e.number}</div>
                                <div class="text-xs text-gray-400">{e.e164}</div>
                            </div>
                            <div class="text-sm text-center">{e.rejected_count}</div>
                            <div class="text-xs text-gray-500">{e.last_rejected}</div>
                            <div class="text-xs text-gray-600">{e.added_by}</div>
                            <div class="text-xs text-gray-500">{e.created_at}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-10 of 5,910"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span></button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost">"3"</button>
                    <span class="text-xs text-gray-400">"..."</span>
                    <button class="btn btn-xs btn-ghost">"60"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                </div>
            </div>
        </div>
    }
}
