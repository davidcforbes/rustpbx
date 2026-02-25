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

#[component]
pub fn KnowledgeBanksPage() -> impl IntoView {
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

            // Search
            <div class="bg-white border-b border-gray-200 px-4 py-2">
                <div class="join">
                    <input type="text" placeholder="Search..." class="input input-sm input-bordered join-item w-64" />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
            </div>

            <div class="sticky top-0 bg-white border-b border-gray-200 z-10">
                <div class="grid grid-cols-[1fr_100px_120px_120px_120px_120px] gap-2 px-4 py-2 items-center">
                    <div class="col-header">"Name"</div>
                    <div class="col-header">"Documents"</div>
                    <div class="col-header">"Category"</div>
                    <div class="col-header">"Last Import"</div>
                    <div class="col-header">"Updated"</div>
                    <div class="col-header">"Created"</div>
                </div>
            </div>

            // Empty state
            <div class="flex-1 flex items-center justify-center">
                <div class="text-center py-12">
                    <span class="w-12 h-12 inline-flex text-gray-300 mx-auto mb-3"><Icon icon=icondata::BsDatabase /></span>
                    <p class="text-sm text-gray-400">"No knowledge banks found."</p>
                    <p class="text-xs text-gray-400 mt-1">"Create a knowledge bank to get started."</p>
                </div>
            </div>

            <div class="h-10 bg-white border-t border-gray-200 flex items-center px-4 text-sm text-gray-500 flex-shrink-0">
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

// ---------------------------------------------------------------------------
// Placeholder for AI Tools pages not yet built (kept for compatibility)
// ---------------------------------------------------------------------------

#[component]
pub fn AIToolsPlaceholderPage(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
) -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <div class="flex-1 flex items-center justify-center bg-iiz-gray-bg">
                <div class="text-center max-w-md">
                    <div class="w-16 h-16 rounded-full bg-iiz-cyan-light flex items-center justify-center mx-auto mb-4">
                        <span class="w-8 h-8 inline-flex text-iiz-cyan"><Icon icon=icondata::BsStars /></span>
                    </div>
                    <h2 class="text-xl font-semibold text-gray-700">{title}</h2>
                    <p class="text-sm text-gray-500 mt-2">{description}</p>
                </div>
            </div>
        </div>
    }
}
