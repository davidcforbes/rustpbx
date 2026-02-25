use leptos::prelude::*;
use leptos_icons::Icon;

// ---------------------------------------------------------------------------
// Trust Center side navigation
// ---------------------------------------------------------------------------

#[component]
pub fn TrustCenterSideNav() -> impl IntoView {
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
                <a href="/trust-center/business" class="side-nav-item active">"Business/Contact Info"</a>
                <a href="/trust-center/local-text" class="side-nav-item">"Local Text Messaging"</a>
                <a href="/trust-center/toll-free-text" class="side-nav-item">"Toll Free Text Messaging"</a>
                <a href="/trust-center/voice-reg" class="side-nav-item">"Voice Registration"</a>
                <a href="/trust-center/caller-id" class="side-nav-item">"Caller ID"</a>
            </div>

            // Global Compliance group
            <div>
                <h3 class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsGlobe /></span>
                    "Global Compliance"
                </h3>
                <a href="/trust-center/requirements" class="side-nav-item">"Requirements"</a>
                <a href="/trust-center/applications" class="side-nav-item">"Applications"</a>
                <a href="/trust-center/addresses" class="side-nav-item">"Addresses"</a>
            </div>
        </nav>
    }
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Campaign {
    name: &'static str,
    created: &'static str,
    status: &'static str,
    assigned_numbers: u32,
    max_numbers: u32,
    cost: &'static str,
    carrier: &'static str,
}

// ---------------------------------------------------------------------------
// Mock data
// ---------------------------------------------------------------------------

fn mock_campaigns() -> Vec<Campaign> {
    vec![
        Campaign { name: "General Campaign", created: "2023-05-22", status: "Approved", assigned_numbers: 116, max_numbers: 400, cost: "$1.5/mo", carrier: "Carrier A" },
        Campaign { name: "New Campaign", created: "2023-11-03", status: "Approved", assigned_numbers: 59, max_numbers: 400, cost: "$1.5/mo", carrier: "Carrier A" },
    ]
}

// ---------------------------------------------------------------------------
// Business/Contact Information page (main Trust Center page)
// ---------------------------------------------------------------------------

#[component]
pub fn BusinessInfoPage() -> impl IntoView {
    let campaigns = mock_campaigns();

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
                                        {campaigns.into_iter().map(|c| {
                                            let nums = format!("({}/{})", c.assigned_numbers, c.max_numbers);
                                            view! {
                                                <tr class="border-b border-gray-100 hover:bg-gray-50">
                                                    <td>
                                                        <button class="btn btn-xs btn-ghost text-iiz-cyan">
                                                            <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsEye /></span>
                                                        </button>
                                                    </td>
                                                    <td class="font-medium text-sm">{c.name}</td>
                                                    <td class="text-sm text-gray-600">{c.created}</td>
                                                    <td>
                                                        <span class="flex items-center gap-1 text-sm text-green-600">
                                                            <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsCheckLg /></span>
                                                            {c.status}
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
                                                    <td class="text-sm text-gray-600">{c.cost}</td>
                                                    <td class="text-sm text-gray-600">{c.carrier}</td>
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
                                        <span class="font-medium ml-1">"2/50"</span>
                                    </div>
                                </div>
                                <button class="btn btn-sm bg-iiz-cyan hover:bg-iiz-cyan/80 text-white border-none">"Add Campaigns"</button>
                            </div>
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
// Placeholder for Trust Center pages
// ---------------------------------------------------------------------------

#[component]
pub fn TrustCenterPlaceholderPage(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
) -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <div class="flex-1 flex items-center justify-center bg-iiz-gray-bg">
                <div class="max-w-md text-center">
                    <div class="w-16 h-16 rounded-full bg-iiz-cyan-light flex items-center justify-center mx-auto mb-4">
                        <span class="w-8 h-8 inline-flex text-iiz-cyan"><Icon icon=icondata::BsShieldCheck /></span>
                    </div>
                    <h2 class="text-xl font-semibold text-gray-700">{title}</h2>
                    <p class="text-sm text-gray-500 mt-2">{description}</p>
                </div>
            </div>
        </div>
    }
}
