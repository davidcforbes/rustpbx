use leptos::prelude::*;
use leptos_icons::Icon;

use crate::components::FilterBar;

// ---------------------------------------------------------------------------
// Flows side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn FlowsSideNav() -> impl IntoView {
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
                <a href="/flows/voice-menus" class="side-nav-item active">"Voice Menus"</a>
                <a href="/flows/queues" class="side-nav-item">"Queues"</a>
                <a href="/flows/smart-routers" class="side-nav-item">"Smart Routers"</a>
                <a href="/flows/geo-routers" class="side-nav-item">"Geo Routers"</a>
                <a href="/flows/schedules" class="side-nav-item">"Schedules"</a>
                <a href="/flows/agent-scripts" class="side-nav-item">"Agent Scripts"</a>
                <a href="/flows/routing-tables" class="side-nav-item">"Routing Tables"</a>
                <a href="/flows/voicemails" class="side-nav-item">"Voicemails"</a>
            </div>

            // Automation group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsLightningFill /></span>
                    "Automation"
                </h3>
                <a href="/flows/workflows" class="side-nav-item">"Workflows"</a>
                <a href="/flows/triggers" class="side-nav-item">"Triggers"</a>
                <a href="/flows/keyword-spotting" class="side-nav-item">"Keyword Spotting"</a>
                <a href="/flows/lambdas" class="side-nav-item">"Lambdas"</a>
                <a href="/flows/api-logs" class="side-nav-item">"API Logs"</a>
                <a href="/flows/global" class="side-nav-item">"Global"</a>
                <a href="/flows/webhooks" class="side-nav-item">"Webhooks"</a>
            </div>

            // Engagement group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsChatDotsFill /></span>
                    "Engagement"
                </h3>
                <a href="/flows/bulk-messages" class="side-nav-item">"Bulk Messages"</a>
                <a href="/flows/lead-reactor" class="side-nav-item">"LeadReactor"</a>
                <a href="/flows/smart-dialers" class="side-nav-item">"Smart Dialers"</a>
                <a href="/flows/form-reactor" class="side-nav-item">"FormReactor"</a>
                <a href="/flows/chat-widget" class="side-nav-item">"Chat Widget"</a>
                <a href="/flows/chat-ai" class="side-nav-item">
                    "ChatAI"
                    <span class="badge badge-xs bg-iiz-cyan text-white border-none ml-1">"BETA"</span>
                </a>
                <a href="/flows/dialogflow" class="side-nav-item">
                    "Dialogflow"
                    <span class="badge badge-xs bg-iiz-cyan text-white border-none ml-1">"BETA"</span>
                </a>
                <a href="/flows/reminders" class="side-nav-item">"Reminders"</a>
            </div>
        </nav>
    }
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct VoiceMenu {
    name: &'static str,
    greeting: bool,
    tag: &'static str,
    speech_rec: bool,
    speech_lang: &'static str,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct Queue {
    name: &'static str,
    repeat_callers: &'static str,
    distribute: &'static str,
    prompt: bool,
    caller_id: &'static str,
    schedule: &'static str,
    agents: Vec<&'static str>,
    no_answer: &'static str,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct SmartRouter {
    name: &'static str,
    if_rules: &'static str,
    then_action: &'static str,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct Trigger {
    name: &'static str,
    trigger_event: &'static str,
    run_on: &'static str,
    if_rules: &'static str,
    then_action: &'static str,
    runs_7d: &'static str,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct Webhook {
    name: &'static str,
    trigger_event: &'static str,
    callback_url: &'static str,
    method: &'static str,
    body_type: &'static str,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct BulkMessage {
    label: &'static str,
    phone: &'static str,
    body: &'static str,
    recipients: u32,
    send_time: &'static str,
    delivered: &'static str,
    status: &'static str,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct Schedule {
    name: &'static str,
    times: Vec<&'static str>,
    days: Vec<&'static str>,
    timezone: &'static str,
    updated: &'static str,
    created: &'static str,
}

#[derive(Clone, Debug)]
struct FormReactorEntry {
    name: &'static str,
    fields: &'static str,
    tracking_number: &'static str,
    updated: &'static str,
    created: &'static str,
    calls: u32,
}

// ---------------------------------------------------------------------------
// Mock data
// ---------------------------------------------------------------------------

fn mock_voice_menus() -> Vec<VoiceMenu> {
    vec![
        VoiceMenu { name: "CM Agencies VM", greeting: true, tag: "", speech_rec: false, speech_lang: "", updated: "2025-04-25 10:52 AM", created: "2024-01-15 09:30 AM" },
        VoiceMenu { name: "Voicemail Language Selection", greeting: true, tag: "", speech_rec: false, speech_lang: "", updated: "2025-06-12 02:15 PM", created: "2024-02-20 11:00 AM" },
        VoiceMenu { name: "English Voicemail", greeting: true, tag: "english", speech_rec: false, speech_lang: "", updated: "2025-06-12 02:20 PM", created: "2024-02-20 11:05 AM" },
        VoiceMenu { name: "Spanish Voicemail", greeting: true, tag: "spanish", speech_rec: false, speech_lang: "", updated: "2025-06-12 02:25 PM", created: "2024-02-20 11:10 AM" },
        VoiceMenu { name: "English IVR", greeting: true, tag: "", speech_rec: true, speech_lang: "en-US", updated: "2025-08-10 04:00 PM", created: "2024-03-15 10:00 AM" },
        VoiceMenu { name: "Spanish IVR", greeting: true, tag: "", speech_rec: true, speech_lang: "es-MX", updated: "2025-08-10 04:05 PM", created: "2024-03-15 10:05 AM" },
        VoiceMenu { name: "Initial Language Selection", greeting: true, tag: "", speech_rec: true, speech_lang: "multi", updated: "2025-09-01 09:00 AM", created: "2024-04-01 08:00 AM" },
    ]
}

fn mock_queues() -> Vec<Queue> {
    vec![
        Queue { name: "Collections FKM/RMs", repeat_callers: "Next available agents", distribute: "Simultaneously", prompt: true, caller_id: "Use Caller Number", schedule: "Business Hours", agents: vec!["Maria G.", "Carlos R.", "Ana T.", "+4 more"], no_answer: "VM Language Selection", updated: "2026-02-23", created: "2024-01-15" },
        Queue { name: "SYSTANGO TESTING", repeat_callers: "Next available agents", distribute: "Round Robin", prompt: false, caller_id: "Use Caller Number", schedule: "Always On", agents: vec!["Test Agent 1", "Test Agent 2"], no_answer: "Voicemail", updated: "2026-02-20", created: "2025-06-01" },
        Queue { name: "Customer Service Queue (Official)", repeat_callers: "Next available agents", distribute: "Simultaneously", prompt: true, caller_id: "Use Caller Number", schedule: "Business Hours", agents: vec!["Sarah L.", "James K.", "Lisa M.", "David P.", "+8 more"], no_answer: "English IVR", updated: "2026-02-22", created: "2023-11-01" },
        Queue { name: "Sales", repeat_callers: "Next available agents", distribute: "Longest idle", prompt: true, caller_id: "Use Tracking Number", schedule: "Sales Hours", agents: vec!["Mike S.", "Rachel W.", "+3 more"], no_answer: "Sales VM", updated: "2026-02-21", created: "2024-05-10" },
    ]
}

fn mock_smart_routers() -> Vec<SmartRouter> {
    vec![
        SmartRouter { name: "Current Client or Priming?", if_rules: "Contact Category includes any 'Current Client', 'Priming'", then_action: "CallQueue → Collections FKM/RMs, Tag Call", updated: "2026-01-15", created: "2024-06-01" },
        SmartRouter { name: "Priming Routing", if_rules: "Contact Category equals 'Priming'", then_action: "CallQueue → PRIMING CALLS ONLY", updated: "2025-12-20", created: "2024-06-15" },
        SmartRouter { name: "Check if New Lead or Current Client", if_rules: "Contact Category is empty OR equals 'New Lead'", then_action: "CallQueue → Sales, Tag Call 'new-lead'", updated: "2026-02-10", created: "2024-07-01" },
        SmartRouter { name: "VIP Client Routing", if_rules: "Contact Tag includes 'VIP'", then_action: "CallQueue → Customer Service Queue (Official)", updated: "2025-11-05", created: "2024-08-20" },
        SmartRouter { name: "After Hours Routing", if_rules: "Schedule 'Business Hours' is inactive", then_action: "Voice Menu → Voicemail Language Selection", updated: "2026-01-30", created: "2024-09-10" },
    ]
}

fn mock_triggers() -> Vec<Trigger> {
    vec![
        Trigger { name: "FKM Tagging - Outbound", trigger_event: "End event with all data ready", run_on: "All Tracking Numbers", if_rules: "Agent is any [Maria G., Carlos R., Ana T.]", then_action: "Tag Call 'fkm-outbound'", runs_7d: "1,689", updated: "2026-02-23", created: "2024-03-15" },
        Trigger { name: "#NA & Missed Calls Tickets", trigger_event: "End event with all data ready", run_on: "All Tracking Numbers", if_rules: "Call status is 'missed' OR 'no-answer'", then_action: "Create Ticket, Send Email", runs_7d: "1,074", updated: "2026-02-22", created: "2024-04-01" },
        Trigger { name: "AnswerHero Tickets", trigger_event: "End event with all data ready", run_on: "AnswerHero Numbers", if_rules: "Agent contains 'AnswerHero'", then_action: "Create Ticket, Webhook → Zapier", runs_7d: "474", updated: "2026-02-21", created: "2024-05-20" },
        Trigger { name: "Repeated Callers", trigger_event: "End event with all data ready", run_on: "All Tracking Numbers", if_rules: "Caller has called > 3 times in 24h", then_action: "Tag Call 'repeat-caller', Notify Manager", runs_7d: "1,429", updated: "2026-02-23", created: "2024-06-10" },
        Trigger { name: "Assigned Agent", trigger_event: "End event with all data ready", run_on: "All Tracking Numbers", if_rules: "Call answered by agent", then_action: "Update Contact → Assigned Agent", runs_7d: "23,274", updated: "2026-02-23", created: "2024-01-15" },
    ]
}

fn mock_webhooks() -> Vec<Webhook> {
    vec![
        Webhook { name: "4iiz AI Outbound SMS", trigger_event: "After text sent [outbound_text]", callback_url: "https://gfphuh6fxq...lambda-url.us-east-1.on.aws/", method: "POST", body_type: "Log Data", updated: "2026-02-23", created: "2025-08-15" },
        Webhook { name: "Offline-Comms-test1", trigger_event: "At end of call/form/chat [end]", callback_url: "https://abc123.ngrok.io/webhook", method: "POST", body_type: "Log Data", updated: "2026-02-20", created: "2025-09-01" },
        Webhook { name: "#Na Calls to Tickets", trigger_event: "Through a trigger [route]", callback_url: "https://hooks.zapier.com/hooks/catch/123...", method: "POST", body_type: "Log Data", updated: "2026-02-18", created: "2025-06-10" },
        Webhook { name: "AnswerHero", trigger_event: "Through a trigger [route]", callback_url: "https://hooks.zapier.com/hooks/catch/456...", method: "POST", body_type: "Log Data", updated: "2026-02-15", created: "2025-07-20" },
        Webhook { name: "4iiz AI SMS hook", trigger_event: "After text sent [outbound_text]", callback_url: "https://xyz789.lambda-url.us-east-1.on.aws/", method: "POST", body_type: "Log Data", updated: "2026-02-22", created: "2025-10-01" },
    ]
}

fn mock_bulk_messages() -> Vec<BulkMessage> {
    vec![
        BulkMessage { label: "Atencion: tienes una cita importante con Diener Law", phone: "(919) 725-8000", body: "Atencion: tienes una cita importante con Diener Law Group...", recipients: 61, send_time: "2026-02-24 10:15", delivered: "2026-02-24 10:16 AM", status: "Completed", updated: "2026-02-24", created: "2026-02-24" },
        BulkMessage { label: "Diener Law", phone: "(919) 725-8000", body: "Your appointment with Diener Law Group is confirmed...", recipients: 9, send_time: "2026-02-24 09:00", delivered: "2026-02-24 09:01 AM", status: "Completed", updated: "2026-02-24", created: "2026-02-24" },
        BulkMessage { label: "Atencion: tienes una cita importante con Diener Law", phone: "(919) 725-8000", body: "Atencion: tienes una cita importante con Diener Law Group...", recipients: 407, send_time: "2026-02-23 14:30", delivered: "2026-02-23 02:32 PM", status: "Completed", updated: "2026-02-23", created: "2026-02-23" },
        BulkMessage { label: "Weekly follow-up reminder", phone: "(855) 563-5818", body: "Hi! This is a friendly reminder about your upcoming...", recipients: 234, send_time: "2026-02-22 08:00", delivered: "2026-02-22 08:02 AM", status: "Completed", updated: "2026-02-22", created: "2026-02-22" },
        BulkMessage { label: "Payment reminder - February", phone: "(888) 361-3349", body: "Your payment is due. Please contact us at...", recipients: 156, send_time: "2026-02-20 10:00", delivered: "", status: "Failed", updated: "2026-02-20", created: "2026-02-20" },
    ]
}

fn mock_schedules() -> Vec<Schedule> {
    vec![
        Schedule { name: "Jalisco (PV/GDL Agents)", times: vec!["07:55 AM - 10:00 PM", "07:55 AM - 09:00 PM", "09:00 AM - 09:00 PM"], days: vec!["M T W Th F", "Sa", "S"], timezone: "Eastern Time (US & Canada)", updated: "2026-01-15", created: "2024-03-01" },
        Schedule { name: "Business Hours", times: vec!["08:00 AM - 06:00 PM"], days: vec!["M T W Th F"], timezone: "Eastern Time (US & Canada)", updated: "2026-02-10", created: "2023-11-15" },
        Schedule { name: "Sales Hours", times: vec!["09:00 AM - 08:00 PM", "10:00 AM - 04:00 PM"], days: vec!["M T W Th F", "Sa"], timezone: "Central Time (US & Canada)", updated: "2026-01-20", created: "2024-05-01" },
    ]
}

fn mock_form_reactors() -> Vec<FormReactorEntry> {
    vec![
        FormReactorEntry { name: "Permanent Residence Form", fields: "Name * Phone * Email * Submit", tracking_number: "+15622836869", updated: "2025-12-15", created: "2024-06-10", calls: 7 },
        FormReactorEntry { name: "Book - Spanish", fields: "Nombre * Teléfono * Correo * Enviar", tracking_number: "+19197258000", updated: "2025-11-20", created: "2024-03-01", calls: 5 },
        FormReactorEntry { name: "Questions", fields: "Name * Phone * Email * Message Submit", tracking_number: "+18553635818", updated: "2025-12-22", created: "2023-08-15", calls: 335 },
        FormReactorEntry { name: "Immigration Questionnaire", fields: "Name * Phone * Email * Country * Case Type * Submit (Gravity Forms)", tracking_number: "+18883613349", updated: "2025-12-20", created: "2024-01-20", calls: 120 },
        FormReactorEntry { name: "Footer Contact", fields: "Name * Phone * Email * Submit", tracking_number: "+18883998387", updated: "2025-12-22", created: "2023-05-01", calls: 747 },
    ]
}

// ---------------------------------------------------------------------------
// Voice Menus page
// ---------------------------------------------------------------------------

#[component]
pub fn VoiceMenusPage() -> impl IntoView {
    let menus = mock_voice_menus();

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
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Voice Menu"
                </button>
            </div>

            // Table
            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200">
                    <div class="grid grid-cols-[32px_1fr_60px_100px_80px_100px_140px_140px] gap-2 px-4 py-2 border-b border-gray-200">
                        <div class="col-header"></div>
                        <div class="col-header">"Name"</div>
                        <div class="col-header">"Greeting"</div>
                        <div class="col-header">"Tag this call"</div>
                        <div class="col-header">"Speech Rec"</div>
                        <div class="col-header">"Speech Lang"</div>
                        <div class="col-header">"Updated"</div>
                        <div class="col-header">"Created"</div>
                    </div>

                    {menus.into_iter().map(|m| {
                        view! {
                            <div class="activity-row grid grid-cols-[32px_1fr_60px_100px_80px_100px_140px_140px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                </button>
                                <div class="text-sm font-medium text-iiz-blue-link">{m.name}</div>
                                <div>
                                    {if m.greeting {
                                        view! { <span class="w-4 h-4 inline-flex text-gray-400"><Icon icon=icondata::BsVolumeUpFill /></span> }.into_any()
                                    } else {
                                        view! { <span class="text-xs text-gray-300">"-"</span> }.into_any()
                                    }}
                                </div>
                                <div>
                                    {if !m.tag.is_empty() {
                                        view! { <span class="badge badge-sm bg-gray-100 text-gray-600 border-none">{m.tag}</span> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>
                                <div>
                                    {if m.speech_rec {
                                        view! { <span class="text-green-500 text-sm">"✓"</span> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>
                                <div class="text-xs text-gray-500">{m.speech_lang}</div>
                                <div class="text-xs text-gray-500">{m.updated}</div>
                                <div class="text-xs text-gray-500">{m.created}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex items-center justify-between mt-3 text-sm text-gray-500">
                    <div class="flex items-center gap-2">
                        <span>"Per page:"</span>
                        <select class="select select-xs select-bordered">
                            <option selected>"10"</option>
                            <option>"25"</option>
                            <option>"50"</option>
                        </select>
                    </div>
                    <span>"7 Voice Menus"</span>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Queues page
// ---------------------------------------------------------------------------

#[component]
pub fn QueuesPage() -> impl IntoView {
    let queues = mock_queues();

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Queues"</h1>
                    <p class="text-sm text-gray-500">"Allow callers to wait for the next available agent"</p>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Queue"
                </button>
            </div>

            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
                    <div class="grid grid-cols-[32px_180px_130px_100px_50px_120px_100px_180px_140px_90px_90px] gap-1 px-4 py-2 border-b border-gray-200 min-w-max">
                        <div class="col-header"></div>
                        <div class="col-header">"Name"</div>
                        <div class="col-header">"Repeat Callers"</div>
                        <div class="col-header">"Distribute"</div>
                        <div class="col-header">"Prompt"</div>
                        <div class="col-header">"Caller ID"</div>
                        <div class="col-header">"Schedule"</div>
                        <div class="col-header">"Agents"</div>
                        <div class="col-header">"No Answer"</div>
                        <div class="col-header">"Updated"</div>
                        <div class="col-header">"Created"</div>
                    </div>

                    {queues.into_iter().map(|q| {
                        let agents = q.agents.clone();
                        view! {
                            <div class="activity-row grid grid-cols-[32px_180px_130px_100px_50px_120px_100px_180px_140px_90px_90px] gap-1 px-4 py-2.5 items-center cursor-pointer min-w-max">
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                </button>
                                <div class="text-sm font-medium text-iiz-blue-link">{q.name}</div>
                                <div class="text-xs text-gray-500">{q.repeat_callers}</div>
                                <div class="text-xs text-gray-500">{q.distribute}</div>
                                <div class="text-center">
                                    {if q.prompt {
                                        view! { <span class="text-green-500 text-sm">"✓"</span> }.into_any()
                                    } else {
                                        view! { <span class="text-gray-300 text-sm">"-"</span> }.into_any()
                                    }}
                                </div>
                                <div class="text-xs text-gray-500">{q.caller_id}</div>
                                <div><a class="text-xs text-iiz-cyan hover:underline cursor-pointer">{q.schedule}</a></div>
                                <div class="flex flex-wrap gap-1">
                                    {agents.iter().map(|a| {
                                        if a.starts_with('+') {
                                            view! { <span class="badge badge-xs bg-gray-100 text-gray-600 border-none">{*a}</span> }.into_any()
                                        } else {
                                            view! { <a class="text-xs text-iiz-blue-link hover:underline">{*a}</a> }.into_any()
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <div class="text-xs text-gray-500">{q.no_answer}</div>
                                <div class="text-xs text-gray-500">{q.updated}</div>
                                <div class="text-xs text-gray-500">{q.created}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex items-center justify-between mt-3 text-sm text-gray-500">
                    <div class="flex items-center gap-1">
                        <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                        <button class="btn btn-xs btn-ghost">"2"</button>
                        <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                    </div>
                    <span>"11 Queues"</span>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Smart Routers page
// ---------------------------------------------------------------------------

#[component]
pub fn SmartRoutersPage() -> impl IntoView {
    let routers = mock_smart_routers();

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Smart Routers"</h1>
                    <p class="text-sm text-gray-500">"Conditionally route callers based on their specific properties"</p>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Smart Router"
                </button>
            </div>

            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200">
                    <div class="grid grid-cols-[32px_200px_1fr_100px_100px] gap-2 px-4 py-2 border-b border-gray-200">
                        <div class="col-header"></div>
                        <div class="col-header">"Name"</div>
                        <div class="col-header">"Routes"</div>
                        <div class="col-header">"Updated"</div>
                        <div class="col-header">"Created"</div>
                    </div>

                    {routers.into_iter().map(|r| {
                        view! {
                            <div class="activity-row grid grid-cols-[32px_200px_1fr_100px_100px] gap-2 px-4 py-3 items-center cursor-pointer">
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                </button>
                                <div class="text-sm font-medium text-iiz-blue-link">{r.name}</div>
                                <div class="bg-gray-50 rounded px-3 py-2 text-xs">
                                    <div>
                                        <span class="font-semibold">"If all rules match: "</span>
                                        <span class="text-gray-600">{r.if_rules}</span>
                                    </div>
                                    <div class="mt-1">
                                        <span class="font-semibold text-iiz-cyan">"Then: "</span>
                                        <span class="text-gray-600">{r.then_action}</span>
                                    </div>
                                </div>
                                <div class="text-xs text-gray-500">{r.updated}</div>
                                <div class="text-xs text-gray-500">{r.created}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex items-center justify-end mt-3 text-sm text-gray-500">
                    <span>"5 Smart Routers"</span>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Schedules page
// ---------------------------------------------------------------------------

#[component]
pub fn SchedulesPage() -> impl IntoView {
    let schedules = mock_schedules();

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Schedules"</h1>
                    <p class="text-sm text-gray-500">"Recurring active times for agents, voice menus, call queues, and more"</p>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Schedule"
                </button>
            </div>

            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200">
                    <div class="grid grid-cols-[32px_200px_180px_120px_200px_100px_100px] gap-2 px-4 py-2 border-b border-gray-200">
                        <div class="col-header"></div>
                        <div class="col-header">"Name"</div>
                        <div class="col-header">"Times"</div>
                        <div class="col-header">"Days"</div>
                        <div class="col-header">"Time Zone"</div>
                        <div class="col-header">"Updated"</div>
                        <div class="col-header">"Created"</div>
                    </div>

                    {schedules.into_iter().map(|s| {
                        let times = s.times.clone();
                        let days = s.days.clone();
                        view! {
                            <div class="activity-row grid grid-cols-[32px_200px_180px_120px_200px_100px_100px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                </button>
                                <div class="text-sm font-medium text-iiz-blue-link">{s.name}</div>
                                <div class="flex flex-col gap-0.5">
                                    {times.iter().map(|t| view! { <span class="text-xs text-gray-600">{*t}</span> }).collect::<Vec<_>>()}
                                </div>
                                <div class="flex flex-col gap-0.5">
                                    {days.iter().map(|d| view! { <span class="text-xs text-gray-600">{*d}</span> }).collect::<Vec<_>>()}
                                </div>
                                <div class="text-xs text-gray-500">{s.timezone}</div>
                                <div class="text-xs text-gray-500">{s.updated}</div>
                                <div class="text-xs text-gray-500">{s.created}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex items-center justify-end mt-3 text-sm text-gray-500">
                    <span>"3 Schedules"</span>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Triggers page
// ---------------------------------------------------------------------------

#[component]
pub fn TriggersPage() -> impl IntoView {
    let triggers = mock_triggers();

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Triggers"</h1>
                    <p class="text-sm text-gray-500">"Trigger actions on your activities"</p>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Visual Workflows"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"API Logs"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Trigger"
                </button>
            </div>

            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
                    <div class="grid grid-cols-[32px_180px_180px_140px_1fr_80px_90px_90px] gap-1 px-4 py-2 border-b border-gray-200 min-w-max">
                        <div class="col-header"></div>
                        <div class="col-header">"Name"</div>
                        <div class="col-header">"Trigger"</div>
                        <div class="col-header">"Run"</div>
                        <div class="col-header">"Rules"</div>
                        <div class="col-header text-right">"Runs (7d)"</div>
                        <div class="col-header">"Updated"</div>
                        <div class="col-header">"Created"</div>
                    </div>

                    {triggers.into_iter().map(|t| {
                        view! {
                            <div class="activity-row grid grid-cols-[32px_180px_180px_140px_1fr_80px_90px_90px] gap-1 px-4 py-3 items-center cursor-pointer min-w-max">
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                </button>
                                <div class="text-sm font-medium text-iiz-blue-link">{t.name}</div>
                                <div class="text-xs text-gray-500">{t.trigger_event}</div>
                                <div class="text-xs text-gray-500">{t.run_on}</div>
                                <div class="bg-gray-50 rounded px-3 py-2 text-xs">
                                    <div>
                                        <span class="font-semibold">"If all rules match: "</span>
                                        <span class="text-gray-600">{t.if_rules}</span>
                                    </div>
                                    <div class="mt-1">
                                        <span class="font-semibold text-iiz-cyan">"Then: "</span>
                                        <span class="text-gray-600">{t.then_action}</span>
                                    </div>
                                </div>
                                <div class="text-sm font-bold text-gray-700 text-right">{t.runs_7d}</div>
                                <div class="text-xs text-gray-500">{t.updated}</div>
                                <div class="text-xs text-gray-500">{t.created}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex items-center justify-between mt-3 text-sm text-gray-500">
                    <div class="flex items-center gap-1">
                        <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                        <button class="btn btn-xs btn-ghost">"2"</button>
                        <button class="btn btn-xs btn-ghost">"3"</button>
                        <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                    </div>
                    <span>"26 Triggers"</span>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Webhooks page
// ---------------------------------------------------------------------------

#[component]
pub fn WebhooksPage() -> impl IntoView {
    let hooks = mock_webhooks();

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"Webhooks"</h1>
                    <p class="text-sm text-gray-500">"Send data to your servers"</p>
                </div>
                <div class="flex-1"></div>
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

            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
                    <div class="grid grid-cols-[32px_160px_180px_200px_60px_80px_90px_90px_100px] gap-1 px-4 py-2 border-b border-gray-200 min-w-max">
                        <div class="col-header"></div>
                        <div class="col-header">"Name"</div>
                        <div class="col-header">"Trigger"</div>
                        <div class="col-header">"Callback URL"</div>
                        <div class="col-header">"Method"</div>
                        <div class="col-header">"Body Type"</div>
                        <div class="col-header">"Updated"</div>
                        <div class="col-header">"Created"</div>
                        <div class="col-header">"Actions"</div>
                    </div>

                    {hooks.into_iter().map(|h| {
                        view! {
                            <div class="activity-row grid grid-cols-[32px_160px_180px_200px_60px_80px_90px_90px_100px] gap-1 px-4 py-2.5 items-center cursor-pointer min-w-max">
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                </button>
                                <div class="text-sm font-medium text-iiz-blue-link">{h.name}</div>
                                <div class="text-xs text-gray-500">{h.trigger_event}</div>
                                <div class="text-xs text-gray-400 truncate max-w-[200px]">{h.callback_url}</div>
                                <div class="text-xs text-gray-500">{h.method}</div>
                                <div class="text-xs text-gray-500">{h.body_type}</div>
                                <div class="text-xs text-gray-500">{h.updated}</div>
                                <div class="text-xs text-gray-500">{h.created}</div>
                                <div class="flex items-center gap-1">
                                    <button class="btn btn-xs bg-iiz-cyan text-white border-none">"Test"</button>
                                    <button class="btn btn-xs btn-ghost text-gray-400">"Deact."</button>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex items-center justify-between mt-3 text-sm text-gray-500">
                    <div class="flex items-center gap-1">
                        <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                        <button class="btn btn-xs btn-ghost">"2"</button>
                        <button class="btn btn-xs btn-ghost">"3"</button>
                        <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                    </div>
                    <span>"24 Webhooks"</span>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Bulk Messages page
// ---------------------------------------------------------------------------

#[component]
pub fn BulkMessagesPage() -> impl IntoView {
    let messages = mock_bulk_messages();

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

            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
                    <div class="grid grid-cols-[32px_200px_120px_200px_70px_120px_140px_80px_90px_90px] gap-1 px-4 py-2 border-b border-gray-200 min-w-max">
                        <div class="col-header"></div>
                        <div class="col-header">"Label"</div>
                        <div class="col-header">"Phone Numbers"</div>
                        <div class="col-header">"Body"</div>
                        <div class="col-header text-center">"Recipients"</div>
                        <div class="col-header">"Send Time"</div>
                        <div class="col-header">"Delivered On"</div>
                        <div class="col-header">"Status"</div>
                        <div class="col-header">"Updated"</div>
                        <div class="col-header">"Created"</div>
                    </div>

                    {messages.into_iter().map(|m| {
                        let status_class = match m.status {
                            "Completed" => "badge badge-sm bg-green-500 text-white border-none",
                            "Failed" => "badge badge-sm bg-red-500 text-white border-none",
                            "Sending" => "badge badge-sm bg-blue-500 text-white border-none",
                            "Pending" => "badge badge-sm bg-orange-400 text-white border-none",
                            _ => "badge badge-sm bg-gray-400 text-white border-none",
                        };
                        view! {
                            <div class="activity-row grid grid-cols-[32px_200px_120px_200px_70px_120px_140px_80px_90px_90px] gap-1 px-4 py-2.5 items-center cursor-pointer min-w-max">
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsArrowRepeat /></span>
                                </button>
                                <div class="text-sm font-medium text-iiz-blue-link truncate max-w-[200px]">{m.label}</div>
                                <div class="text-xs text-gray-500 whitespace-nowrap">{m.phone}</div>
                                <div class="text-xs text-gray-400 truncate max-w-[200px]">{m.body}</div>
                                <div class="text-sm font-bold text-gray-600 text-center">{m.recipients}</div>
                                <div class="text-xs text-gray-500">{m.send_time}</div>
                                <div class="text-xs text-gray-500">{m.delivered}</div>
                                <div><span class=status_class>{m.status}</span></div>
                                <div class="text-xs text-gray-500">{m.updated}</div>
                                <div class="text-xs text-gray-500">{m.created}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex items-center justify-between mt-3 text-sm text-gray-500">
                    <div class="flex items-center gap-1">
                        <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                        <button class="btn btn-xs btn-ghost">"2"</button>
                        <button class="btn btn-xs btn-ghost">"3"</button>
                        <span class="text-xs text-gray-400">"..."</span>
                        <button class="btn btn-xs btn-ghost">"169"</button>
                        <button class="btn btn-xs btn-ghost">"170"</button>
                        <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                    </div>
                    <span>"1,695 Bulk Messages"</span>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// FormReactor page
// ---------------------------------------------------------------------------

#[component]
pub fn FormReactorPage() -> impl IntoView {
    let forms = mock_form_reactors();

    view! {
        <div class="flex flex-col h-full">
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <div>
                    <h1 class="text-xl font-semibold text-iiz-dark">"FormReactors"</h1>
                    <p class="text-sm text-gray-500">"Embeddable forms that can trigger phone calls, text messages, and emails"</p>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Restore"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New FormReactor"
                </button>
            </div>

            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200">
                    <div class="grid grid-cols-[32px_200px_1fr_140px_90px_90px_60px] gap-2 px-4 py-2 border-b border-gray-200">
                        <div class="col-header"></div>
                        <div class="col-header">"Form Name"</div>
                        <div class="col-header">"Fields"</div>
                        <div class="col-header">"Tracking Number"</div>
                        <div class="col-header">"Updated"</div>
                        <div class="col-header">"Created"</div>
                        <div class="col-header text-center">"Calls"</div>
                    </div>

                    {forms.into_iter().map(|f| {
                        view! {
                            <div class="activity-row grid grid-cols-[32px_200px_1fr_140px_90px_90px_60px] gap-2 px-4 py-2.5 items-center cursor-pointer">
                                <button class="btn btn-xs btn-ghost text-gray-400">
                                    <span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                </button>
                                <div class="text-sm font-medium text-iiz-blue-link whitespace-nowrap">{f.name}</div>
                                <div class="bg-gray-50 rounded px-3 py-1.5 text-xs text-gray-600">{f.fields}</div>
                                <div class="text-xs text-gray-500 whitespace-nowrap">{f.tracking_number}</div>
                                <div class="text-xs text-gray-500">{f.updated}</div>
                                <div class="text-xs text-gray-500">{f.created}</div>
                                <div class="text-sm font-bold text-gray-600 text-center">{f.calls}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex items-center justify-between mt-3 text-sm text-gray-500">
                    <div class="flex items-center gap-1">
                        <button class="btn btn-xs bg-iiz-cyan text-white border-none">"1"</button>
                        <button class="btn btn-xs btn-ghost">"2"</button>
                        <button class="btn btn-xs btn-ghost">"3"</button>
                        <button class="btn btn-xs btn-ghost text-gray-400"><span class="w-3 h-3 inline-flex"><Icon icon=icondata::BsChevronRight /></span></button>
                    </div>
                    <span>"30 FormReactors"</span>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Placeholder for pages not yet fully built
// ---------------------------------------------------------------------------

#[component]
pub fn FlowsPlaceholderPage(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
) -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <FilterBar />
            <div class="flex-1 flex items-center justify-center">
                <div class="text-center max-w-md">
                    <span class="w-16 h-16 inline-flex text-gray-300 mx-auto mb-4">
                        <Icon icon=icondata::BsArrowLeftRight />
                    </span>
                    <h2 class="text-xl font-semibold text-gray-500">{title}</h2>
                    <p class="text-gray-400 mt-2">{description}</p>
                </div>
            </div>
        </div>
    }
}
