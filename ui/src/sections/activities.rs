use leptos::ev;
use leptos::prelude::*;
use leptos_icons::Icon;

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
}

// ---------------------------------------------------------------------------
// Mock data -- mirrors the prototype's Alpine.js `calls` array
// ---------------------------------------------------------------------------

fn mock_calls() -> Vec<CallRecord> {
    vec![
        CallRecord {
            id: "4045529975".into(),
            name: "Wilne Jean".into(),
            phone: "(773) 648-1494".into(),
            location: "Month Olive, NC".into(),
            source: "Mystery Shopper".into(),
            source_number: "(919) 436-4235".into(),
            source_name: "(Mystery Shopper)".into(),
            has_audio: true,
            duration: "00:08".into(),
            date: "Tue Feb 24th".into(),
            time: "02:39 PM".into(),
            status: "Answered".into(),
            agent: "Magaly Almaraz".into(),
            agent_initials: "MA".into(),
            agent_color: "#7b1fa2".into(),
            automation: "Answered Calls Lookup - Missed Calls Automation".into(),
            tags: vec!["agent_assigned".into()],
        },
        CallRecord {
            id: "4045529976".into(),
            name: "Jose Hipolito -Fb".into(),
            phone: "(408) 449-1936".into(),
            location: "San Jose/North Da, CA US".into(),
            source: "Google Organic".into(),
            source_number: "(949) 649-6378".into(),
            source_name: "(Santa Ana Office - Google My Business)".into(),
            has_audio: true,
            duration: "00:28".into(),
            date: "Tue Feb 24th".into(),
            time: "02:39 PM".into(),
            status: "Answered".into(),
            agent: "Cecilia Arrezola".into(),
            agent_initials: "CA".into(),
            agent_color: "#00897b".into(),
            automation: "Cecilia Arrezola".into(),
            tags: vec![],
        },
        CallRecord {
            id: "4045529977".into(),
            name: "Wilne Jean".into(),
            phone: "(773) 648-1494".into(),
            location: "Month Olive, NC".into(),
            source: "Mystery Shopper".into(),
            source_number: "(919) 436-4235".into(),
            source_name: "(Mystery Shopper)".into(),
            has_audio: true,
            duration: "00:12".into(),
            date: "Tue Feb 24th".into(),
            time: "02:39 PM".into(),
            status: "Answered".into(),
            agent: "Magaly Almaraz".into(),
            agent_initials: "MA".into(),
            agent_color: "#7b1fa2".into(),
            automation: "Answered Calls Lookup - Missed Calls Automation".into(),
            tags: vec![],
        },
        CallRecord {
            id: "4045529978".into(),
            name: "Jose Ramon Garcia Sanchez 25-45997".into(),
            phone: "(602) 930-7605".into(),
            location: "Phoenix, AZ US".into(),
            source: "Google Organic".into(),
            source_number: "(602) 838-6665".into(),
            source_name: "(Phoenix Office - Google My Business)".into(),
            has_audio: false,
            duration: "01:08".into(),
            date: "Tue Feb 24th".into(),
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
        },
        CallRecord {
            id: "4045529979".into(),
            name: "Jose Tores".into(),
            phone: "(786) 862-3629".into(),
            location: "FL US".into(),
            source: "Tiktok Organic".into(),
            source_number: "(657) 279-5506".into(),
            source_name: "(TikTok Organic)".into(),
            has_audio: false,
            duration: "01:13".into(),
            date: "Tue Feb 24th".into(),
            time: "02:39 PM".into(),
            status: "in progress".into(),
            agent: "Mario Rivas".into(),
            agent_initials: "MR".into(),
            agent_color: "#c62828".into(),
            automation: "Initial Language Selection".into(),
            tags: vec![
                "repeated caller".into(),
                "spanish ivr".into(),
                "in ice custody".into(),
                "sales call".into(),
            ],
        },
        CallRecord {
            id: "4045529980".into(),
            name: "Ismael Diosdado".into(),
            phone: "(919) 360-0772".into(),
            location: "Chapel Hill, NC US".into(),
            source: "Google Organic".into(),
            source_number: "(919) 725-8000".into(),
            source_name: "(Durham Office - Google My Business)".into(),
            has_audio: false,
            duration: "02:30".into(),
            date: "Tue Feb 24th".into(),
            time: "02:37 PM".into(),
            status: "in progress".into(),
            agent: "Celia Torres".into(),
            agent_initials: "CT".into(),
            agent_color: "#4527a0".into(),
            automation: String::new(),
            tags: vec![],
        },
        CallRecord {
            id: "4045529981".into(),
            name: "Clemente Aldahir Gonzalez".into(),
            phone: "(657) 520-8092".into(),
            location: "Santa Ana, CA US".into(),
            source: "Tiktok Organic".into(),
            source_number: "(657) 279-5506".into(),
            source_name: "(TikTok Organic)".into(),
            has_audio: false,
            duration: "02:49".into(),
            date: "Tue Feb 24th".into(),
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
        },
        CallRecord {
            id: "4045529982".into(),
            name: "Adolfo Angel Valerio Armijo 25-47527".into(),
            phone: "(323) 598-3978".into(),
            location: "Montebello, CA US".into(),
            source: "Google Organic".into(),
            source_number: "(949) 649-6378".into(),
            source_name: "(Santa Ana Office - Google My Business)".into(),
            has_audio: false,
            duration: "03:05".into(),
            date: "Tue Feb 24th".into(),
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
        },
    ]
}

// ---------------------------------------------------------------------------
// Activities side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn ActivitiesSideNav() -> impl IntoView {
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
                <a href="/activities/calls" class="side-nav-item active">"Calls"</a>
                <a href="/activities/texts" class="side-nav-item">"Texts"</a>
                <a href="/activities/forms" class="side-nav-item">"Forms"</a>
                <a href="/activities/chats" class="side-nav-item">"Chats"</a>
                <a href="/activities/faxes" class="side-nav-item">"Faxes"</a>
                <a href="/activities/videos" class="side-nav-item">"Videos"</a>
                <a href="/activities/export" class="side-nav-item">"Export Log"</a>
            </div>

            // Contacts group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsPeopleFill /></span>
                    "Contacts"
                </h3>
                <a href="/contacts/lists" class="side-nav-item">"Lists"</a>
                <a href="/contacts/blocked" class="side-nav-item">"Blocked Numbers"</a>
                <a href="/contacts/do-not-call" class="side-nav-item">"Do Not Call List"</a>
                <a href="/contacts/do-not-text" class="side-nav-item">"Do Not Text List"</a>
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
                <div class="grid grid-cols-[24px_2fr_1.5fr_0.8fr_0.6fr_0.8fr_1.2fr_1fr_48px] gap-2 px-4 py-2 items-center">
                    <div></div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPerson /></span>
                        "Contact"
                    </div>
                    <div class="col-header flex items-center gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsBuilding /></span>
                        "Source"
                    </div>
                    <div class="col-header">"Session Data"</div>
                    <div class="col-header">"Score"</div>
                    <div class="col-header">"Audio"</div>
                    <div class="col-header">"Metrics"</div>
                    <div class="col-header">"Routing"</div>
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
                <span>"Showing 1-8 of 3,694,942 results"</span>
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
    let status_class = if call.status == "Answered" {
        "text-xs mt-0.5 text-iiz-cyan"
    } else {
        "text-xs mt-0.5 text-iiz-orange"
    };

    let audio_icon_class = if call.has_audio {
        "w-4 h-4 inline-flex text-iiz-cyan"
    } else {
        "w-4 h-4 inline-flex text-gray-300"
    };

    let audio_label: &'static str = if call.has_audio { "audio" } else { "no audio" };

    let has_tags = !call.tags.is_empty();
    let has_agent = !call.agent.is_empty();
    let has_automation = !call.automation.is_empty();

    // Pre-compute all strings for the view
    let name = call.name.clone();
    let phone = call.phone.clone();
    let location = call.location.clone();
    let source = call.source.clone();
    let source_number = call.source_number.clone();
    let source_name = call.source_name.clone();
    let duration_text = format!("\u{25C0} {}", &call.duration);
    let date = call.date.clone();
    let time = call.time.clone();
    let status_text = format!("\u{25CF} {}", &call.status);
    let agent_name = call.agent.clone();
    let agent_initials = call.agent_initials.clone();
    let agent_color_style = format!("background-color:{}", &call.agent_color);
    let automation = call.automation.clone();
    let tags = call.tags.clone();

    view! {
        <div
            class=move || {
                let base = "activity-row grid grid-cols-[24px_2fr_1.5fr_0.8fr_0.6fr_0.8fr_1.2fr_1fr_48px] gap-2 px-4 py-3 items-start cursor-pointer";
                if selected.get() {
                    format!("{} bg-iiz-cyan-light", base)
                } else {
                    base.to_string()
                }
            }
            on:click=on_click
        >
            // Call icon
            <div class="pt-1">
                <span class="w-4 h-4 inline-flex text-green-500"><Icon icon=icondata::BsTelephoneFill /></span>
            </div>

            // Contact column
            <div>
                <div class="flex items-center gap-2">
                    <span class="font-medium text-sm">{name}</span>
                    <button class="text-gray-400 hover:text-gray-600">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsThreeDotsVertical /></span>
                    </button>
                </div>
                <div class="text-xs text-gray-500">{phone}</div>
                <div class="flex items-center gap-1 mt-0.5">
                    <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Call"</a>
                    <span class="text-gray-300">"|"</span>
                    <a class="text-xs text-blue-500 hover:underline cursor-pointer">{location}</a>
                </div>
                <div class="flex items-center gap-1 mt-0.5">
                    <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Edit"</a>
                </div>
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
            <div>
                <div class="flex items-center gap-1">
                    <span class="w-3.5 h-3.5 inline-flex text-gray-400"><Icon icon=icondata::BsBuilding /></span>
                    <span class="text-sm font-medium">{source}</span>
                </div>
                <div class="text-xs text-iiz-blue-link">{source_number}</div>
                <div class="text-xs text-gray-400">{source_name}</div>
            </div>

            // Session Data column
            <div class="flex items-center gap-1">
                <span class="w-4 h-4 inline-flex text-gray-400"><Icon icon=icondata::BsBarChartFill /></span>
            </div>

            // Score column
            <div>
                <button class="text-iiz-cyan hover:text-iiz-cyan/80 flex flex-col items-center">
                    <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsBarChartFill /></span>
                    <span class="text-[10px]">"Score"</span>
                </button>
            </div>

            // Audio column
            <div>
                <div class="flex items-center gap-1">
                    <span class={audio_icon_class}><Icon icon=icondata::BsVolumeUpFill /></span>
                    <span class="text-xs">{audio_label}</span>
                </div>
                <div class="text-xs text-gray-500">{duration_text}</div>
            </div>

            // Metrics column
            <div>
                <div class="text-xs text-gray-500 flex items-center gap-1">
                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsCalendar /></span>
                    <span>{date}</span>
                </div>
                <div class="text-xs text-gray-500 flex items-center gap-1">
                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsClock /></span>
                    <span>{time}</span>
                </div>
                <div class={status_class}>
                    {status_text}
                </div>
            </div>

            // Routing / Agent column
            <div>
                {if has_agent {
                    view! {
                        <div class="flex items-center gap-2">
                            <div class="avatar placeholder">
                                <div
                                    class="w-7 h-7 rounded-full text-white text-[10px] flex items-center justify-center"
                                    style=agent_color_style.clone()
                                >
                                    <span>{agent_initials.clone()}</span>
                                </div>
                            </div>
                            <span class="text-sm">{agent_name.clone()}</span>
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="flex items-center gap-2">
                            <div class="avatar placeholder">
                                <div
                                    class="w-7 h-7 rounded-full text-white text-[10px] flex items-center justify-center"
                                    style=agent_color_style.clone()
                                >
                                    <span>{agent_initials.clone()}</span>
                                </div>
                            </div>
                            <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"+ set agent"</a>
                        </div>
                    }
                    .into_any()
                }}
                {if has_automation {
                    Some(view! {
                        <div class="text-xs text-iiz-blue-link mt-0.5">{automation.clone()}</div>
                    })
                } else {
                    None
                }}
            </div>

            // Actions column
            <div class="flex items-center gap-1">
                <button class="btn btn-xs btn-ghost text-gray-400">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsEnvelope /></span>
                </button>
                <button class="btn btn-xs btn-ghost text-red-400">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsFlag /></span>
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Placeholder page for routes not yet built
// ---------------------------------------------------------------------------

#[component]
pub fn PlaceholderPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <FilterBar />
            <div class="flex-1 flex items-center justify-center">
                <div class="text-center">
                    <span class="w-16 h-16 inline-flex text-gray-300 mx-auto mb-4">
                        <Icon icon=icondata::BsInboxFill />
                    </span>
                    <h2 class="text-xl font-semibold text-gray-500">"Coming Soon"</h2>
                    <p class="text-gray-400 mt-2">"This section is under development."</p>
                </div>
            </div>
        </div>
    }
}
