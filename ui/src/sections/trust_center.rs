use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

use crate::api::api_get;
use crate::api::types::{ListResponse, A2pCampaignItem, ComplianceRequirementItem, ComplianceApplicationItem, ComplianceAddressItem, CallerIdCnamItem, TollFreeRegistrationItem, VoiceRegistrationItem};

// ---------------------------------------------------------------------------
// Trust Center side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn TrustCenterSideNav() -> impl IntoView {
    let location = use_location();
    let active = |href: &'static str| {
        move || {
            if location.pathname.get() == href { "side-nav-item active" } else { "side-nav-item" }
        }
    };

    view! {
        <div class="px-4 pt-4 pb-2">
            <div class="flex items-center gap-2 text-iiz-cyan">
                <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsShieldCheck /></span>
                <span class="text-lg font-light">"Trust Center"</span>
            </div>
        </div>

        <nav class="px-2 pb-4">
            // US Outbound Compliance group
            <div class="mb-4">
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsFlag /></span>
                    "US Outbound Compliance"
                </h3>
                <a href="/trust-center/business" class=active("/trust-center/business")>"Business/Contact Info"</a>
                <a href="/trust-center/local-text" class=active("/trust-center/local-text")>"Local Text Messaging"</a>
                <a href="/trust-center/toll-free-text" class=active("/trust-center/toll-free-text")>"Toll Free Text Messaging"</a>
                <a href="/trust-center/voice-reg" class=active("/trust-center/voice-reg")>"Voice Registration"</a>
                <a href="/trust-center/caller-id" class=active("/trust-center/caller-id")>"Caller ID"</a>
            </div>

            // Global Compliance group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsGlobe /></span>
                    "Global Compliance"
                </h3>
                <a href="/trust-center/requirements" class=active("/trust-center/requirements")>"Requirements"</a>
                <a href="/trust-center/applications" class=active("/trust-center/applications")>"Applications"</a>
                <a href="/trust-center/addresses" class=active("/trust-center/addresses")>"Addresses"</a>
            </div>
        </nav>
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Truncate an ISO-8601 datetime to just the date portion.
fn fmt_date(iso: &str) -> String {
    if iso.len() >= 10 { iso[..10].to_string() } else { iso.to_string() }
}

/// Loading spinner placeholder.
fn loading_view() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-32">
            <span class="loading loading-spinner loading-lg text-iiz-cyan"></span>
        </div>
    }
}

/// Error message display.
fn error_view(msg: String) -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-32">
            <div class="text-center">
                <div class="text-red-500 text-lg font-semibold">"Error"</div>
                <div class="text-gray-500 mt-1">{msg}</div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Business/Contact Information page (main Trust Center page)
// ---------------------------------------------------------------------------

#[component]
pub fn BusinessInfoPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<A2pCampaignItem>>("/trust-center/a2p-campaigns?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            // Breadcrumb header
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Trust Center"</span></li>
                        <li><span class="text-gray-500">"View"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"Caller ID (CNAM)"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer">"Agency View"</a>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-4xl mx-auto p-6 space-y-6">

                    // CARD 1: Business/Contact Information
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <div class="flex items-start justify-between mb-4">
                                <h2 class="card-title text-lg font-semibold">"Business/Contact Information"</h2>
                                <div class="flex items-center gap-2">
                                    <span class="text-sm text-gray-500">"Trust Center Contact"</span>
                                    <select class="select select-sm select-bordered w-48">
                                        <option selected>"Chris Forbes"</option>
                                        <option>"Account Admin"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="grid grid-cols-[180px_1fr] gap-y-3 gap-x-4 text-sm">
                                <div class="text-gray-500 font-medium">"Legal Business Name"</div>
                                <div class="text-gray-800">"Diener Law, PA"</div>

                                <div class="text-gray-500 font-medium">"Address"</div>
                                <div class="text-gray-800">
                                    "3333 Jaeckle Dr, Suite 130"<br />"Wilmington, NC 28403, US"
                                </div>

                                <div class="text-gray-500 font-medium">"Company Type"</div>
                                <div class="text-gray-800">"Private"</div>

                                <div class="text-gray-500 font-medium">"Business Type"</div>
                                <div class="text-gray-800">"Corporation"</div>

                                <div class="text-gray-500 font-medium">"Industry Type"</div>
                                <div class="text-gray-800">"LEGAL"</div>

                                <div class="text-gray-500 font-medium">"EIN Number"</div>
                                <div class="text-gray-800">"271813765"</div>

                                <div class="text-gray-500 font-medium">"URL"</div>
                                <div><a class="text-iiz-blue-link hover:underline">"https://dienerlaw.net"</a></div>
                            </div>
                        </div>
                    </div>

                    // CARD 2: Local Text Messaging Campaigns
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Local Text Messaging Campaigns"</h2>
                            <p class="text-sm text-gray-500 mt-1">
                                "Register and manage campaigns required to send outbound text messages from local numbers to U.S. recipients. "
                                <a class="text-iiz-blue-link hover:underline cursor-pointer">"A2P 10DLC"</a>
                            </p>

                            <label class="flex items-center gap-2 mt-3 mb-4 cursor-pointer">
                                <input type="checkbox" class="toggle toggle-sm" />
                                <span class="text-sm text-gray-500">"Show expired campaigns"</span>
                            </label>

                            {move || match data.get() {
                                None => loading_view().into_any(),
                                Some(Err(e)) => error_view(e).into_any(),
                                Some(Ok(resp)) => {
                                    let items = resp.items.clone();
                                    let total = resp.pagination.total_items;
                                    view! {
                                        <>
                                            <div class="overflow-x-auto">
                                                <table class="table table-sm w-full">
                                                    <thead>
                                                        <tr class="border-b border-gray-200">
                                                            <th class="text-xs font-medium text-gray-500 uppercase w-16"></th>
                                                            <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                                            <th class="text-xs font-medium text-gray-500 uppercase">"Created"</th>
                                                            <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                                            <th class="text-xs font-medium text-gray-500 uppercase">"Assigned Numbers"</th>
                                                            <th class="text-xs font-medium text-gray-500 uppercase">"Cost"</th>
                                                            <th class="text-xs font-medium text-gray-500 uppercase">"Carrier"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {items.into_iter().map(|c| {
                                                            let nums = format!("({}/{})", c.assigned_numbers, c.max_numbers.unwrap_or(0));
                                                            let cost = c.monthly_cost.map(|v| format!("${v}/mo")).unwrap_or_default();
                                                            let carrier = c.carrier.clone().unwrap_or_default();
                                                            let created = fmt_date(&c.created_at);
                                                            view! {
                                                                <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                                    <td>
                                                                        <button class="btn btn-xs btn-ghost text-iiz-cyan">
                                                                            <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsEye /></span>
                                                                        </button>
                                                                    </td>
                                                                    <td class="font-medium text-sm">{c.campaign_name.clone()}</td>
                                                                    <td class="text-sm text-gray-600">{created}</td>
                                                                    <td>
                                                                        <span class="flex items-center gap-1 text-sm text-green-600">
                                                                            <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsCheckLg /></span>
                                                                            {c.status.clone()}
                                                                        </span>
                                                                    </td>
                                                                    <td>
                                                                        <div class="flex items-center gap-1.5">
                                                                            <span class="w-4 h-4 inline-flex text-green-600"><Icon icon=icondata::BsCheckLg /></span>
                                                                            <span class="text-sm">{nums}</span>
                                                                            <button class="text-gray-400 hover:text-gray-600">
                                                                                <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsPencil /></span>
                                                                            </button>
                                                                        </div>
                                                                    </td>
                                                                    <td class="text-sm text-gray-600">{cost}</td>
                                                                    <td class="text-sm text-gray-600">{carrier}</td>
                                                                </tr>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </tbody>
                                                </table>
                                            </div>

                                            <div class="flex items-center justify-between mt-4 pt-4 border-t border-gray-100">
                                                <div class="flex items-center gap-6 text-sm">
                                                    <div>
                                                        <span class="text-gray-500">"Local Text Registration Status:"</span>
                                                        <span class="text-green-600 font-medium ml-1">"approved"</span>
                                                    </div>
                                                    <div>
                                                        <span class="text-gray-500">"Campaigns:"</span>
                                                        <span class="font-medium ml-1">{format!("{}/50", total)}</span>
                                                    </div>
                                                </div>
                                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Add Campaigns"</button>
                                            </div>
                                        </>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>

                    // CARD 3: Toll-Free Text Messaging Campaign
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Toll-Free Text Messaging Campaign"</h2>
                            <p class="text-sm text-gray-500 mt-1">
                                "Register your toll-free numbers to send text messages to U.S. and Canadian phone numbers. Also known as Toll-Free A2P."
                            </p>
                            <p class="text-sm text-gray-600 mt-3">
                                "Register your business with carriers to send text messages from toll-free numbers. Registration is free."
                            </p>
                            <div class="mt-4">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Manage Toll-Free Messaging"</button>
                            </div>
                        </div>
                    </div>

                    // CARD 4: Outbound Calling Verification
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <div class="flex items-start justify-between">
                                <div>
                                    <h2 class="card-title text-lg font-semibold">"Outbound Calling Verification"</h2>
                                    <p class="text-sm text-gray-500 mt-1">
                                        "Register your business for outbound calling to reduce call blocking and spam labeling. Also known as STIR/SHAKEN."
                                    </p>
                                </div>
                                <span class="text-sm font-medium flex-shrink-0">
                                    "Status: "<span class="text-green-600">"APPROVED"</span>
                                </span>
                            </div>
                            <p class="text-sm text-gray-600 mt-3">
                                "Your business is verified for outbound calling to U.S. numbers, reducing call blocking and spam labeling by carriers. To further improve answer rates, register your numbers with the "
                                <a class="text-iiz-blue-link hover:underline cursor-pointer">"Free Caller Registry"</a>
                                "."
                            </p>
                        </div>
                    </div>

                    // CARD 5: Caller ID (CNAM)
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <div class="flex items-start justify-between">
                                <h2 class="card-title text-lg font-semibold">"Caller ID (CNAM)"</h2>
                                <a class="text-sm text-iiz-blue-link hover:underline cursor-pointer">"Learn more"</a>
                            </div>
                            <p class="text-sm text-gray-600 mt-2">
                                "Display a custom business name when placing outbound calls from your tracking numbers. This can improve answer rates on supported networks. Caller ID display depends on the recipient's carrier and device settings and is not guaranteed on every call."
                            </p>
                            <div class="mt-4">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Manage Caller ID"</button>
                            </div>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Local Text Messaging page - A2P 10DLC campaign management
// ---------------------------------------------------------------------------

#[component]
pub fn LocalTextPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<A2pCampaignItem>>("/trust-center/a2p-campaigns?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Trust Center"</span></li>
                        <li><span class="text-gray-500">"US Outbound Compliance"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"Local Text Messaging"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-4xl mx-auto p-6 space-y-6">
                    // Campaign list card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"A2P 10DLC Campaigns"</h2>
                            <p class="text-sm text-gray-500 mt-1">
                                "Manage your local text messaging campaigns. Each campaign must be registered and approved before sending messages."
                            </p>

                            {move || match data.get() {
                                None => loading_view().into_any(),
                                Some(Err(e)) => error_view(e).into_any(),
                                Some(Ok(resp)) => {
                                    let items = resp.items.clone();
                                    view! {
                                        <div class="overflow-x-auto mt-4">
                                            <table class="table table-sm w-full">
                                                <thead>
                                                    <tr class="border-b border-gray-200">
                                                        <th class="text-xs font-medium text-gray-500 uppercase w-16"></th>
                                                        <th class="text-xs font-medium text-gray-500 uppercase">"Name"</th>
                                                        <th class="text-xs font-medium text-gray-500 uppercase">"Created"</th>
                                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                                        <th class="text-xs font-medium text-gray-500 uppercase">"Assigned Numbers"</th>
                                                        <th class="text-xs font-medium text-gray-500 uppercase">"Cost"</th>
                                                        <th class="text-xs font-medium text-gray-500 uppercase">"Carrier"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {items.into_iter().map(|c| {
                                                        let nums = format!("({}/{})", c.assigned_numbers, c.max_numbers.unwrap_or(0));
                                                        let cost = c.monthly_cost.map(|v| format!("${v}/mo")).unwrap_or_default();
                                                        let carrier = c.carrier.clone().unwrap_or_default();
                                                        let created = fmt_date(&c.created_at);
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                                <td>
                                                                    <button class="btn btn-xs btn-ghost text-iiz-cyan">
                                                                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsEye /></span>
                                                                    </button>
                                                                </td>
                                                                <td class="font-medium text-sm">{c.campaign_name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{created}</td>
                                                                <td>
                                                                    <span class="flex items-center gap-1 text-sm text-green-600">
                                                                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsCheckLg /></span>
                                                                        {c.status.clone()}
                                                                    </span>
                                                                </td>
                                                                <td class="text-sm text-gray-600">{nums}</td>
                                                                <td class="text-sm text-gray-600">{cost}</td>
                                                                <td class="text-sm text-gray-600">{carrier}</td>
                                                            </tr>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>

                    // Add Campaign form card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Add Campaign"</h2>

                            <div class="space-y-4 mt-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Campaign Name"</label>
                                    <input type="text" class="input input-bordered w-full" placeholder="Enter campaign name" />
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Use Case"</label>
                                    <select class="select select-bordered w-full">
                                        <option selected disabled>"Select use case..."</option>
                                        <option>"Marketing"</option>
                                        <option>"Customer Care"</option>
                                        <option>"Account Notifications"</option>
                                        <option>"Delivery Notifications"</option>
                                        <option>"Fraud Alert"</option>
                                        <option>"Mixed"</option>
                                    </select>
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Description"</label>
                                    <input type="text" class="input input-bordered w-full" placeholder="Brief description of your campaign" />
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Sample Messages"</label>
                                    <textarea class="textarea textarea-bordered w-full h-24" placeholder="Provide 2-3 sample messages that represent this campaign..."></textarea>
                                    <p class="text-xs text-gray-400 mt-1">"Include examples of messages you plan to send."</p>
                                </div>
                            </div>

                            <div class="mt-6">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Submit Campaign"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Toll-Free Text Messaging page - toll-free registration
// ---------------------------------------------------------------------------

#[component]
pub fn TollFreeTextPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<TollFreeRegistrationItem>>("/trust-center/toll-free?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Trust Center"</span></li>
                        <li><span class="text-gray-500">"US Outbound Compliance"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"Toll Free Text Messaging"</span></li>
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
                    // Status card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-full bg-orange-100 flex items-center justify-center">
                                    <span class="w-5 h-5 inline-flex text-orange-500"><Icon icon=icondata::BsExclamationTriangle /></span>
                                </div>
                                <div>
                                    <h2 class="text-lg font-semibold text-gray-800">"Not Registered"</h2>
                                    <p class="text-sm text-gray-500">"Your toll-free numbers are not yet registered for text messaging."</p>
                                </div>
                            </div>
                            <p class="text-sm text-gray-600 mt-3">
                                "Toll-free text messaging registration is required to send text messages from toll-free numbers to U.S. and Canadian phone numbers. Registration is free and typically takes 2-3 business days."
                            </p>
                        </div>
                    </div>

                    // Registration form
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Begin Registration"</h2>

                            <div class="space-y-4 mt-4">
                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Business Name"</label>
                                    <input type="text" class="input input-bordered w-full" value="Diener Law, PA" />
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Contact Name"</label>
                                    <input type="text" class="input input-bordered w-full" placeholder="Primary contact name" />
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Phone"</label>
                                    <input type="text" class="input input-bordered w-full" placeholder="+1 (555) 000-0000" />
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Use Case Description"</label>
                                    <textarea class="textarea textarea-bordered w-full h-24" placeholder="Describe how you will use toll-free text messaging..."></textarea>
                                </div>

                                <div>
                                    <label class="text-sm font-medium text-gray-700 block mb-1">"Monthly Message Volume"</label>
                                    <select class="select select-bordered w-full">
                                        <option selected disabled>"Select estimated volume..."</option>
                                        <option>"Under 1,000"</option>
                                        <option>"1,000 - 10,000"</option>
                                        <option>"10,000 - 100,000"</option>
                                        <option>"100,000+"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="mt-6">
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Begin Registration"</button>
                            </div>
                        </div>
                    </div>

                    // Existing registrations
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Existing Registrations"</h2>
                            <div class="overflow-x-auto mt-4">
                                <table class="table table-sm w-full">
                                    <thead>
                                        <tr class="border-b border-gray-200">
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Number"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Business"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Created"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || match data.get() {
                                            None => loading_view().into_any(),
                                            Some(Err(e)) => error_view(e).into_any(),
                                            Some(Ok(resp)) => {
                                                let items = resp.items.clone();
                                                view! {
                                                    <>
                                                        {items.into_iter().map(|r| {
                                                            let status_badge = match r.status.as_str() {
                                                                "approved" | "active" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                                                "pending" => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                                                _ => "badge badge-sm bg-gray-100 text-gray-500 border-gray-200",
                                                            };
                                                            let created = fmt_date(&r.created_at);
                                                            view! {
                                                                <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                                    <td class="text-sm font-mono text-gray-700">{r.number.as_deref().unwrap_or("\u{2014}").to_string()}</td>
                                                                    <td class="text-sm text-gray-600">{r.business_name.as_deref().unwrap_or("\u{2014}").to_string()}</td>
                                                                    <td><span class=status_badge>{r.status.clone()}</span></td>
                                                                    <td class="text-sm text-gray-600">{created}</td>
                                                                </tr>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </>
                                                }.into_any()
                                            }
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Voice Registration page - STIR/SHAKEN status
// ---------------------------------------------------------------------------

#[component]
pub fn VoiceRegPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<VoiceRegistrationItem>>("/trust-center/voice-registrations?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Trust Center"</span></li>
                        <li><span class="text-gray-500">"US Outbound Compliance"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"Voice Registration"</span></li>
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
                    // Status card
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-full bg-green-100 flex items-center justify-center">
                                    <span class="w-5 h-5 inline-flex text-green-600"><Icon icon=icondata::BsCheckCircleFill /></span>
                                </div>
                                <div>
                                    <h2 class="text-lg font-semibold text-gray-800">"APPROVED"</h2>
                                    <p class="text-sm text-gray-500">"Your business is verified for STIR/SHAKEN outbound calling."</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Business info summary
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Verified Business Information"</h2>

                            <div class="grid grid-cols-[160px_1fr] gap-y-3 gap-x-4 text-sm mt-4">
                                <div class="text-gray-500 font-medium">"Business Name"</div>
                                <div class="text-gray-800">"Diener Law, PA"</div>

                                <div class="text-gray-500 font-medium">"EIN"</div>
                                <div class="text-gray-800">"27-1813765"</div>

                                <div class="text-gray-500 font-medium">"Address"</div>
                                <div class="text-gray-800">"3333 Jaeckle Dr, Suite 130, Wilmington, NC 28403"</div>
                            </div>

                            <div class="mt-6">
                                <button class="btn btn-sm btn-outline">"Re-verify"</button>
                            </div>
                        </div>
                    </div>

                    // History table
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Verification History"</h2>

                            <div class="overflow-x-auto mt-4">
                                <table class="table table-sm w-full">
                                    <thead>
                                        <tr class="border-b border-gray-200">
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Date"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Business"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Attestation"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || match data.get() {
                                            None => loading_view().into_any(),
                                            Some(Err(e)) => error_view(e).into_any(),
                                            Some(Ok(resp)) => {
                                                let items = resp.items.clone();
                                                view! {
                                                    <>
                                                        {items.into_iter().map(|r| {
                                                            let status_badge = match r.status.as_str() {
                                                                "approved" | "active" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                                                "pending" => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                                                "rejected" => "badge badge-sm bg-red-100 text-red-700 border-red-200",
                                                                _ => "badge badge-sm bg-gray-100 text-gray-500 border-gray-200",
                                                            };
                                                            let created = fmt_date(&r.created_at);
                                                            view! {
                                                                <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                                    <td class="text-sm text-gray-600">{created}</td>
                                                                    <td class="text-sm text-gray-600">{r.business_name.as_deref().unwrap_or("\u{2014}").to_string()}</td>
                                                                    <td><span class=status_badge>{r.status.clone()}</span></td>
                                                                    <td class="text-sm text-gray-600">{r.attestation_level.as_deref().unwrap_or("\u{2014}").to_string()}</td>
                                                                </tr>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </>
                                                }.into_any()
                                            }
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Caller ID page - CNAM management
// ---------------------------------------------------------------------------

#[component]
pub fn CallerIdPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<CallerIdCnamItem>>("/numbers/caller-id?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full overflow-y-auto">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <div class="breadcrumbs text-sm">
                    <ul>
                        <li><span class="text-gray-500">"Trust Center"</span></li>
                        <li><span class="text-gray-500">"US Outbound Compliance"</span></li>
                        <li><span class="text-iiz-cyan font-medium">"Caller ID"</span></li>
                    </ul>
                </div>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="max-w-4xl mx-auto p-6 space-y-6">
                    // Info
                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <p class="text-sm text-gray-600">
                            "Caller ID (CNAM) displays your business name to recipients when making outbound calls. Display depends on the recipient\u{2019}s carrier and device settings and is not guaranteed on every call. Updates may take up to 48 hours to propagate."
                        </p>
                    </div>

                    // CNAM table
                    <div class="card bg-white border border-gray-200">
                        <div class="card-body p-6">
                            <h2 class="card-title text-lg font-semibold">"Caller ID Numbers"</h2>

                            <div class="overflow-x-auto mt-4">
                                <table class="table table-sm w-full">
                                    <thead>
                                        <tr class="border-b border-gray-200">
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Number"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Current CNAM"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase">"Updated"</th>
                                            <th class="text-xs font-medium text-gray-500 uppercase"></th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || match data.get() {
                                            None => loading_view().into_any(),
                                            Some(Err(e)) => error_view(e).into_any(),
                                            Some(Ok(resp)) => {
                                                let items = resp.items.clone();
                                                view! {
                                                    <>
                                                        {items.into_iter().map(|r| {
                                                            let status_class = match r.status.as_str() {
                                                                "active" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                                                "pending" => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                                                _ => "badge badge-sm bg-gray-100 text-gray-500 border-gray-200",
                                                            };
                                                            let cnam_display = r.display_name.clone().unwrap_or_else(|| "Not Set".to_string());
                                                            let cnam_class = if cnam_display == "Not Set" { "text-sm text-gray-400 italic" } else { "text-sm text-gray-800 font-medium" };
                                                            let updated = fmt_date(&r.updated_at);
                                                            view! {
                                                                <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                                    <td class="text-sm font-mono text-gray-700">{r.number.clone()}</td>
                                                                    <td class=cnam_class>{cnam_display}</td>
                                                                    <td><span class=status_class>{r.status.clone()}</span></td>
                                                                    <td class="text-sm text-gray-600">{updated}</td>
                                                                    <td>
                                                                        <button class="btn btn-xs btn-outline text-iiz-cyan border-iiz-cyan">"Update CNAM"</button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </>
                                                }.into_any()
                                            }
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Requirements page - Global compliance requirements
// ---------------------------------------------------------------------------

#[component]
pub fn RequirementsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ComplianceRequirementItem>>("/trust-center/requirements?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Requirements"</h1>
                <p class="text-xs text-gray-400">"Global regulatory compliance requirements"</p>
                <div class="flex-1"></div>
                <a class="text-xs text-iiz-cyan hover:underline cursor-pointer flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsInfoCircle /></span>
                    "Info"
                </a>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Country"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Requirement"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Documentation"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || match data.get() {
                                        None => loading_view().into_any(),
                                        Some(Err(e)) => error_view(e).into_any(),
                                        Some(Ok(resp)) => {
                                            let items = resp.items.clone();
                                            view! {
                                                <>
                                                    {items.into_iter().map(|r| {
                                                        let status_badge = match r.status.as_str() {
                                                            "completed" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                                            "in_progress" => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                                            _ => "badge badge-sm bg-gray-100 text-gray-500 border-gray-200",
                                                        };
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                                <td class="text-sm font-medium">{r.country.as_deref().unwrap_or("\u{2014}").to_string()}</td>
                                                                <td class="text-sm text-gray-600">{r.name.clone()}</td>
                                                                <td><span class=status_badge>{r.status.clone()}</span></td>
                                                                <td class="text-xs text-gray-500">{r.description.as_deref().unwrap_or("\u{2014}").to_string()}</td>
                                                            </tr>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </>
                                            }.into_any()
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Applications page - Regulatory applications
// ---------------------------------------------------------------------------

#[component]
pub fn ApplicationsPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ComplianceApplicationItem>>("/trust-center/applications?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Applications"</h1>
                <p class="text-xs text-gray-400">"Regulatory applications for international communications"</p>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Application"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Application"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Country"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Status"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Submitted"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Updated"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || match data.get() {
                                        None => loading_view().into_any(),
                                        Some(Err(e)) => error_view(e).into_any(),
                                        Some(Ok(resp)) => {
                                            let items = resp.items.clone();
                                            view! {
                                                <>
                                                    {items.into_iter().map(|r| {
                                                        let status_badge = match r.status.as_str() {
                                                            "approved" => "badge badge-sm bg-green-100 text-green-700 border-green-200",
                                                            "pending" => "badge badge-sm bg-yellow-100 text-yellow-700 border-yellow-200",
                                                            "rejected" => "badge badge-sm bg-red-100 text-red-700 border-red-200",
                                                            _ => "badge badge-sm bg-gray-100 text-gray-500 border-gray-200",
                                                        };
                                                        let submitted = r.submitted_at.as_deref().map(fmt_date).unwrap_or_else(|| "\u{2014}".to_string());
                                                        let updated = fmt_date(&r.updated_at);
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                                <td class="text-sm font-medium">{r.name.clone()}</td>
                                                                <td class="text-sm text-gray-600">{r.country.as_deref().unwrap_or("\u{2014}").to_string()}</td>
                                                                <td><span class=status_badge>{r.status.clone()}</span></td>
                                                                <td class="text-sm text-gray-600">{submitted}</td>
                                                                <td class="text-sm text-gray-600">{updated}</td>
                                                            </tr>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </>
                                            }.into_any()
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Addresses page - Business addresses
// ---------------------------------------------------------------------------

#[component]
pub fn AddressesPage() -> impl IntoView {
    let data = LocalResource::new(|| async move {
        api_get::<ListResponse<ComplianceAddressItem>>("/trust-center/addresses?page=1&per_page=25").await
    });

    view! {
        <div class="flex flex-col h-full">
            <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
                <h1 class="text-lg font-semibold text-iiz-dark">"Addresses"</h1>
                <p class="text-xs text-gray-400">"Business addresses for regulatory compliance"</p>
                <div class="flex-1"></div>
                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"New Address"</button>
            </header>

            <div class="flex-1 overflow-y-auto bg-iiz-gray-bg">
                <div class="p-4">
                    <div class="card bg-white border border-gray-200">
                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr class="border-b border-gray-200">
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Label"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Address"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Country"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Verified"</th>
                                        <th class="text-xs font-medium text-gray-500 uppercase">"Updated"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || match data.get() {
                                        None => loading_view().into_any(),
                                        Some(Err(e)) => error_view(e).into_any(),
                                        Some(Ok(resp)) => {
                                            let items = resp.items.clone();
                                            view! {
                                                <>
                                                    {items.into_iter().map(|r| {
                                                        let label = r.label.as_deref().unwrap_or("\u{2014}").to_string();
                                                        let mut addr_parts = vec![r.street_line1.clone(), r.city.clone()];
                                                        if let Some(ref st) = r.state { addr_parts.push(st.clone()); }
                                                        if let Some(ref pc) = r.postal_code { addr_parts.push(pc.clone()); }
                                                        let full_addr = addr_parts.join(", ");
                                                        let updated = fmt_date(&r.updated_at);
                                                        let verified = r.is_verified;
                                                        view! {
                                                            <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                                <td class="text-sm font-medium">{label}</td>
                                                                <td class="text-sm text-gray-600">{full_addr}</td>
                                                                <td class="text-sm text-gray-600">{r.country.clone()}</td>
                                                                <td>
                                                                    {if verified {
                                                                        view! {
                                                                            <span class="flex items-center gap-1 text-green-600">
                                                                                <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsCheckCircleFill /></span>
                                                                                <span class="text-sm">"Verified"</span>
                                                                            </span>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <span class="text-sm text-gray-400">"Unverified"</span>
                                                                        }.into_any()
                                                                    }}
                                                                </td>
                                                                <td class="text-sm text-gray-600">{updated}</td>
                                                            </tr>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </>
                                            }.into_any()
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

