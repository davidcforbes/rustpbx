use leptos::prelude::*;
use leptos_icons::Icon;

use crate::sections::activities::CallRecord;

// ---------------------------------------------------------------------------
// Detail panel tabs
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailTab {
    TextMessage,
    Contact,
    VisitorDetail,
    Score,
    Email,
    VoiceAnalysis,
    Flow,
    Reminder,
    Zoho,
    Script,
}

impl DetailTab {
    fn label(&self) -> &'static str {
        match self {
            Self::TextMessage => "Text Message",
            Self::Contact => "Contact",
            Self::VisitorDetail => "Visitor Detail",
            Self::Score => "Score",
            Self::Email => "Email",
            Self::VoiceAnalysis => "Voice Analysis",
            Self::Flow => "Flow",
            Self::Reminder => "Reminder",
            Self::Zoho => "Zoho",
            Self::Script => "Script",
        }
    }

    fn icon(&self) -> icondata::Icon {
        match self {
            Self::TextMessage => icondata::BsChatLeftTextFill,
            Self::Contact => icondata::BsPersonFill,
            Self::VisitorDetail => icondata::BsBarChartFill,
            Self::Score => icondata::BsStarFill,
            Self::Email => icondata::BsEnvelopeFill,
            Self::VoiceAnalysis => icondata::BsMicFill,
            Self::Flow => icondata::BsClockHistory,
            Self::Reminder => icondata::BsBellFill,
            Self::Zoho => icondata::BsGrid3x3GapFill,
            Self::Script => icondata::BsFileTextFill,
        }
    }
}

const ALL_TABS: [DetailTab; 10] = [
    DetailTab::TextMessage,
    DetailTab::Contact,
    DetailTab::VisitorDetail,
    DetailTab::Score,
    DetailTab::Email,
    DetailTab::VoiceAnalysis,
    DetailTab::Flow,
    DetailTab::Reminder,
    DetailTab::Zoho,
    DetailTab::Script,
];

// ---------------------------------------------------------------------------
// Main detail panel
// ---------------------------------------------------------------------------

#[component]
pub fn CallDetailPanel(
    call: CallRecord,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let active_tab = RwSignal::new(DetailTab::TextMessage);
    let call_for_header = call.clone();
    let call_for_content = call.clone();

    view! {
        // Backdrop + slide-out container
        <div class="fixed inset-0 z-50 flex">
            // Backdrop
            <div
                class="flex-1 bg-black/20"
                on:click=move |_| on_close.run(())
            ></div>

            // Panel
            <div class="w-[900px] bg-white shadow-2xl flex flex-col overflow-hidden animate-slide-in-right">
                // Header with call summary
                <DetailHeader call=call_for_header on_close=on_close />

                // Body: tabs + content
                <div class="flex flex-1 overflow-hidden">
                    // Tab sidebar
                    <nav class="w-40 border-r border-gray-100 py-4 px-2 flex-shrink-0 overflow-y-auto">
                        {ALL_TABS
                            .iter()
                            .map(|tab| {
                                let t = *tab;
                                let is_active = move || active_tab.get() == t;
                                let is_indented = matches!(
                                    t,
                                    DetailTab::Score | DetailTab::Email | DetailTab::Flow
                                );
                                view! {
                                    <button
                                        class=move || {
                                            let base = if is_indented {
                                                "detail-tab ml-4"
                                            } else {
                                                "detail-tab"
                                            };
                                            if is_active() {
                                                format!("{} active", base)
                                            } else {
                                                base.to_string()
                                            }
                                        }
                                        on:click=move |_| active_tab.set(t)
                                    >
                                        <span class="w-4 h-4 inline-flex flex-shrink-0">
                                            <Icon icon=t.icon() />
                                        </span>
                                        <span class="text-left">{t.label()}</span>
                                    </button>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </nav>

                    // Content area
                    <div class="flex-1 overflow-y-auto p-6">
                        <TabContent tab=active_tab call=call_for_content />
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Detail header
// ---------------------------------------------------------------------------

#[component]
fn DetailHeader(
    call: CallRecord,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let status_class = if call.status == "Answered" {
        "text-xs text-iiz-cyan"
    } else {
        "text-xs text-iiz-orange"
    };
    let audio_label = if call.has_audio { "audio" } else { "no audio" };
    let audio_icon_class = if call.has_audio {
        "w-4 h-4 inline-flex text-iiz-cyan"
    } else {
        "w-4 h-4 inline-flex text-gray-300"
    };
    let duration_text = format!("\u{25C0} {}", &call.duration);
    let status_text = format!("\u{25CF} {}", &call.status);
    let agent_color_style = format!("background-color:{}", &call.agent_color);

    view! {
        <div class="border-b border-gray-200 px-4 py-3 flex items-start gap-6 bg-white flex-shrink-0">
            // Contact info
            <div class="min-w-0">
                <div class="flex items-center gap-2">
                    <span class="w-4 h-4 inline-flex text-green-500">
                        <Icon icon=icondata::BsTelephoneFill />
                    </span>
                    <span class="font-medium text-sm truncate">{call.name.clone()}</span>
                </div>
                <div class="text-xs text-gray-500">{call.phone.clone()}</div>
                <div class="text-xs text-iiz-blue-link">{call.location.clone()}</div>
            </div>

            // Source
            <div class="text-center">
                <div class="text-sm font-medium">{call.source.clone()}</div>
                <div class="text-xs text-iiz-blue-link">{call.source_number.clone()}</div>
            </div>

            // Score
            <div class="flex flex-col items-center">
                <span class="w-5 h-5 inline-flex text-iiz-cyan">
                    <Icon icon=icondata::BsBarChartFill />
                </span>
                <span class="text-[10px] text-iiz-cyan">"Score"</span>
            </div>

            // Audio
            <div>
                <div class="flex items-center gap-1">
                    <span class={audio_icon_class}><Icon icon=icondata::BsVolumeUpFill /></span>
                    <span class="text-xs">{audio_label}</span>
                </div>
                <div class="text-xs text-gray-500">{duration_text}</div>
            </div>

            // Metrics
            <div>
                <div class="text-xs text-gray-500">{call.date.clone()}</div>
                <div class="text-xs text-gray-500">{call.time.clone()}</div>
                <div class={status_class}>{status_text}</div>
            </div>

            // Agent
            {if !call.agent.is_empty() {
                view! {
                    <div class="flex items-center gap-2">
                        <div
                            class="w-7 h-7 rounded-full text-white text-[10px] flex items-center justify-center"
                            style=agent_color_style
                        >
                            <span>{call.agent_initials.clone()}</span>
                        </div>
                        <div>
                            <div class="text-sm">{call.agent.clone()}</div>
                            <div class="text-xs text-iiz-blue-link">{call.automation.clone()}</div>
                        </div>
                    </div>
                }
                .into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            // Spacer + actions
            <div class="ml-auto flex items-center gap-1">
                <button class="btn btn-xs btn-ghost text-gray-400">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsEnvelope /></span>
                </button>
                <button class="btn btn-xs btn-ghost text-red-400">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsFlag /></span>
                </button>
                <button
                    class="btn btn-xs btn-ghost text-gray-400"
                    on:click=move |_| on_close.run(())
                >
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsXLg /></span>
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Tab content router
// ---------------------------------------------------------------------------

#[component]
fn TabContent(
    #[prop(into)] tab: Signal<DetailTab>,
    call: CallRecord,
) -> impl IntoView {
    let call = StoredValue::new(call);

    view! {
        {move || {
            let c = call.get_value();
            match tab.get() {
                DetailTab::TextMessage => TextMessageTab().into_any(),
                DetailTab::Contact => ContactTab(ContactTabProps { call: c }).into_any(),
                DetailTab::VisitorDetail => VisitorDetailTab().into_any(),
                DetailTab::Score => ScoreTab().into_any(),
                DetailTab::Email => EmailTab().into_any(),
                DetailTab::VoiceAnalysis => VoiceAnalysisTab(VoiceAnalysisTabProps { call: c }).into_any(),
                DetailTab::Flow => FlowTab().into_any(),
                DetailTab::Reminder => ReminderTab().into_any(),
                DetailTab::Zoho => PlaceholderTab(PlaceholderTabProps { name: "Zoho".into() }).into_any(),
                DetailTab::Script => PlaceholderTab(PlaceholderTabProps { name: "Script".into() }).into_any(),
            }
        }}
    }
}

// ---------------------------------------------------------------------------
// Text Message tab
// ---------------------------------------------------------------------------

#[component]
fn TextMessageTab() -> impl IntoView {
    view! {
        <div>
            <h3 class="text-sm font-semibold text-gray-700 mb-1">"Text Message"</h3>
            <p class="text-xs text-gray-400 mb-4">"Reply to the message conversation"</p>

            <div class="text-center mb-4">
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Load more"</a>
            </div>

            // Example chat bubbles
            <div class="space-y-3">
                <div class="text-center text-xs text-gray-400">"Mon, Sep 30, 2024 2:24 PM"</div>

                // Outbound message
                <div class="flex justify-end">
                    <div class="max-w-[70%] bg-iiz-cyan text-white text-sm px-4 py-2 rounded-2xl rounded-tr-sm">
                        "Hello! Thank you for calling. How can we help you today?"
                    </div>
                </div>

                // Inbound message
                <div class="flex justify-start">
                    <div class="max-w-[70%] bg-red-400 text-white text-sm px-4 py-2 rounded-2xl rounded-tl-sm">
                        "Hi, I'm calling about my appointment scheduled for next week."
                    </div>
                </div>

                <div class="text-center text-xs text-gray-400">"Mon, Sep 30, 2024 2:26 PM"</div>

                // Outbound
                <div class="flex justify-end">
                    <div class="max-w-[70%] bg-iiz-cyan text-white text-sm px-4 py-2 rounded-2xl rounded-tr-sm">
                        "Of course! Let me pull up your information. One moment please."
                    </div>
                </div>
            </div>

            // Reply input
            <div class="mt-6 flex gap-2">
                <input
                    type="text"
                    placeholder="Type a message..."
                    class="input input-sm input-bordered flex-1"
                />
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "Send"
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Contact tab
// ---------------------------------------------------------------------------

#[component]
fn ContactTab(call: CallRecord) -> impl IntoView {
    view! {
        <div>
            <h3 class="text-sm font-semibold text-gray-700 mb-1">"Contact"</h3>
            <p class="text-xs text-gray-400 mb-4">"Caller profile"</p>

            <div class="grid grid-cols-2 gap-4 max-w-2xl">
                <div>
                    <label class="label"><span class="label-text text-xs">"Contact Name"</span></label>
                    <input type="text" value=call.name class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Email"</span></label>
                    <input type="email" placeholder="email@example.com" class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Contact Number"</span></label>
                    <input type="text" value=call.phone class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Street"</span></label>
                    <input type="text" placeholder="Street address" class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"City"</span></label>
                    <input type="text" placeholder="City" class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"State"</span></label>
                    <input type="text" placeholder="State" class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Country"</span></label>
                    <input type="text" value="US" class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Postal Code"</span></label>
                    <input type="text" placeholder="ZIP" class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Contact Category"</span></label>
                    <select class="select select-sm select-bordered w-full">
                        <option selected>"Select category"</option>
                        <option>"Lead"</option>
                        <option>"Customer"</option>
                        <option>"Vendor"</option>
                    </select>
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Call Outcome"</span></label>
                    <select class="select select-sm select-bordered w-full">
                        <option selected>"Select outcome"</option>
                        <option>"Appointment Set"</option>
                        <option>"Follow-up Required"</option>
                        <option>"No Interest"</option>
                    </select>
                </div>
            </div>

            // Toggles row
            <div class="flex flex-wrap gap-6 mt-4">
                <label class="flex items-center gap-2 text-sm">
                    <input type="checkbox" class="toggle toggle-sm toggle-primary" />
                    "Appointment Set?"
                </label>
                <label class="flex items-center gap-2 text-sm">
                    <input type="checkbox" class="toggle toggle-sm toggle-primary" />
                    "Answered?"
                </label>
                <label class="flex items-center gap-2 text-sm">
                    <input type="checkbox" class="toggle toggle-sm toggle-primary" />
                    "NPS?"
                </label>
            </div>

            // Action buttons
            <div class="flex gap-2 mt-6">
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "Save Changes"
                </button>
                <button class="btn btn-sm btn-outline">
                    "Save & Close"
                </button>
            </div>

            // Notes section
            <div class="border-t border-gray-200 mt-6 pt-4">
                <h4 class="text-sm font-semibold text-gray-700 mb-2">"Notes"</h4>
                <textarea
                    placeholder="Add a note..."
                    class="textarea textarea-bordered w-full h-20 text-sm"
                ></textarea>
                <div class="flex gap-2 mt-2">
                    <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                        "Save Note"
                    </button>
                    <a class="text-xs text-iiz-cyan hover:underline cursor-pointer self-center">"Load more notes"</a>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Visitor Detail tab
// ---------------------------------------------------------------------------

#[component]
fn VisitorDetailTab() -> impl IntoView {
    view! {
        <div>
            <h3 class="text-sm font-semibold text-gray-700 mb-1">"Visitor Detail"</h3>
            <p class="text-xs text-gray-400 mb-6">"User/visitor activity for this contact"</p>

            <div class="text-center text-gray-400 py-8">
                <span class="w-12 h-12 inline-flex text-gray-300 mx-auto mb-2">
                    <Icon icon=icondata::BsBarChartFill />
                </span>
                <p class="text-sm">"No session data"</p>
            </div>

            <div class="flex gap-2 justify-center">
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "GA4 Activity Log"
                </button>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "Check Analytics (GA3)"
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Score tab
// ---------------------------------------------------------------------------

#[component]
fn ScoreTab() -> impl IntoView {
    view! {
        <div>
            <h3 class="text-sm font-semibold text-gray-700 mb-1">"Score"</h3>
            <p class="text-xs text-gray-400 mb-4">"Call revenue / sales rating"</p>

            <div class="max-w-md space-y-4">
                <div>
                    <label class="label"><span class="label-text text-xs">"Reporting Tag"</span></label>
                    <input type="text" placeholder="Enter reporting tag" class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Score Call"</span></label>
                    <select class="select select-sm select-bordered w-full">
                        <option selected>"No rating"</option>
                        <option>"1 - Poor"</option>
                        <option>"2 - Fair"</option>
                        <option>"3 - Good"</option>
                        <option>"4 - Very Good"</option>
                        <option>"5 - Excellent"</option>
                    </select>
                </div>
                <label class="flex items-center gap-2 text-sm">
                    <input type="checkbox" class="toggle toggle-sm toggle-primary" />
                    "Converted"
                </label>
            </div>

            <div class="flex justify-between max-w-md mt-6">
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "Save Changes"
                </button>
                <button class="btn btn-sm btn-outline text-red-500 border-red-300 hover:bg-red-50">
                    "Remove Score"
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Email tab
// ---------------------------------------------------------------------------

#[component]
fn EmailTab() -> impl IntoView {
    view! {
        <div>
            <h3 class="text-sm font-semibold text-gray-700 mb-1">"Email"</h3>
            <p class="text-xs text-gray-400 mb-4">"Send an email of this call"</p>

            <div class="max-w-lg space-y-4">
                <div>
                    <label class="label"><span class="label-text text-xs">"To"</span></label>
                    <input type="email" placeholder="recipient@example.com" class="input input-sm input-bordered w-full" />
                    <div class="flex gap-2 mt-1">
                        <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"CC Email"</a>
                        <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"BCC Email"</a>
                    </div>
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Subject"</span></label>
                    <input type="text" value="Follow-up from our call" class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Message"</span></label>
                    <textarea
                        placeholder="Type your message..."
                        class="textarea textarea-bordered w-full h-32 text-sm"
                    ></textarea>
                </div>
                <label class="flex items-center gap-2 text-sm">
                    <input type="checkbox" class="checkbox checkbox-sm checkbox-primary" />
                    "Include Call Record"
                </label>
                <p class="text-xs text-gray-400">"Optionally include a link of call details with the email including a recording if enabled."</p>
            </div>

            <div class="flex gap-2 mt-6">
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "Send Call"
                </button>
                <button class="btn btn-sm btn-outline">
                    "Close"
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Voice Analysis tab
// ---------------------------------------------------------------------------

#[component]
fn VoiceAnalysisTab(call: CallRecord) -> impl IntoView {
    view! {
        <div>
            <div class="flex items-center justify-between mb-4">
                <div>
                    <h3 class="text-sm font-semibold text-gray-700 mb-1">"Voice Analysis"</h3>
                    <p class="text-xs text-gray-400">"Transcription of the call"</p>
                </div>
                <button class="btn btn-sm btn-outline">"Access Logs"</button>
            </div>

            // Audio player
            <div class="bg-gray-50 rounded-lg p-4 mb-6">
                <div class="flex items-center gap-3">
                    <span class="text-xs text-gray-500">"1.00x"</span>
                    <button class="btn btn-circle btn-sm bg-iiz-blue-link hover:bg-iiz-blue-link/80 text-white border-none">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsPlayFill /></span>
                    </button>
                    <span class="text-xs text-gray-500">"00:00"</span>

                    // Waveform placeholder
                    <div class="flex-1 flex items-center gap-[1px] h-8">
                        {(0..80)
                            .map(|i| {
                                let height = ((i * 7 + 3) % 20) + 4;
                                let style = format!("height:{}px", height);
                                view! {
                                    <div
                                        class="w-[2px] bg-iiz-cyan/60 rounded-full flex-shrink-0"
                                        style=style
                                    ></div>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </div>

                    <span class="text-xs text-gray-500">{call.duration.clone()}</span>
                    <button class="btn btn-xs btn-ghost text-gray-400">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsVolumeUpFill /></span>
                    </button>
                </div>

                <div class="flex gap-4 mt-2 text-xs">
                    <a class="text-gray-500 hover:text-gray-700 cursor-pointer">"Download"</a>
                    <a class="text-iiz-cyan hover:underline cursor-pointer">"Share"</a>
                    <a class="text-red-500 hover:underline cursor-pointer">"Delete"</a>
                </div>
            </div>

            // Transcription
            <div>
                <h4 class="text-sm font-semibold text-gray-700 mb-2">"Transcription"</h4>
                <div class="bg-gray-50 rounded-lg p-4">
                    <p class="text-sm text-gray-400 italic">"No transcription available"</p>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Flow tab (timeline)
// ---------------------------------------------------------------------------

#[component]
fn FlowTab() -> impl IntoView {
    let events = vec![
        ("02:39 PM", "Abandoned Call Notification"),
        ("02:39 PM", "SMS for AHC"),
        ("02:39 PM", "Zoho Integration2"),
        ("02:39 PM", "Answer Call"),
        ("02:39 PM", "IVR Menu Selection"),
        ("02:38 PM", "Language Detection"),
        ("02:38 PM", "Initial Routing"),
        ("02:38 PM", "Smart Router Evaluation"),
        ("02:38 PM", "Agent Queue Assignment"),
        ("02:37 PM", "Call Received"),
        ("02:37 PM", "Caller ID Lookup"),
        ("02:37 PM", "DID Route Match"),
        ("02:37 PM", "Inbound Connection"),
    ];

    view! {
        <div>
            <div class="flex items-center justify-between mb-4">
                <div>
                    <h3 class="text-sm font-semibold text-gray-700 mb-1">"Flow"</h3>
                    <p class="text-xs text-gray-400">"Key events of the activity"</p>
                </div>
                <div class="flex gap-2">
                    <button class="btn btn-sm btn-outline gap-1">
                        <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsBugFill /></span>
                        "Debug Flow"
                    </button>
                    <button class="btn btn-sm btn-outline">"Agent Logs"</button>
                </div>
            </div>

            // Timeline
            <div class="relative pl-6">
                // Vertical line
                <div class="absolute left-2 top-0 bottom-0 w-[1px] bg-blue-200"></div>

                {events
                    .iter()
                    .map(|(time, event)| {
                        let t = (*time).to_string();
                        let e = (*event).to_string();
                        view! {
                            <div class="relative flex items-start gap-3 pb-4">
                                // Dot
                                <div class="absolute left-[-18px] top-1 w-2.5 h-2.5 rounded-full bg-blue-400 border-2 border-white"></div>
                                // Content
                                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex-shrink-0 w-16">{t}</a>
                                <span class="text-sm text-gray-700">{e}</span>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Reminder tab
// ---------------------------------------------------------------------------

#[component]
fn ReminderTab() -> impl IntoView {
    view! {
        <div>
            <h3 class="text-sm font-semibold text-gray-700 mb-1">"Callback Reminder"</h3>
            <p class="text-xs text-gray-400 mb-4">"Schedule a callback reminder"</p>

            <div class="max-w-md space-y-4">
                <div>
                    <label class="label"><span class="label-text text-xs">"How to remind"</span></label>
                    <select class="select select-sm select-bordered w-full">
                        <option>"Email"</option>
                        <option>"SMS"</option>
                    </select>
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Remind at"</span></label>
                    <input type="datetime-local" class="input input-sm input-bordered w-full" />
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Timezone"</span></label>
                    <select class="select select-sm select-bordered w-full">
                        <option>"(GMT-05:00) America/New_York"</option>
                        <option>"(GMT-06:00) America/Chicago"</option>
                        <option>"(GMT-07:00) America/Denver"</option>
                        <option>"(GMT-08:00) America/Los_Angeles"</option>
                    </select>
                </div>
                <div>
                    <label class="label"><span class="label-text text-xs">"Reminder message"</span></label>
                    <textarea
                        placeholder="Enter reminder message..."
                        class="textarea textarea-bordered w-full h-20 text-sm"
                    ></textarea>
                </div>
                <div class="flex flex-wrap gap-2">
                    <span class="text-xs text-gray-400">"Template variables:"</span>
                    <button class="btn btn-xs btn-ghost text-iiz-cyan">"Activity"</button>
                    <button class="btn btn-xs btn-ghost text-iiz-cyan">"Contact"</button>
                    <button class="btn btn-xs btn-ghost text-iiz-cyan">"Score"</button>
                    <button class="btn btn-xs btn-ghost text-iiz-cyan">"Enhanced"</button>
                </div>
            </div>

            <div class="flex gap-2 mt-6">
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "Save"
                </button>
                <button class="btn btn-sm btn-outline">
                    "Close"
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Generic placeholder for unbuilt tabs
// ---------------------------------------------------------------------------

#[component]
fn PlaceholderTab(name: String) -> impl IntoView {
    view! {
        <div class="text-center py-12">
            <span class="w-12 h-12 inline-flex text-gray-300 mx-auto mb-4">
                <Icon icon=icondata::BsInboxFill />
            </span>
            <h3 class="text-lg font-semibold text-gray-500">{name.clone()}</h3>
            <p class="text-gray-400 mt-2">"This integration is under development."</p>
        </div>
    }
}
