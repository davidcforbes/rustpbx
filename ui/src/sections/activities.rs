use leptos::ev;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

use crate::api::api_get;
use crate::api::types::{
    ChatRecordItem, ExportRecordItem, FaxRecordItem, FormRecordItem, ListResponse, PaginationMeta,
    TextRecordItem, VideoRecordItem,
};
use crate::components::{CallDetailPanel, FilterBar};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct CallRecord {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub location: String,
    pub source: String,
    pub source_number: String,
    pub source_name: String,
    pub has_audio: bool,
    pub duration: String,
    pub date: String,
    pub time: String,
    pub status: String,
    pub agent: String,
    pub agent_initials: String,
    pub agent_color: String,
    pub automation: String,
    pub tags: Vec<String>,
    // CRM / case metadata (visible in the real 4iiz UI)
    pub case_description: String,
    pub contact_category: String,
    pub crm_contact_id: String,
    pub crm_matter_id: String,
    pub case_subtype: String,
    pub matter_status: String,
    pub answered_by: String,
}

// ---------------------------------------------------------------------------
// Mock data -- mirrors the prototype's Alpine.js `calls` array
// ---------------------------------------------------------------------------

fn mock_calls() -> Vec<CallRecord> {
    vec![
        CallRecord {
            id: "4045529975".into(),
            name: "Francisco Javier 26-49335".into(),
            phone: "(714) 737-3835".into(),
            location: "Cypress, CA US".into(),
            source: "Google Organic".into(),
            source_number: "(949) 649-6378".into(),
            source_name: "Santa Ana Office - Google My Business".into(),
            has_audio: false,
            duration: "00:00".into(),
            date: "Wed Feb 25th".into(),
            time: "04:46 PM".into(),
            status: "Hangup".into(),
            agent: "Javi Guerrero".into(),
            agent_initials: "JG".into(),
            agent_color: "#0277bd".into(),
            automation: String::new(),
            tags: vec!["agent_assigned".into()],
            case_description: "U-Visa Investigation (Not Detained)".into(),
            contact_category: String::new(),
            crm_contact_id: "4307460001363256958".into(),
            crm_matter_id: "4307460001306053781".into(),
            case_subtype: String::new(),
            matter_status: String::new(),
            answered_by: String::new(),
        },
        CallRecord {
            id: "4045529976".into(),
            name: "Martha A Morales - Fb".into(),
            phone: "(979) 567-8615".into(),
            location: "Bryan, TX US".into(),
            source: "Google Organic".into(),
            source_number: "(603) 218-2269".into(),
            source_name: "Dallas Office - Google My Business".into(),
            has_audio: true,
            duration: "01:03".into(),
            date: "Wed Feb 25th".into(),
            time: "04:45 PM".into(),
            status: "Answered".into(),
            agent: "Fernanda Padilla".into(),
            agent_initials: "FP".into(),
            agent_color: "#7b1fa2".into(),
            automation: String::new(),
            tags: vec![],
            case_description: String::new(),
            contact_category: String::new(),
            crm_contact_id: String::new(),
            crm_matter_id: String::new(),
            case_subtype: String::new(),
            matter_status: String::new(),
            answered_by: "Appointments Set?: No".into(),
        },
        CallRecord {
            id: "4045529977".into(),
            name: "Mirian Elizabeth Ocampo Diaz 25-42709".into(),
            phone: "(910) 305-3917".into(),
            location: "Fayetteville, NC US".into(),
            source: "Google Organic".into(),
            source_number: "(919) 725-8000".into(),
            source_name: "Durham Office - Google My Business".into(),
            has_audio: true,
            duration: "00:35".into(),
            date: "Wed Feb 25th".into(),
            time: "04:44 PM".into(),
            status: "Answered".into(),
            agent: "Daniela Nubia".into(),
            agent_initials: "DN".into(),
            agent_color: "#00897b".into(),
            automation: "Answered Calls Lookup - Missed Calls Automation".into(),
            tags: vec![],
            case_description: "Adjustment Of Status Relative Petition".into(),
            contact_category: "Customer Service".into(),
            crm_contact_id: "4307460001924153177".into(),
            crm_matter_id: "4307460010060854368".into(),
            case_subtype: "Document Collection (AOS-IR)".into(),
            matter_status: String::new(),
            answered_by: "agent_assigned".into(),
        },
        CallRecord {
            id: "4045529978".into(),
            name: "Raquel Escobar".into(),
            phone: "(714) 981-6483".into(),
            location: "Placentia, CA US".into(),
            source: "Google Organic".into(),
            source_number: "(949) 649-6378".into(),
            source_name: "Santa Ana Office - Google My Business".into(),
            has_audio: false,
            duration: "54:07".into(),
            date: "Wed Feb 25th".into(),
            time: "04:45 PM".into(),
            status: "in progress".into(),
            agent: "Brandon Nunez".into(),
            agent_initials: "BN".into(),
            agent_color: "#c62828".into(),
            automation: String::new(),
            tags: vec![
                "repeated caller".into(),
                "spanish ivr".into(),
                "sales call".into(),
            ],
            case_description: String::new(),
            contact_category: String::new(),
            crm_contact_id: String::new(),
            crm_matter_id: String::new(),
            case_subtype: String::new(),
            matter_status: String::new(),
            answered_by: "Answered?: \u{2714}".into(),
        },
        CallRecord {
            id: "4045529979".into(),
            name: "Karla Velazquez Girlfriend Olvan Josue Fajardo Dominguez 25-36648".into(),
            phone: "(240) 733-5285".into(),
            location: "Durham, NC".into(),
            source: "Google Organic".into(),
            source_number: "(919) 725-8000".into(),
            source_name: "Durham Office - Google My Business".into(),
            has_audio: true,
            duration: "03:42".into(),
            date: "Wed Feb 25th".into(),
            time: "04:45 PM".into(),
            status: "Answered".into(),
            agent: "Judith Andrade".into(),
            agent_initials: "JA".into(),
            agent_color: "#4527a0".into(),
            automation: String::new(),
            tags: vec![],
            case_description: "U-Visa Investigation".into(),
            contact_category: "Customer Service".into(),
            crm_contact_id: "4307460008623341469".into(),
            crm_matter_id: "4307460009004326759".into(),
            case_subtype: "0023 - Follow up Call to Agency (UVil)".into(),
            matter_status: String::new(),
            answered_by: String::new(),
        },
        CallRecord {
            id: "4045529980".into(),
            name: "Jose Ramon Garcia Sanchez 25-45997".into(),
            phone: "(602) 930-7605".into(),
            location: "Phoenix, AZ US".into(),
            source: "Google Organic".into(),
            source_number: "(602) 838-6665".into(),
            source_name: "Phoenix Office - Google My Business".into(),
            has_audio: false,
            duration: "01:08".into(),
            date: "Wed Feb 25th".into(),
            time: "02:39 PM".into(),
            status: "in progress".into(),
            agent: "Oswaldo Aguilera".into(),
            agent_initials: "OA".into(),
            agent_color: "#0277bd".into(),
            automation: "Initial Language Selection".into(),
            tags: vec![
                "repeated caller".into(),
                "spanish ivr".into(),
                "inbound to make payment".into(),
            ],
            case_description: String::new(),
            contact_category: String::new(),
            crm_contact_id: String::new(),
            crm_matter_id: String::new(),
            case_subtype: String::new(),
            matter_status: String::new(),
            answered_by: String::new(),
        },
        CallRecord {
            id: "4045529981".into(),
            name: "Clemente Aldahir Gonzalez".into(),
            phone: "(657) 520-8092".into(),
            location: "Santa Ana, CA US".into(),
            source: "Tiktok Organic".into(),
            source_number: "(657) 279-5506".into(),
            source_name: "TikTok Organic".into(),
            has_audio: false,
            duration: "02:49".into(),
            date: "Wed Feb 25th".into(),
            time: "02:37 PM".into(),
            status: "in progress".into(),
            agent: "Israel Navarro".into(),
            agent_initials: "IN".into(),
            agent_color: "#558b2f".into(),
            automation: "Initial Language Selection".into(),
            tags: vec![
                "repeated caller".into(),
                "spanish ivr".into(),
                "sales call".into(),
            ],
            case_description: String::new(),
            contact_category: String::new(),
            crm_contact_id: String::new(),
            crm_matter_id: String::new(),
            case_subtype: String::new(),
            matter_status: String::new(),
            answered_by: String::new(),
        },
        CallRecord {
            id: "4045529982".into(),
            name: "Adolfo Angel Valerio Armijo 25-47527".into(),
            phone: "(323) 598-3978".into(),
            location: "Montebello, CA US".into(),
            source: "Google Organic".into(),
            source_number: "(949) 649-6378".into(),
            source_name: "Santa Ana Office - Google My Business".into(),
            has_audio: false,
            duration: "03:05".into(),
            date: "Wed Feb 25th".into(),
            time: "02:37 PM".into(),
            status: "in progress".into(),
            agent: String::new(),
            agent_initials: "+".into(),
            agent_color: "#9e9e9e".into(),
            automation: "Initial Language Selection".into(),
            tags: vec![
                "repeated caller".into(),
                "english ivr".into(),
                "cs routed per priming".into(),
                "cs smart router".into(),
            ],
            case_description: String::new(),
            contact_category: String::new(),
            crm_contact_id: String::new(),
            crm_matter_id: String::new(),
            case_subtype: String::new(),
            matter_status: String::new(),
            answered_by: String::new(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Helper functions for API data rendering
// ---------------------------------------------------------------------------

fn fmt_date(iso: &str) -> String {
    if iso.len() >= 10 {
        iso[..10].to_string()
    } else {
        iso.to_string()
    }
}

fn fmt_time(t: &str) -> String {
    if t.len() >= 16 {
        t[11..16].to_string()
    } else {
        t.to_string()
    }
}

fn fmt_duration(secs: i32) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{m:02}:{s:02}")
}

fn loading_view() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-64">
            <span class="loading loading-spinner loading-lg text-iiz-cyan"></span>
        </div>
    }
}

fn error_view(msg: String) -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-64">
            <div class="text-center">
                <div class="text-red-500 text-lg font-semibold">"Error"</div>
                <div class="text-gray-500 mt-1">{msg}</div>
            </div>
        </div>
    }
}

fn pagination_footer(meta: &PaginationMeta) -> impl IntoView {
    let start = ((meta.page - 1) * meta.per_page) + 1;
    let end = std::cmp::min(meta.page * meta.per_page, meta.total_items);
    let total = meta.total_items;
    view! {
        <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
            <span class="text-xs text-gray-400">{format!("{start}\u{2013}{end} of {total}")}</span>
            <div class="flex-1"></div>
            <span class="text-xs text-gray-400">"Per page:"</span>
            <select class="select select-xs select-bordered ml-1">
                <option selected>"25"</option>
                <option>"50"</option>
                <option>"100"</option>
            </select>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Activities side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn ActivitiesSideNav() -> impl IntoView {
    let location = use_location();
    let active = |href: &'static str| {
        move || {
            if location.pathname.get() == href { "side-nav-item active" } else { "side-nav-item" }
        }
    };

    view! {
        // Section header
        <div class="px-4 pt-4 pb-2">
            <div class="flex items-center gap-2 text-iiz-cyan">
                <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsTelephoneFill /></span>
                <span class="text-lg font-light">"Activities"</span>
            </div>
        </div>

        <nav class="px-2 pb-4">
            // Activity Logs group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsGrid3x3GapFill /></span>
                    "Activity Logs"
                </h3>
                <a href="/activities/calls" class=active("/activities/calls")>"Calls"</a>
                <a href="/activities/texts" class=active("/activities/texts")>"Texts"</a>
                <a href="/activities/forms" class=active("/activities/forms")>"Forms"</a>
                <a href="/activities/chats" class=active("/activities/chats")>"Chats"</a>
                <a href="/activities/faxes" class=active("/activities/faxes")>"Faxes"</a>
                <a href="/activities/videos" class=active("/activities/videos")>"Videos"</a>
                <a href="/activities/export" class=active("/activities/export")>"Export Log"</a>
            </div>

            // Contacts group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsPeopleFill /></span>
                    "Contacts"
                </h3>
                <a href="/contacts/lists" class=active("/contacts/lists")>"Lists"</a>
                <a href="/contacts/blocked" class=active("/contacts/blocked")>"Blocked Numbers"</a>
                <a href="/contacts/do-not-call" class=active("/contacts/do-not-call")>"Do Not Call List"</a>
                <a href="/contacts/do-not-text" class=active("/contacts/do-not-text")>"Do Not Text List"</a>
            </div>
        </nav>
    }
}

// ---------------------------------------------------------------------------
// Calls page -- the main activity list view
// ---------------------------------------------------------------------------

#[component]
pub fn CallsPage() -> impl IntoView {
    let calls = mock_calls();
    let selected_call = RwSignal::new(Option::<CallRecord>::None);

    view! {
        <div class="flex flex-col h-full relative">
            // Top filter bar
            <FilterBar />

            // Column headers
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[24px_2.5fr_1.5fr_0.6fr_0.5fr_0.6fr_0.6fr_1.2fr_36px] gap-2 px-4 py-2 items-center">
                    <div></div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex text-iiz-cyan"><Icon icon=icondata::BsPerson /></span>
                        "Contact"
                    </div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsBuilding /></span>
                        "Source"
                    </div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsLayoutTextSidebar /></span>
                        "Session Data"
                    </div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsStarFill /></span>
                        "Score"
                    </div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsVolumeUpFill /></span>
                        "Audio"
                    </div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsGraphUp /></span>
                        "Metrics"
                    </div>
                    <div></div>
                    <div></div>
                </div>
            </div>

            // Call rows
            <div class="flex-1 overflow-y-auto">
                {calls
                    .into_iter()
                    .map(|call| {
                        let call_for_click = call.clone();
                        let call_id_sel = call.id.clone();
                        let is_selected = move || {
                            selected_call
                                .get()
                                .as_ref()
                                .map(|c| c.id == call_id_sel)
                                .unwrap_or(false)
                        };
                        view! {
                            <CallRow
                                call=call
                                selected=Signal::derive(is_selected)
                                on_click=move |_| {
                                    selected_call.set(Some(call_for_click.clone()));
                                }
                            />
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>

            // Status bar
            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-8 of 3,700,569 results"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span>
                    </button>
                    <span class="text-xs">"Page 1"</span>
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span>
                    </button>
                </div>
            </div>

            // Detail panel (slide-out)
            <Show when=move || selected_call.get().is_some()>
                {move || {
                    selected_call.get().map(|call| {
                        view! {
                            <CallDetailPanel
                                call=call
                                on_close=move |_| selected_call.set(None)
                            />
                        }
                    })
                }}
            </Show>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Individual call row component
// ---------------------------------------------------------------------------

#[component]
fn CallRow(
    call: CallRecord,
    #[prop(into)] selected: Signal<bool>,
    on_click: impl Fn(ev::MouseEvent) + 'static,
) -> impl IntoView {
    let status_color = match call.status.as_str() {
        "Answered" => "text-iiz-cyan",
        "Hangup" => "text-red-500",
        _ => "text-iiz-orange",
    };
    let status_dot_color = match call.status.as_str() {
        "Answered" => "bg-iiz-cyan",
        "Hangup" => "bg-red-400",
        _ => "bg-orange-400",
    };

    let audio_icon_class = if call.has_audio {
        "w-3.5 h-3.5 inline-flex text-iiz-cyan"
    } else {
        "w-3.5 h-3.5 inline-flex text-gray-300"
    };

    let audio_label: &'static str = if call.has_audio { "audio" } else { "no audio" };

    let has_tags = !call.tags.is_empty();
    let has_agent = !call.agent.is_empty();
    let has_automation = !call.automation.is_empty();
    let has_case = !call.case_description.is_empty();
    let has_crm = !call.crm_contact_id.is_empty();
    let has_category = !call.contact_category.is_empty();
    let has_subtype = !call.case_subtype.is_empty();
    let has_answered_by = !call.answered_by.is_empty();

    // Pre-compute all strings for the view
    let name = call.name.clone();
    let phone = call.phone.clone();
    let location = call.location.clone();
    let source = call.source.clone();
    let source_number = call.source_number.clone();
    let source_name = call.source_name.clone();
    let duration_text = format!("\u{25B6} {}", &call.duration);
    let date = call.date.clone();
    let time = call.time.clone();
    let status_text = call.status.clone();
    let agent_name = call.agent.clone();
    let agent_initials = call.agent_initials.clone();
    let agent_color_style = format!("background-color:{}", &call.agent_color);
    let automation = call.automation.clone();
    let tags = call.tags.clone();
    let case_description = call.case_description.clone();
    let crm_contact_id = format!("crm_contact_id: {}", &call.crm_contact_id);
    let crm_matter_id = format!("crm_matter_id: {}", &call.crm_matter_id);
    let contact_category = format!("Contact Category: {}", &call.contact_category);
    let case_subtype = format!("Case Subtype: {}", &call.case_subtype);
    let answered_by = call.answered_by.clone();

    view! {
        <div
            class=move || {
                let base = "activity-row grid grid-cols-[24px_2.5fr_1.5fr_0.6fr_0.5fr_0.6fr_0.6fr_1.2fr_36px] gap-2 px-4 py-2.5 items-start cursor-pointer";
                if selected.get() {
                    format!("{} bg-iiz-cyan-light", base)
                } else {
                    base.to_string()
                }
            }
            on:click=on_click
        >
            // Call icon
            <div class="pt-0.5">
                <span class="w-4 h-4 inline-flex text-green-500"><Icon icon=icondata::BsTelephoneFill /></span>
            </div>

            // Contact column -- dense multi-line card with CRM metadata
            <div class="min-w-0">
                <div class="flex items-start gap-1">
                    <span class="font-semibold text-[13px] leading-tight text-gray-900">{name}</span>
                    <button class="text-gray-300 hover:text-gray-500 flex-shrink-0 mt-0.5">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsThreeDotsVertical /></span>
                    </button>
                </div>
                <div class="text-[11px] text-gray-500 leading-tight">{phone}</div>
                <div class="text-[11px] text-gray-400 leading-tight">{location}</div>
                {if has_case {
                    Some(view! {
                        <div class="text-[11px] text-gray-600 leading-tight mt-0.5 font-medium">{case_description}</div>
                    })
                } else {
                    None
                }}
                {if has_category {
                    Some(view! {
                        <div class="text-[10px] text-gray-400 leading-tight">{contact_category}</div>
                    })
                } else {
                    None
                }}
                {if has_crm {
                    Some(view! {
                        <div class="text-[10px] text-gray-400 leading-tight font-mono">{crm_contact_id}</div>
                        <div class="text-[10px] text-gray-400 leading-tight font-mono">{crm_matter_id}</div>
                    })
                } else {
                    None
                }}
                {if has_subtype {
                    Some(view! {
                        <div class="text-[10px] text-gray-400 leading-tight">{case_subtype}</div>
                    })
                } else {
                    None
                }}
                {if has_answered_by {
                    Some(view! {
                        <div class="text-[10px] text-gray-400 leading-tight">{answered_by}</div>
                    })
                } else {
                    None
                }}
                {if has_tags {
                    Some(
                        view! {
                            <div class="flex flex-wrap gap-1 mt-1">
                                {tags
                                    .iter()
                                    .map(|tag| {
                                        let t = tag.clone();
                                        view! { <span class="tag-badge">{t}</span> }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        },
                    )
                } else {
                    None
                }}
            </div>

            // Source column
            <div class="min-w-0">
                <div class="flex items-center gap-1">
                    <span class="w-3 h-3 inline-flex text-red-400 flex-shrink-0"><Icon icon=icondata::BsGeoAltFill /></span>
                    <span class="text-[12px] font-semibold text-gray-800 truncate">{source}</span>
                </div>
                <div class="text-[11px] text-iiz-blue-link leading-tight">{source_number}</div>
                <div class="text-[10px] text-gray-400 leading-tight truncate">{source_name}</div>
            </div>

            // Session Data column
            <div class="flex justify-center pt-0.5">
                <span class="w-4 h-4 inline-flex text-gray-300"><Icon icon=icondata::BsBarChartFill /></span>
            </div>

            // Score column
            <div class="flex justify-center pt-0.5">
                <button class="text-gray-300 hover:text-iiz-cyan">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsBarChartFill /></span>
                </button>
            </div>

            // Audio column
            <div>
                <div class="flex items-center gap-1">
                    <span class={audio_icon_class}><Icon icon=icondata::BsVolumeUpFill /></span>
                    <span class="text-[10px] text-gray-500">{audio_label}</span>
                </div>
                <div class="text-[10px] text-gray-500 flex items-center gap-0.5 mt-0.5">
                    <span class="w-2.5 h-2.5 inline-flex"><Icon icon=icondata::BsPlayFill /></span>
                    {duration_text}
                </div>
            </div>

            // Metrics + Agent combined column (stacked right side like real app)
            <div class="text-right">
                // Agent row at top
                <div class="flex items-center justify-end gap-1.5 mb-1">
                    {if has_agent {
                        view! {
                            <span class="text-[11px] text-gray-600 truncate">{agent_name.clone()}</span>
                            <div
                                class="w-6 h-6 rounded-full text-white text-[9px] flex items-center justify-center flex-shrink-0"
                                style=agent_color_style.clone()
                            >
                                <span>{agent_initials.clone()}</span>
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <a class="text-[11px] text-iiz-cyan hover:underline cursor-pointer">"+ set agent"</a>
                            <div
                                class="w-6 h-6 rounded-full text-white text-[9px] flex items-center justify-center flex-shrink-0"
                                style=agent_color_style.clone()
                            >
                                <span>{agent_initials.clone()}</span>
                            </div>
                        }
                        .into_any()
                    }}
                </div>
                // Date & time
                <div class="text-[11px] text-gray-500 leading-tight">{date}</div>
                <div class="text-[11px] text-gray-500 leading-tight">{time}</div>
                // Status with colored dot
                <div class="flex items-center justify-end gap-1 mt-0.5">
                    <span class={format!("w-1.5 h-1.5 rounded-full inline-block {}", status_dot_color)}></span>
                    <span class={format!("text-[11px] font-medium {}", status_color)}>{status_text}</span>
                </div>
                {if has_automation {
                    Some(view! {
                        <div class="text-[10px] text-iiz-blue-link leading-tight mt-0.5 truncate">{automation.clone()}</div>
                    })
                } else {
                    None
                }}
            </div>

            // Actions column
            <div class="flex flex-col items-center gap-1 pt-0.5">
                <button class="text-gray-300 hover:text-gray-500">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsEnvelope /></span>
                </button>
                <button class="text-gray-300 hover:text-red-400">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsFlag /></span>
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Texts page -- SMS/text message activity log
// ---------------------------------------------------------------------------

#[component]
pub fn TextsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<TextRecordItem>>("/activities/texts?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <FilterBar />

            // Column headers
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[1.5fr_2fr_1fr_0.8fr_0.8fr_1fr] gap-2 px-4 py-2 items-center">
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPerson /></span>
                        "Contact"
                    </div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChatDots /></span>
                        "Message Preview"
                    </div>
                    <div class="col-header">"Source"</div>
                    <div class="col-header">"Direction"</div>
                    <div class="col-header">"Status"</div>
                    <div class="col-header">"Date / Time"</div>
                </div>
            </div>

            // Text rows with loading/error handling
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
                                    let phone = t.contact_phone.clone().unwrap_or_default();
                                    let preview = t.preview.clone().unwrap_or_default();
                                    let source = t.tracking_number_id.clone().unwrap_or_default();
                                    let dir_class = if t.direction == "Inbound" {
                                        "badge badge-sm bg-blue-100 text-blue-700 border-none"
                                    } else {
                                        "badge badge-sm bg-purple-100 text-purple-700 border-none"
                                    };
                                    let status_class = match t.status.as_str() {
                                        "Delivered" => "badge badge-sm bg-green-100 text-green-700 border-none",
                                        "Failed" => "badge badge-sm bg-red-100 text-red-700 border-none",
                                        _ => "badge badge-sm bg-yellow-100 text-yellow-700 border-none",
                                    };
                                    let date = fmt_date(&t.sent_at);
                                    let time = fmt_time(&t.sent_at);
                                    view! {
                                        <div class="activity-row grid grid-cols-[1.5fr_2fr_1fr_0.8fr_0.8fr_1fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <div>
                                                <div class="text-sm font-medium">{phone.clone()}</div>
                                                <div class="text-xs text-gray-500">{t.id.clone()}</div>
                                            </div>
                                            <div class="text-sm text-gray-600 truncate">{preview}</div>
                                            <div class="text-xs text-iiz-cyan">{source}</div>
                                            <div><span class=dir_class>{t.direction.clone()}</span></div>
                                            <div><span class=status_class>{t.status.clone()}</span></div>
                                            <div>
                                                <div class="text-xs text-gray-500">{date}</div>
                                                <div class="text-xs text-gray-400">{time}</div>
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
// Forms page -- form submission activity log
// ---------------------------------------------------------------------------

#[component]
pub fn FormsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<FormRecordItem>>("/activities/forms?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            // Header
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <FilterBar />
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">
                    {move || {
                        data.get()
                            .and_then(|r| r.ok())
                            .map(|r| format!("{} Forms", r.pagination.total_items))
                            .unwrap_or_default()
                    }}
                </span>
            </header>

            // Column headers
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[1.8fr_1.2fr_1fr_1fr_1fr_0.7fr] gap-2 px-4 py-2 items-center">
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPerson /></span>
                        "Contact"
                    </div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsFileEarmarkText /></span>
                        "Form Name"
                    </div>
                    <div class="col-header">"Source"</div>
                    <div class="col-header">"Tracking Number"</div>
                    <div class="col-header">"Date / Time"</div>
                    <div class="col-header">"Status"</div>
                </div>
            </div>

            // Form rows with loading/error handling
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
                                    let name = f.contact_name.clone().unwrap_or_default();
                                    let phone = f.contact_phone.clone().unwrap_or_default();
                                    let email = f.contact_email.clone().unwrap_or_default();
                                    let form_name = f.form_name.clone().unwrap_or_default();
                                    let source = f.source.clone().unwrap_or_default();
                                    let tracking = f.tracking_number.clone().unwrap_or_default();
                                    let date = fmt_date(&f.submitted_at);
                                    let time = fmt_time(&f.submitted_at);
                                    let status_class = if f.status == "New" {
                                        "badge badge-sm bg-blue-100 text-blue-700 border-none"
                                    } else {
                                        "badge badge-sm bg-green-100 text-green-700 border-none"
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[1.8fr_1.2fr_1fr_1fr_1fr_0.7fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <div>
                                                <div class="text-sm font-medium">{name}</div>
                                                <div class="text-xs text-gray-500">{phone}</div>
                                                <div class="text-xs text-iiz-cyan">{email}</div>
                                            </div>
                                            <div class="text-sm text-gray-700">{form_name}</div>
                                            <div class="text-xs text-gray-600">{source}</div>
                                            <div class="text-xs text-iiz-cyan">{tracking}</div>
                                            <div>
                                                <div class="text-xs text-gray-500">{date}</div>
                                                <div class="text-xs text-gray-400">{time}</div>
                                            </div>
                                            <div><span class=status_class>{f.status.clone()}</span></div>
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
// Chats page -- live chat conversation activity log
// ---------------------------------------------------------------------------

#[component]
pub fn ChatsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ChatRecordItem>>("/activities/chats?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <FilterBar />

            // Column headers
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[1.5fr_0.8fr_0.6fr_0.7fr_1fr_0.7fr_1fr] gap-2 px-4 py-2 items-center">
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPerson /></span>
                        "Visitor"
                    </div>
                    <div class="col-header">"Channel"</div>
                    <div class="col-header">"Messages"</div>
                    <div class="col-header">"Duration"</div>
                    <div class="col-header">"Agent"</div>
                    <div class="col-header">"Status"</div>
                    <div class="col-header">"Date / Time"</div>
                </div>
            </div>

            // Chat rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|c| {
                                    let visitor = c.visitor_name.clone().unwrap_or("Unknown Visitor".to_string());
                                    let detail = c.visitor_detail.clone().unwrap_or_default();
                                    let channel = c.channel.clone().unwrap_or("Web Chat".to_string());
                                    let agent = c.agent_id.clone().unwrap_or_default();
                                    let msg_count = c.message_count.to_string();
                                    let duration = fmt_duration(c.duration_secs);
                                    let date = fmt_date(&c.started_at);
                                    let time = fmt_time(&c.started_at);
                                    let channel_class = if channel == "Web Chat" {
                                        "badge badge-sm bg-indigo-100 text-indigo-700 border-none"
                                    } else {
                                        "badge badge-sm bg-teal-100 text-teal-700 border-none"
                                    };
                                    let status_class = if c.status == "Active" {
                                        "badge badge-sm bg-green-100 text-green-700 border-none"
                                    } else {
                                        "badge badge-sm bg-gray-100 text-gray-600 border-none"
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[1.5fr_0.8fr_0.6fr_0.7fr_1fr_0.7fr_1fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <div>
                                                <div class="text-sm font-medium">{visitor}</div>
                                                <div class="text-xs text-gray-400">{detail}</div>
                                            </div>
                                            <div><span class=channel_class>{channel}</span></div>
                                            <div class="text-sm text-gray-700">{msg_count}</div>
                                            <div class="text-sm text-gray-600">{duration}</div>
                                            <div class="text-sm text-gray-700">{agent}</div>
                                            <div><span class=status_class>{c.status.clone()}</span></div>
                                            <div>
                                                <div class="text-xs text-gray-500">{date}</div>
                                                <div class="text-xs text-gray-400">{time}</div>
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
// Faxes page -- fax activity log
// ---------------------------------------------------------------------------

#[component]
pub fn FaxesPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<FaxRecordItem>>("/activities/fax?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <FilterBar />

            // Column headers
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[1.8fr_0.5fr_0.8fr_0.8fr_1fr] gap-2 px-4 py-2 items-center">
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPerson /></span>
                        "Contact"
                    </div>
                    <div class="col-header">"Pages"</div>
                    <div class="col-header">"Direction"</div>
                    <div class="col-header">"Status"</div>
                    <div class="col-header">"Date / Time"</div>
                </div>
            </div>

            // Fax rows with loading/error handling
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
                                    let from = f.from_number.clone().unwrap_or_default();
                                    let to = f.to_number.clone().unwrap_or_default();
                                    let display_number = if f.direction == "Inbound" { from } else { to };
                                    let page_count = f.pages.to_string();
                                    let date = fmt_date(&f.sent_at);
                                    let time = fmt_time(&f.sent_at);
                                    let dir_class = if f.direction == "Inbound" {
                                        "badge badge-sm bg-blue-100 text-blue-700 border-none"
                                    } else {
                                        "badge badge-sm bg-purple-100 text-purple-700 border-none"
                                    };
                                    let status_class = match f.status.as_str() {
                                        "Received" | "Sent" => "badge badge-sm bg-green-100 text-green-700 border-none",
                                        _ => "badge badge-sm bg-red-100 text-red-700 border-none",
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[1.8fr_0.5fr_0.8fr_0.8fr_1fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <div>
                                                <div class="text-sm font-medium">{f.id.clone()}</div>
                                                <div class="text-xs text-gray-500">{display_number}</div>
                                            </div>
                                            <div class="text-sm text-gray-700">{page_count}</div>
                                            <div><span class=dir_class>{f.direction.clone()}</span></div>
                                            <div><span class=status_class>{f.status.clone()}</span></div>
                                            <div>
                                                <div class="text-xs text-gray-500">{date}</div>
                                                <div class="text-xs text-gray-400">{time}</div>
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
// Videos page -- video call activity log
// ---------------------------------------------------------------------------

#[component]
pub fn VideosPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<VideoRecordItem>>("/activities/video?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <FilterBar />

            // Column headers
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[1.5fr_0.7fr_0.7fr_0.7fr_1fr_1fr] gap-2 px-4 py-2 items-center">
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPerson /></span>
                        "Contact"
                    </div>
                    <div class="col-header">"Duration"</div>
                    <div class="col-header">"Platform"</div>
                    <div class="col-header">"Recording"</div>
                    <div class="col-header">"Agent"</div>
                    <div class="col-header">"Date / Time"</div>
                </div>
            </div>

            // Video rows with loading/error handling
            {move || match data.get() {
                None => loading_view().into_any(),
                Some(Err(e)) => error_view(e).into_any(),
                Some(Ok(resp)) => {
                    let items = resp.items.clone();
                    let meta = resp.pagination.clone();
                    view! {
                        <>
                            <div class="flex-1 overflow-y-auto">
                                {items.into_iter().map(|v| {
                                    let name = v.participant_name.clone().unwrap_or_default();
                                    let email = v.participant_email.clone().unwrap_or_default();
                                    let platform = v.platform.clone().unwrap_or("Zoom".to_string());
                                    let agent = v.host_agent_id.clone().unwrap_or_default();
                                    let duration = fmt_duration(v.duration_secs);
                                    let date = fmt_date(&v.started_at);
                                    let time = fmt_time(&v.started_at);
                                    let rec_view = if v.has_recording {
                                        ("w-4 h-4 inline-flex text-green-500", "Available")
                                    } else {
                                        ("w-4 h-4 inline-flex text-gray-300", "None")
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[1.5fr_0.7fr_0.7fr_0.7fr_1fr_1fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <div>
                                                <div class="text-sm font-medium">{name}</div>
                                                <div class="text-xs text-iiz-cyan">{email}</div>
                                            </div>
                                            <div class="text-sm text-gray-700">{duration}</div>
                                            <div>
                                                <span class="badge badge-sm bg-blue-100 text-blue-700 border-none">{platform}</span>
                                            </div>
                                            <div class="flex items-center gap-1">
                                                <span class=rec_view.0><Icon icon=icondata::BsCameraVideoFill /></span>
                                                <span class="text-xs text-gray-500">{rec_view.1}</span>
                                            </div>
                                            <div class="text-sm text-gray-700">{agent}</div>
                                            <div>
                                                <div class="text-xs text-gray-500">{date}</div>
                                                <div class="text-xs text-gray-400">{time}</div>
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
// Export Log page -- export history with CTA buttons
// ---------------------------------------------------------------------------

#[component]
pub fn ExportLogPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ExportRecordItem>>("/activities/exports?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            // Header with title and CTA buttons
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-gray-800">"Export Log"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsDownload /></span>
                    "Export Calls"
                </button>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsDownload /></span>
                    "Export Texts"
                </button>
            </header>

            // Column headers
            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[0.8fr_1.2fr_0.6fr_0.7fr_0.7fr_1fr_1.2fr] gap-2 px-4 py-2 items-center">
                    <div class="col-header">"Export Type"</div>
                    <div class="col-header">"Date Range"</div>
                    <div class="col-header">"Format"</div>
                    <div class="col-header">"Status"</div>
                    <div class="col-header">"Rows"</div>
                    <div class="col-header">"Requested By"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            // Export rows with loading/error handling
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
                                    let export_type = e.export_type.clone().unwrap_or_default();
                                    let date_range = e.date_range.clone().unwrap_or_default();
                                    let requested_by = e.requested_by_id.clone().unwrap_or_default();
                                    let row_count = e.record_count.to_string();
                                    let created = fmt_date(&e.created_at);
                                    let is_complete = e.status == "Complete";
                                    let status_class = if is_complete {
                                        "badge badge-sm bg-green-100 text-green-700 border-none"
                                    } else {
                                        "badge badge-sm bg-yellow-100 text-yellow-700 border-none"
                                    };
                                    let format_class = if e.format == "CSV" {
                                        "badge badge-sm bg-gray-100 text-gray-600 border-none"
                                    } else {
                                        "badge badge-sm bg-red-100 text-red-600 border-none"
                                    };
                                    view! {
                                        <div class="activity-row grid grid-cols-[0.8fr_1.2fr_0.6fr_0.7fr_0.7fr_1fr_1.2fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                            <div class="flex items-center gap-2">
                                                <span class="text-sm font-medium text-gray-700">{export_type}</span>
                                            </div>
                                            <div class="text-sm text-gray-600">{date_range}</div>
                                            <div><span class=format_class>{e.format.clone()}</span></div>
                                            <div class="flex items-center gap-1">
                                                <span class=status_class>{e.status.clone()}</span>
                                                {if is_complete {
                                                    Some(view! {
                                                        <button class="btn btn-xs btn-ghost text-iiz-cyan">
                                                            <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsDownload /></span>
                                                        </button>
                                                    })
                                                } else {
                                                    None
                                                }}
                                            </div>
                                            <div class="text-sm text-gray-700">{row_count}</div>
                                            <div class="text-sm text-gray-700">{requested_by}</div>
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
