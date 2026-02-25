use leptos::ev;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

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
// Texts page -- SMS/text message activity log
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct TextRecord {
    name: &'static str,
    phone: &'static str,
    preview: &'static str,
    source: &'static str,
    direction: &'static str,
    status: &'static str,
    date: &'static str,
    time: &'static str,
}

fn mock_texts() -> Vec<TextRecord> {
    vec![
        TextRecord {
            name: "Maria Guadalupe Torres",
            phone: "(602) 930-7605",
            preview: "Hola, necesito informacion sobre mi caso de inmigracion...",
            source: "(602) 838-6665",
            direction: "Inbound",
            status: "Delivered",
            date: "Tue Feb 24th",
            time: "02:41 PM",
        },
        TextRecord {
            name: "Jose Ramon Garcia",
            phone: "(408) 449-1936",
            preview: "Your appointment is confirmed for Thursday at 10:00 AM...",
            source: "(949) 649-6378",
            direction: "Outbound",
            status: "Delivered",
            date: "Tue Feb 24th",
            time: "02:38 PM",
        },
        TextRecord {
            name: "Ana Patricia Mendez",
            phone: "(786) 862-3629",
            preview: "Gracias por contactarnos. Un abogado se comunicara con...",
            source: "(657) 279-5506",
            direction: "Outbound",
            status: "Delivered",
            date: "Tue Feb 24th",
            time: "02:35 PM",
        },
        TextRecord {
            name: "Carlos Alberto Reyes",
            phone: "(323) 598-3978",
            preview: "Quiero saber el estado de mi caso numero 25-47521...",
            source: "(949) 649-6378",
            direction: "Inbound",
            status: "Delivered",
            date: "Tue Feb 24th",
            time: "02:30 PM",
        },
        TextRecord {
            name: "Rosa Elena Villarreal",
            phone: "(919) 360-0772",
            preview: "Reminder: Your consultation is tomorrow at 2:00 PM...",
            source: "(919) 725-8000",
            direction: "Outbound",
            status: "Failed",
            date: "Tue Feb 24th",
            time: "02:25 PM",
        },
        TextRecord {
            name: "Fernando Diaz Morales",
            phone: "(657) 520-8092",
            preview: "Necesito hablar con alguien urgente sobre mi situacion...",
            source: "(657) 279-5506",
            direction: "Inbound",
            status: "Pending",
            date: "Tue Feb 24th",
            time: "02:20 PM",
        },
    ]
}

#[component]
pub fn TextsPage() -> impl IntoView {
    let texts = mock_texts();

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

            // Text rows
            <div class="flex-1 overflow-y-auto">
                {texts.into_iter().map(|t| {
                    let dir_class = if t.direction == "Inbound" {
                        "badge badge-sm bg-blue-100 text-blue-700 border-none"
                    } else {
                        "badge badge-sm bg-purple-100 text-purple-700 border-none"
                    };
                    let status_class = match t.status {
                        "Delivered" => "badge badge-sm bg-green-100 text-green-700 border-none",
                        "Failed" => "badge badge-sm bg-red-100 text-red-700 border-none",
                        _ => "badge badge-sm bg-yellow-100 text-yellow-700 border-none",
                    };
                    view! {
                        <div class="activity-row grid grid-cols-[1.5fr_2fr_1fr_0.8fr_0.8fr_1fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <div>
                                <div class="text-sm font-medium">{t.name}</div>
                                <div class="text-xs text-gray-500">{t.phone}</div>
                            </div>
                            <div class="text-sm text-gray-600 truncate">{t.preview}</div>
                            <div class="text-xs text-iiz-cyan">{t.source}</div>
                            <div><span class=dir_class>{t.direction}</span></div>
                            <div><span class=status_class>{t.status}</span></div>
                            <div>
                                <div class="text-xs text-gray-500">{t.date}</div>
                                <div class="text-xs text-gray-400">{t.time}</div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Pagination
            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-6 of 12,847"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span>
                    </button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost">"3"</button>
                    <span class="text-xs text-gray-400">"..."</span>
                    <button class="btn btn-xs btn-ghost">"2142"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Forms page -- form submission activity log
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct FormRecord {
    name: &'static str,
    phone: &'static str,
    email: &'static str,
    form_name: &'static str,
    source: &'static str,
    tracking_number: &'static str,
    date: &'static str,
    time: &'static str,
    status: &'static str,
}

fn mock_forms() -> Vec<FormRecord> {
    vec![
        FormRecord {
            name: "Alejandra Ruiz Flores",
            phone: "(714) 555-0134",
            email: "alejandra.ruiz@gmail.com",
            form_name: "Free Consultation Request",
            source: "Google Ads - Immigration",
            tracking_number: "(949) 649-6378",
            date: "Tue Feb 24th",
            time: "02:45 PM",
            status: "New",
        },
        FormRecord {
            name: "Miguel Angel Hernandez",
            phone: "(602) 555-0198",
            email: "m.hernandez88@yahoo.com",
            form_name: "Contact Us - Spanish",
            source: "Google Organic",
            tracking_number: "(602) 838-6665",
            date: "Tue Feb 24th",
            time: "02:30 PM",
            status: "Contacted",
        },
        FormRecord {
            name: "Patricia Morales Vega",
            phone: "(919) 555-0276",
            email: "patricia.mv@hotmail.com",
            form_name: "Free Consultation Request",
            source: "TikTok Organic",
            tracking_number: "(657) 279-5506",
            date: "Tue Feb 24th",
            time: "01:55 PM",
            status: "New",
        },
        FormRecord {
            name: "Roberto Carlos Soto",
            phone: "(323) 555-0342",
            email: "r.soto.legal@gmail.com",
            form_name: "Case Status Inquiry",
            source: "Direct Traffic",
            tracking_number: "(949) 649-6378",
            date: "Tue Feb 24th",
            time: "01:20 PM",
            status: "Contacted",
        },
        FormRecord {
            name: "Lucia Esperanza Campos",
            phone: "(786) 555-0419",
            email: "lecampos@outlook.com",
            form_name: "Emergency Consultation",
            source: "Google Ads - Criminal",
            tracking_number: "(919) 725-8000",
            date: "Tue Feb 24th",
            time: "12:50 PM",
            status: "New",
        },
    ]
}

#[component]
pub fn FormsPage() -> impl IntoView {
    let forms = mock_forms();

    view! {
        <div class="flex flex-col h-full">
            // Header
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <FilterBar />
                <div class="flex-1"></div>
                <span class="text-sm text-gray-500">"30 FormReactors"</span>
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

            // Form rows
            <div class="flex-1 overflow-y-auto">
                {forms.into_iter().map(|f| {
                    let status_class = if f.status == "New" {
                        "badge badge-sm bg-blue-100 text-blue-700 border-none"
                    } else {
                        "badge badge-sm bg-green-100 text-green-700 border-none"
                    };
                    view! {
                        <div class="activity-row grid grid-cols-[1.8fr_1.2fr_1fr_1fr_1fr_0.7fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <div>
                                <div class="text-sm font-medium">{f.name}</div>
                                <div class="text-xs text-gray-500">{f.phone}</div>
                                <div class="text-xs text-iiz-cyan">{f.email}</div>
                            </div>
                            <div class="text-sm text-gray-700">{f.form_name}</div>
                            <div class="text-xs text-gray-600">{f.source}</div>
                            <div class="text-xs text-iiz-cyan">{f.tracking_number}</div>
                            <div>
                                <div class="text-xs text-gray-500">{f.date}</div>
                                <div class="text-xs text-gray-400">{f.time}</div>
                            </div>
                            <div><span class=status_class>{f.status}</span></div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Pagination
            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-5 of 1,243"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span>
                    </button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost">"3"</button>
                    <span class="text-xs text-gray-400">"..."</span>
                    <button class="btn btn-xs btn-ghost">"249"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Chats page -- live chat conversation activity log
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct ChatRecord {
    visitor: &'static str,
    visitor_detail: &'static str,
    channel: &'static str,
    messages: u32,
    duration: &'static str,
    agent: &'static str,
    status: &'static str,
    date: &'static str,
    time: &'static str,
}

fn mock_chats() -> Vec<ChatRecord> {
    vec![
        ChatRecord {
            visitor: "Maria Lopez",
            visitor_detail: "192.168.1.42",
            channel: "Web Chat",
            messages: 14,
            duration: "08:32",
            agent: "Cecilia Arrezola",
            status: "Active",
            date: "Tue Feb 24th",
            time: "02:41 PM",
        },
        ChatRecord {
            visitor: "Unknown Visitor",
            visitor_detail: "76.103.240.55",
            channel: "Web Chat",
            messages: 6,
            duration: "03:15",
            agent: "Mario Rivas",
            status: "Closed",
            date: "Tue Feb 24th",
            time: "02:20 PM",
        },
        ChatRecord {
            visitor: "Jorge Espinoza",
            visitor_detail: "(657) 520-8092",
            channel: "SMS",
            messages: 9,
            duration: "12:45",
            agent: "Israel Navarro",
            status: "Active",
            date: "Tue Feb 24th",
            time: "01:55 PM",
        },
        ChatRecord {
            visitor: "Elena Ramirez",
            visitor_detail: "10.0.0.128",
            channel: "Web Chat",
            messages: 3,
            duration: "01:20",
            agent: "Magaly Almaraz",
            status: "Closed",
            date: "Tue Feb 24th",
            time: "01:30 PM",
        },
    ]
}

#[component]
pub fn ChatsPage() -> impl IntoView {
    let chats = mock_chats();

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

            // Chat rows
            <div class="flex-1 overflow-y-auto">
                {chats.into_iter().map(|c| {
                    let channel_class = if c.channel == "Web Chat" {
                        "badge badge-sm bg-indigo-100 text-indigo-700 border-none"
                    } else {
                        "badge badge-sm bg-teal-100 text-teal-700 border-none"
                    };
                    let status_class = if c.status == "Active" {
                        "badge badge-sm bg-green-100 text-green-700 border-none"
                    } else {
                        "badge badge-sm bg-gray-100 text-gray-600 border-none"
                    };
                    let msg_count = c.messages.to_string();
                    view! {
                        <div class="activity-row grid grid-cols-[1.5fr_0.8fr_0.6fr_0.7fr_1fr_0.7fr_1fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <div>
                                <div class="text-sm font-medium">{c.visitor}</div>
                                <div class="text-xs text-gray-400">{c.visitor_detail}</div>
                            </div>
                            <div><span class=channel_class>{c.channel}</span></div>
                            <div class="text-sm text-gray-700">{msg_count}</div>
                            <div class="text-sm text-gray-600">{c.duration}</div>
                            <div class="text-sm text-gray-700">{c.agent}</div>
                            <div><span class=status_class>{c.status}</span></div>
                            <div>
                                <div class="text-xs text-gray-500">{c.date}</div>
                                <div class="text-xs text-gray-400">{c.time}</div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Pagination
            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-4 of 892"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span>
                    </button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost">"3"</button>
                    <span class="text-xs text-gray-400">"..."</span>
                    <button class="btn btn-xs btn-ghost">"223"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Faxes page -- fax activity log
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct FaxRecord {
    name: &'static str,
    number: &'static str,
    pages: u32,
    direction: &'static str,
    status: &'static str,
    date: &'static str,
    time: &'static str,
}

fn mock_faxes() -> Vec<FaxRecord> {
    vec![
        FaxRecord {
            name: "USCIS - Nebraska Service Center",
            number: "(800) 870-3676",
            pages: 4,
            direction: "Inbound",
            status: "Received",
            date: "Tue Feb 24th",
            time: "01:15 PM",
        },
        FaxRecord {
            name: "Superior Court of California",
            number: "(714) 834-2095",
            pages: 12,
            direction: "Outbound",
            status: "Sent",
            date: "Tue Feb 24th",
            time: "11:30 AM",
        },
        FaxRecord {
            name: "Immigration Court - Los Angeles",
            number: "(213) 894-2811",
            pages: 7,
            direction: "Outbound",
            status: "Failed",
            date: "Mon Feb 23rd",
            time: "04:45 PM",
        },
    ]
}

#[component]
pub fn FaxesPage() -> impl IntoView {
    let faxes = mock_faxes();

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

            // Fax rows
            <div class="flex-1 overflow-y-auto">
                {faxes.into_iter().map(|f| {
                    let dir_class = if f.direction == "Inbound" {
                        "badge badge-sm bg-blue-100 text-blue-700 border-none"
                    } else {
                        "badge badge-sm bg-purple-100 text-purple-700 border-none"
                    };
                    let status_class = match f.status {
                        "Received" => "badge badge-sm bg-green-100 text-green-700 border-none",
                        "Sent" => "badge badge-sm bg-green-100 text-green-700 border-none",
                        _ => "badge badge-sm bg-red-100 text-red-700 border-none",
                    };
                    let page_count = f.pages.to_string();
                    view! {
                        <div class="activity-row grid grid-cols-[1.8fr_0.5fr_0.8fr_0.8fr_1fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <div>
                                <div class="text-sm font-medium">{f.name}</div>
                                <div class="text-xs text-gray-500">{f.number}</div>
                            </div>
                            <div class="text-sm text-gray-700">{page_count}</div>
                            <div><span class=dir_class>{f.direction}</span></div>
                            <div><span class=status_class>{f.status}</span></div>
                            <div>
                                <div class="text-xs text-gray-500">{f.date}</div>
                                <div class="text-xs text-gray-400">{f.time}</div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Pagination
            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-3 of 156"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span>
                    </button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost">"3"</button>
                    <span class="text-xs text-gray-400">"..."</span>
                    <button class="btn btn-xs btn-ghost">"52"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Videos page -- video call activity log
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct VideoRecord {
    name: &'static str,
    email: &'static str,
    duration: &'static str,
    platform: &'static str,
    recording: bool,
    agent: &'static str,
    date: &'static str,
    time: &'static str,
}

fn mock_videos() -> Vec<VideoRecord> {
    vec![
        VideoRecord {
            name: "Ricardo Fuentes Ortega",
            email: "ricardo.fuentes@gmail.com",
            duration: "32:15",
            platform: "Zoom",
            recording: true,
            agent: "Magaly Almaraz",
            date: "Tue Feb 24th",
            time: "01:00 PM",
        },
        VideoRecord {
            name: "Veronica Salazar Cruz",
            email: "v.salazar.cruz@yahoo.com",
            duration: "18:42",
            platform: "Zoom",
            recording: true,
            agent: "Oswaldo Aguilera",
            date: "Tue Feb 24th",
            time: "11:00 AM",
        },
        VideoRecord {
            name: "Hector Manuel Rios",
            email: "hrios.legal@outlook.com",
            duration: "45:08",
            platform: "Zoom",
            recording: false,
            agent: "Celia Torres",
            date: "Mon Feb 23rd",
            time: "03:30 PM",
        },
    ]
}

#[component]
pub fn VideosPage() -> impl IntoView {
    let videos = mock_videos();

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

            // Video rows
            <div class="flex-1 overflow-y-auto">
                {videos.into_iter().map(|v| {
                    let rec_view = if v.recording {
                        ("w-4 h-4 inline-flex text-green-500", "Available")
                    } else {
                        ("w-4 h-4 inline-flex text-gray-300", "None")
                    };
                    view! {
                        <div class="activity-row grid grid-cols-[1.5fr_0.7fr_0.7fr_0.7fr_1fr_1fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <div>
                                <div class="text-sm font-medium">{v.name}</div>
                                <div class="text-xs text-iiz-cyan">{v.email}</div>
                            </div>
                            <div class="text-sm text-gray-700">{v.duration}</div>
                            <div>
                                <span class="badge badge-sm bg-blue-100 text-blue-700 border-none">{v.platform}</span>
                            </div>
                            <div class="flex items-center gap-1">
                                <span class=rec_view.0><Icon icon=icondata::BsCameraVideoFill /></span>
                                <span class="text-xs text-gray-500">{rec_view.1}</span>
                            </div>
                            <div class="text-sm text-gray-700">{v.agent}</div>
                            <div>
                                <div class="text-xs text-gray-500">{v.date}</div>
                                <div class="text-xs text-gray-400">{v.time}</div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Pagination
            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span>"Showing 1-3 of 47"</span>
                <div class="flex-1"></div>
                <div class="flex items-center gap-1">
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronLeft /></span>
                    </button>
                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                    <button class="btn btn-xs btn-ghost">"2"</button>
                    <button class="btn btn-xs btn-ghost">"3"</button>
                    <span class="text-xs text-gray-400">"..."</span>
                    <button class="btn btn-xs btn-ghost">"16"</button>
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Export Log page -- export history with CTA buttons
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct ExportRecord {
    export_type: &'static str,
    date_range: &'static str,
    format: &'static str,
    status: &'static str,
    rows: &'static str,
    requested_by: &'static str,
    created: &'static str,
}

fn mock_exports() -> Vec<ExportRecord> {
    vec![
        ExportRecord {
            export_type: "Calls",
            date_range: "Feb 1 - Feb 24, 2026",
            format: "CSV",
            status: "Complete",
            rows: "14,832",
            requested_by: "Magaly Almaraz",
            created: "Feb 24, 2026 02:30 PM",
        },
        ExportRecord {
            export_type: "Texts",
            date_range: "Feb 1 - Feb 24, 2026",
            format: "CSV",
            status: "Complete",
            rows: "12,847",
            requested_by: "Magaly Almaraz",
            created: "Feb 24, 2026 02:28 PM",
        },
        ExportRecord {
            export_type: "Calls",
            date_range: "Jan 1 - Jan 31, 2026",
            format: "PDF",
            status: "Complete",
            rows: "28,419",
            requested_by: "Cecilia Arrezola",
            created: "Feb 1, 2026 09:00 AM",
        },
        ExportRecord {
            export_type: "Forms",
            date_range: "Feb 1 - Feb 24, 2026",
            format: "CSV",
            status: "Processing",
            rows: "--",
            requested_by: "Israel Navarro",
            created: "Feb 24, 2026 02:45 PM",
        },
        ExportRecord {
            export_type: "Calls",
            date_range: "Dec 1 - Dec 31, 2025",
            format: "CSV",
            status: "Complete",
            rows: "31,205",
            requested_by: "Oswaldo Aguilera",
            created: "Jan 2, 2026 10:15 AM",
        },
    ]
}

#[component]
pub fn ExportLogPage() -> impl IntoView {
    let exports = mock_exports();

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

            // Export rows
            <div class="flex-1 overflow-y-auto">
                {exports.into_iter().map(|e| {
                    let status_class = if e.status == "Complete" {
                        "badge badge-sm bg-green-100 text-green-700 border-none"
                    } else {
                        "badge badge-sm bg-yellow-100 text-yellow-700 border-none"
                    };
                    let format_class = if e.format == "CSV" {
                        "badge badge-sm bg-gray-100 text-gray-600 border-none"
                    } else {
                        "badge badge-sm bg-red-100 text-red-600 border-none"
                    };
                    let is_complete = e.status == "Complete";
                    view! {
                        <div class="activity-row grid grid-cols-[0.8fr_1.2fr_0.6fr_0.7fr_0.7fr_1fr_1.2fr] gap-2 px-4 py-2.5 items-center cursor-pointer">
                            <div class="flex items-center gap-2">
                                <span class="text-sm font-medium text-gray-700">{e.export_type}</span>
                            </div>
                            <div class="text-sm text-gray-600">{e.date_range}</div>
                            <div><span class=format_class>{e.format}</span></div>
                            <div class="flex items-center gap-1">
                                <span class=status_class>{e.status}</span>
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
                            <div class="text-sm text-gray-700">{e.rows}</div>
                            <div class="text-sm text-gray-700">{e.requested_by}</div>
                            <div class="text-xs text-gray-500">{e.created}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
