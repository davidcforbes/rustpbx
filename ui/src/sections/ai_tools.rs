use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

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
// AskAI page - trigger configuration with preset dropdown + workflows
// ---------------------------------------------------------------------------

#[component]
pub fn AskAIPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            // Breadcrumb header
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Triggers"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"General"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // Info banner
                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <h3 class="font-semibold text-iiz-dark">"AskAI"</h3>
                        <p class="text-sm text-gray-600 mt-1">
                            "AskAI allows you to ask any natural language question and get a concise ChatGPT powered answer, which can then be entered into a "
                            <a class="text-iiz-cyan hover:underline cursor-pointer">"custom field"</a>
                            " for easy reporting. Please note that at least one custom field must be created."
                        </p>
                        <p class="text-sm text-gray-600 mt-1">
                            "See our "
                            <a class="text-iiz-cyan hover:underline cursor-pointer">"knowledge base"</a>
                            " for more information."
                        </p>
                    </div>

                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>

                            <div class="space-y-4 mt-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Name"</label>
                                    <input type="text" class="input input-bordered w-full" placeholder="Enter trigger name" />
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Use a Preset"</label>
                                    <div class="dropdown">
                                        <button class="btn btn-sm btn-outline gap-2">
                                            <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGearFill /></span>
                                            "Select Preset"
                                        </button>
                                    </div>
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Tracking Numbers"</label>
                                    <button class="btn btn-sm btn-outline text-gray-400" disabled>
                                        "Edit Assigned Tracking Numbers"
                                    </button>
                                    <p class="text-xs text-gray-400 mt-1">"(save first to assign)"</p>
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Delay workflow"</label>
                                    <div class="flex items-center gap-2">
                                        <input type="number" value="0" class="input input-sm input-bordered w-20" />
                                        <select class="select select-sm select-bordered">
                                            <option selected>"seconds"</option>
                                            <option>"minutes"</option>
                                            <option>"hours"</option>
                                        </select>
                                    </div>
                                    <p class="text-xs text-gray-400 mt-1">"This allows you to delay the start of your workflow by the given amount of time."</p>
                                </div>
                            </div>

                            <div class="mt-6">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                        </div>
                    </div>

                    // Workflows card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <div class="flex items-center justify-between">
                                <div>
                                    <h2 class="card-title text-lg font-semibold">"Workflows"</h2>
                                    <p class="text-sm text-gray-500">"Perform actions in response to this trigger"</p>
                                </div>
                                <div class="flex items-center gap-3">
                                    <a class="text-xs text-gray-500 hover:underline cursor-pointer">"Switch to Visualization"</a>
                                    <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"+ Add Workflow"</button>
                                </div>
                            </div>

                            <div class="text-center py-8">
                                <p class="text-sm text-gray-500">
                                    <span class="font-medium">"No workflows added."</span>
                                    " Click the 'Add Workflow' button above to get started."
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
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
// Knowledge Banks page - empty data table
// ---------------------------------------------------------------------------

struct KnowledgeBankRow {
    name: &'static str,
    description: &'static str,
    documents: u32,
    category: &'static str,
    category_color: &'static str,
    status: &'static str,
    status_color: &'static str,
    size: &'static str,
    last_import: &'static str,
    updated: &'static str,
    created: &'static str,
    used_by: &'static str,
}

fn knowledge_bank_rows() -> Vec<KnowledgeBankRow> {
    vec![
        KnowledgeBankRow { name: "General Knowledge", description: "Company policies, procedures, and general info", documents: 45, category: "General", category_color: "badge-info", status: "Ready", status_color: "badge-success", size: "12.4 MB", last_import: "2025-02-24", updated: "2025-02-24", created: "2024-06-15", used_by: "ChatAI, VoiceAI" },
        KnowledgeBankRow { name: "Product FAQ", description: "Frequently asked questions about services", documents: 128, category: "Support", category_color: "badge-warning", status: "Ready", status_color: "badge-success", size: "8.7 MB", last_import: "2025-02-23", updated: "2025-02-23", created: "2024-07-20", used_by: "ChatAI" },
        KnowledgeBankRow { name: "Legal Templates", description: "Legal document templates and case references", documents: 312, category: "Legal", category_color: "badge-secondary", status: "Ready", status_color: "badge-success", size: "156.2 MB", last_import: "2025-02-22", updated: "2025-02-22", created: "2024-08-01", used_by: "VoiceAI" },
        KnowledgeBankRow { name: "Support Docs", description: "Technical support documentation and guides", documents: 87, category: "Support", category_color: "badge-warning", status: "Indexing", status_color: "badge-warning", size: "23.1 MB", last_import: "2025-02-24", updated: "2025-02-24", created: "2024-09-10", used_by: "ChatAI" },
        KnowledgeBankRow { name: "Training Materials", description: "Agent training guides and scripts", documents: 34, category: "Training", category_color: "badge-primary", status: "Ready", status_color: "badge-success", size: "5.8 MB", last_import: "2025-02-20", updated: "2025-02-20", created: "2025-01-05", used_by: "—" },
    ]
}

#[component]
pub fn KnowledgeBanksPage() -> impl IntoView {
    let banks = knowledge_bank_rows();
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

            <div class="flex-1 overflow-y-auto">
                // Summary cards
                <div class="grid grid-cols-4 gap-3 p-4">
                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Knowledge Banks"</div>
                        <div class="text-2xl font-bold text-iiz-dark mt-1">"5"</div>
                        <div class="text-xs text-green-600">"4 ready, 1 indexing"</div>
                    </div>
                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Total Documents"</div>
                        <div class="text-2xl font-bold text-iiz-dark mt-1">"606"</div>
                        <div class="text-xs text-gray-400">"Across all banks"</div>
                    </div>
                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"Storage Used"</div>
                        <div class="text-2xl font-bold text-iiz-dark mt-1">"206 MB"</div>
                        <div class="text-xs text-gray-400">"of 1 GB limit"</div>
                    </div>
                    <div class="bg-white rounded-lg border border-gray-200 p-3">
                        <div class="text-xs text-gray-500 uppercase tracking-wide">"AI Agents Using"</div>
                        <div class="text-2xl font-bold text-iiz-cyan mt-1">"3"</div>
                        <div class="text-xs text-gray-400">"ChatAI (2), VoiceAI (1)"</div>
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
                                {banks.iter().map(|b| {
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer">
                                            <td>
                                                <div class="flex items-center gap-2">
                                                    <span class="w-8 h-8 bg-iiz-cyan/10 rounded flex items-center justify-center flex-shrink-0">
                                                        <span class="w-4 h-4 inline-flex text-iiz-cyan"><Icon icon=icondata::BsDatabase /></span>
                                                    </span>
                                                    <div>
                                                        <div class="text-sm font-medium text-iiz-dark">{b.name}</div>
                                                        <div class="text-xs text-gray-400">{b.description}</div>
                                                    </div>
                                                </div>
                                            </td>
                                            <td class="text-sm text-center font-medium">{b.documents.to_string()}</td>
                                            <td><span class=format!("badge badge-sm {}", b.category_color)>{b.category}</span></td>
                                            <td>
                                                <div class="flex items-center gap-1">
                                                    {if b.status == "Indexing" {
                                                        view! { <span class="loading loading-spinner loading-xs text-warning"></span> }.into_any()
                                                    } else {
                                                        view! { <span class="w-2 h-2 bg-green-500 rounded-full inline-block"></span> }.into_any()
                                                    }}
                                                    <span class=format!("badge badge-sm {}", b.status_color)>{b.status}</span>
                                                </div>
                                            </td>
                                            <td class="text-sm text-right text-gray-600">{b.size}</td>
                                            <td class="text-xs text-gray-500">{b.last_import}</td>
                                            <td class="text-xs text-gray-500">{b.used_by}</td>
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
                            <p class="text-xs text-gray-400 mt-1">"PDF, DOCX, TXT, CSV, HTML — Max 50MB per file"</p>
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
                            // Indexing progress for "Support Docs"
                            <div class="p-3 bg-yellow-50 rounded-lg border border-yellow-100">
                                <div class="flex items-center justify-between mb-1">
                                    <span class="text-sm font-medium text-yellow-800">"Support Docs"</span>
                                    <span class="text-xs text-yellow-600">"72% complete"</span>
                                </div>
                                <div class="w-full bg-yellow-200 rounded-full h-2">
                                    <div class="bg-yellow-500 h-2 rounded-full" style="width: 72%"></div>
                                </div>
                                <p class="text-xs text-yellow-600 mt-1">"63 of 87 documents indexed"</p>
                            </div>

                            // Recently indexed
                            <h4 class="text-xs font-semibold text-gray-500 uppercase">"Recently Indexed"</h4>
                            <div class="space-y-1">
                                <div class="flex items-center gap-2 p-2 rounded hover:bg-gray-50">
                                    <span class="w-4 h-4 inline-flex text-red-500"><Icon icon=icondata::BsFilePdf /></span>
                                    <span class="text-sm text-gray-700 flex-1">"estate-planning-guide.pdf"</span>
                                    <span class="text-xs text-gray-400">"2.3 MB"</span>
                                    <span class="w-2 h-2 bg-green-500 rounded-full"></span>
                                </div>
                                <div class="flex items-center gap-2 p-2 rounded hover:bg-gray-50">
                                    <span class="w-4 h-4 inline-flex text-blue-500"><Icon icon=icondata::BsFileWord /></span>
                                    <span class="text-sm text-gray-700 flex-1">"client-intake-procedures.docx"</span>
                                    <span class="text-xs text-gray-400">"1.1 MB"</span>
                                    <span class="w-2 h-2 bg-green-500 rounded-full"></span>
                                </div>
                                <div class="flex items-center gap-2 p-2 rounded hover:bg-gray-50">
                                    <span class="w-4 h-4 inline-flex text-gray-500"><Icon icon=icondata::BsFileText /></span>
                                    <span class="text-sm text-gray-700 flex-1">"faq-responses-v3.txt"</span>
                                    <span class="text-xs text-gray-400">"45 KB"</span>
                                    <span class="w-2 h-2 bg-green-500 rounded-full"></span>
                                </div>
                                <div class="flex items-center gap-2 p-2 rounded hover:bg-gray-50">
                                    <span class="w-4 h-4 inline-flex text-green-500"><Icon icon=icondata::BsFileSpreadsheet /></span>
                                    <span class="text-sm text-gray-700 flex-1">"contact-scripts.csv"</span>
                                    <span class="text-xs text-gray-400">"890 KB"</span>
                                    <span class="loading loading-spinner loading-xs text-warning"></span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
                <span class="text-xs text-gray-400">"5 knowledge banks"</span>
                <div class="flex-1"></div>
                <span class="text-xs text-gray-400">"Per page:"</span>
                <select class="select select-xs select-bordered ml-1">
                    <option selected>"10"</option>
                    <option>"25"</option>
                    <option>"50"</option>
                </select>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// VoiceAI page - agent creation form
// ---------------------------------------------------------------------------

#[component]
pub fn VoiceAIPage() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"VoiceAI Agents"</span></li>
                        <li><span class="text-gray-500">"New"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"Name"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Feedback"</a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // Name card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Name"</h2>

                            <div class="space-y-4 mt-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Name your AI"</label>
                                    <div class="flex items-center gap-2">
                                        <input type="text" class="input input-bordered flex-1" placeholder="Enter agent name" />
                                        <span class="w-6 h-6 inline-flex text-red-400"><Icon icon=icondata::BsChatFill /></span>
                                    </div>
                                    <p class="text-xs text-gray-400 mt-1">"A reference for your AI Agent"</p>
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Description (optional)"</label>
                                    <input type="text" class="input input-bordered w-full" placeholder="Additional details about your AI Agent" />
                                    <p class="text-xs text-gray-400 mt-1">"Additional details to help with documentation"</p>
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Welcome message for callers"</label>
                                    <input type="text" class="input input-bordered w-full" value="Thank you for contacting us. How can I help you?" />
                                    <p class="text-xs text-gray-400 mt-1">"Your AI Agent will repeat this message when a customer first connects"</p>
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Instructions:"</label>
                                    <textarea class="textarea textarea-bordered w-full h-32" placeholder="Describe how VoiceAI should engage with callers, such as tone, behavior, and expected outcomes."></textarea>
                                    <p class="text-xs text-gray-400 mt-1">
                                        <a class="text-iiz-cyan hover:underline cursor-pointer">"Learn more about prompt engineering."</a>
                                    </p>
                                </div>
                            </div>

                            <div class="mt-6">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                        </div>
                    </div>

                    // Personality card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Personality"</h2>

                            <div class="mt-4">
                                <h3 class="text-sm font-semibold text-gray-700">"Select a Voice"</h3>
                                <p class="text-sm text-gray-500 mt-1">"Preview and choose from a variety of voice types."</p>
                                <p class="text-xs text-gray-400 mt-2">"Emotion Aware AI uses a curated set of voices optimized for real-time emotion detection."</p>

                                <div class="grid grid-cols-3 gap-3 mt-4">
                                    {["Allison", "Aria", "Davis", "Emily", "Guy", "Jenny"].into_iter().map(|name| {
                                        view! {
                                            <label class="flex items-center gap-3 p-3 rounded-lg border border-gray-200 hover:border-iiz-cyan cursor-pointer">
                                                <input type="radio" name="voice" class="radio radio-sm radio-primary" />
                                                <div>
                                                    <div class="text-sm font-medium">{name}</div>
                                                    <div class="text-xs text-gray-400">"en-US"</div>
                                                </div>
                                                <button class="ml-auto btn btn-xs btn-ghost text-gray-400">
                                                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsPlayFill /></span>
                                                </button>
                                            </label>
                                        }
                                    }).collect::<Vec<_>>()}
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
// ChatAI page - chat agent configuration (BETA)
// ---------------------------------------------------------------------------

#[component]
pub fn ChatAIPage() -> impl IntoView {
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
                <span class="badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200">"BETA"</span>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-3xl mx-auto p-6 space-y-6">
                    // General card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"General"</h2>

                            <div class="space-y-4 mt-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Name"</label>
                                    <input type="text" class="input input-bordered w-full" placeholder="Enter agent name" />
                                    <p class="text-xs text-gray-400 mt-1">"User facing - pick a customer facing name"</p>
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Description"</label>
                                    <input type="text" class="input input-bordered w-full" placeholder="Brief description of this chat agent" />
                                </div>
                            </div>

                            <div class="mt-6">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                        </div>
                    </div>

                    // Knowledge Banks card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Knowledge Banks"</h2>

                            <div class="space-y-4 mt-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Choose a Knowledge Bank"</label>
                                    <select class="select select-bordered w-full">
                                        <option selected disabled>"Select a knowledge bank..."</option>
                                        <option>"General FAQ"</option>
                                        <option>"Product Documentation"</option>
                                        <option>"Legal Resources"</option>
                                    </select>
                                </div>

                                <label class="flex items-center gap-3 cursor-pointer">
                                    <input type="checkbox" class="toggle toggle-sm" checked />
                                    <span class="text-sm">"Include Source"</span>
                                </label>

                                <div>
                                    <button class="btn btn-sm btn-outline">"Manage Knowledge Banks"</button>
                                </div>
                            </div>

                            <div class="mt-6">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                        </div>
                    </div>

                    // Instructions card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Instructions"</h2>

                            <div class="mt-4">
                                <textarea class="textarea textarea-bordered w-full h-40" placeholder="Provide instructions for how the AI chat agent should behave. Include tone, topics to cover, escalation rules, and any specific responses for common questions."></textarea>
                                <p class="text-xs text-gray-400 mt-1">"These instructions guide the AI agent's responses to customer messages."</p>
                            </div>

                            <div class="mt-6">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Save Changes"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

