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

#[component]
pub fn ChatWidgetPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            // Title bar
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3 flex-shrink-0">
                <h1 class="text-xl font-semibold text-iiz-dark">"Chat Widget"</h1>
                <div class="flex-1"></div>
                <button class="btn btn-xs btn-ghost text-gray-500">"User licenses"</button>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Info"</a>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">
                    "+ New Chat Widget"
                </button>
            </div>

            <div class="flex-1 overflow-y-auto p-6">
                <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
                    <div class="grid grid-cols-[140px_80px_60px_80px_100px_100px_80px_80px_100px_100px_60px] gap-1 px-4 py-2 border-b border-gray-200 min-w-max">
                        <div class="col-header">"Chat Widget"</div>
                        <div class="col-header">"Fields"</div>
                        <div class="col-header">"Active"</div>
                        <div class="col-header">"Status"</div>
                        <div class="col-header">"Tracking"</div>
                        <div class="col-header">"Routing"</div>
                        <div class="col-header">"Queue"</div>
                        <div class="col-header">"Agents"</div>
                        <div class="col-header">"Updated"</div>
                        <div class="col-header">"Created"</div>
                        <div class="col-header text-center">"Chats"</div>
                    </div>

                    // Empty state
                    <div class="py-16 text-center">
                        <p class="text-sm text-gray-400">"No chat widgets configured"</p>
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
