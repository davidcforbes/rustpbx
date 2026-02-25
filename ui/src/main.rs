mod components;
mod sections;

use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use sections::activities::{ActivitiesSideNav, CallsPage, PlaceholderPage};
use sections::flows::{
    BulkMessagesPage, FlowsPlaceholderPage, FlowsSideNav, FormReactorPage,
    QueuesPage, SchedulesPage, SmartRoutersPage, TriggersPage, VoiceMenusPage,
    WebhooksPage,
};
use sections::numbers::{
    BuyNumbersPage, CallSettingsPage, NumbersPlaceholderPage, NumbersSideNav,
    ReceivingNumbersPage, TrackingNumbersPage, TrackingSourcesPage,
};

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    mount_to_body(|| {
        view! { <App /> }
    });
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    let section = RwSignal::new(Some("activities".to_string()));

    view! {
        <Router>
            <div class="h-screen w-screen">
                <AppShell active_section=section>
                    <AppShellIconNav class="w-16 bg-white border-r border-iiz-gray-border">
                        // Logo
                        <div class="py-4 mb-2">
                            <span class="text-iiz-cyan font-bold text-lg">"4iiz"</span>
                        </div>

                        // Navigation items
                        <AppShellIconNavItem
                            value="activities"
                            class="[&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light hover:bg-gray-50 py-3"
                        >
                            <Icon icon=icondata::BsTelephoneFill />
                            <span class="text-[10px]">"Activities"</span>
                        </AppShellIconNavItem>

                        <AppShellIconNavItem
                            value="numbers"
                            class="[&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light hover:bg-gray-50 py-3"
                        >
                            <Icon icon=icondata::BsGrid3x3GapFill />
                            <span class="text-[10px]">"Numbers"</span>
                        </AppShellIconNavItem>

                        <AppShellIconNavItem
                            value="flows"
                            class="[&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light hover:bg-gray-50 py-3"
                        >
                            <Icon icon=icondata::BsArrowLeftRight />
                            <span class="text-[10px]">"Flows"</span>
                        </AppShellIconNavItem>

                        <AppShellIconNavItem
                            value="ai-tools"
                            class="[&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light hover:bg-gray-50 py-3"
                        >
                            <Icon icon=icondata::BsStars />
                            <span class="text-[10px]">"AI Tools"</span>
                        </AppShellIconNavItem>

                        <AppShellIconNavItem
                            value="reports"
                            class="[&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light hover:bg-gray-50 py-3"
                        >
                            <Icon icon=icondata::BsBarChartFill />
                            <span class="text-[10px]">"Reports"</span>
                        </AppShellIconNavItem>

                        <AppShellIconNavItem
                            value="trust-center"
                            class="[&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light hover:bg-gray-50 py-3"
                        >
                            <Icon icon=icondata::BsShieldCheck />
                            <span class="text-[10px]">"Trust Center"</span>
                        </AppShellIconNavItem>

                        // Spacer to push bottom icons down
                        <div class="flex-1"></div>

                        // Help icon
                        <div class="flex flex-col items-center py-3 text-gray-400 hover:text-gray-600 cursor-pointer">
                            <Icon icon=icondata::BsQuestionCircle />
                            <span class="text-[10px]">"Help"</span>
                        </div>

                        // Settings icon
                        <div class="flex flex-col items-center py-3 text-gray-400 hover:text-gray-600 cursor-pointer">
                            <Icon icon=icondata::BsGearFill />
                            <span class="text-[10px]">"Settings"</span>
                        </div>
                    </AppShellIconNav>

                    <AppShellSidePanel class="w-48 bg-white border-r border-iiz-gray-border">
                        <Show when=move || section.get() == Some("activities".to_string())>
                            <ActivitiesSideNav />
                        </Show>
                        <Show when=move || section.get() == Some("numbers".to_string())>
                            <NumbersSideNav />
                        </Show>
                        <Show when=move || section.get() == Some("flows".to_string())>
                            <FlowsSideNav />
                        </Show>
                    </AppShellSidePanel>

                    <AppShellContent class="bg-iiz-gray-bg">
                        <Routes fallback=|| "Page not found">
                            <Route path=path!("/") view=HomePage />
                            <Route path=path!("/activities/calls") view=CallsPage />
                            <Route path=path!("/activities/texts") view=PlaceholderPage />
                            <Route path=path!("/activities/forms") view=PlaceholderPage />
                            <Route path=path!("/activities/chats") view=PlaceholderPage />
                            <Route path=path!("/activities/faxes") view=PlaceholderPage />
                            <Route path=path!("/activities/videos") view=PlaceholderPage />
                            <Route path=path!("/activities/export") view=PlaceholderPage />
                            <Route path=path!("/contacts/lists") view=PlaceholderPage />
                            <Route path=path!("/contacts/blocked") view=PlaceholderPage />
                            <Route path=path!("/contacts/do-not-call") view=PlaceholderPage />
                            <Route path=path!("/contacts/do-not-text") view=PlaceholderPage />
                            <Route path=path!("/numbers/tracking") view=TrackingNumbersPage />
                            <Route path=path!("/numbers/buy") view=BuyNumbersPage />
                            <Route path=path!("/numbers/receiving") view=ReceivingNumbersPage />
                            <Route path=path!("/numbers/call-settings") view=CallSettingsPage />
                            <Route path=path!("/numbers/sources") view=TrackingSourcesPage />
                            <Route path=path!("/numbers/text") view=|| view! { <NumbersPlaceholderPage title="Text Numbers" /> } />
                            <Route path=path!("/numbers/port") view=|| view! { <NumbersPlaceholderPage title="Port Numbers" /> } />
                            <Route path=path!("/numbers/pools") view=|| view! { <NumbersPlaceholderPage title="Number Pools" /> } />
                            <Route path=path!("/numbers/targets") view=|| view! { <NumbersPlaceholderPage title="Target Numbers" /> } />
                            <Route path=path!("/numbers/code") view=|| view! { <NumbersPlaceholderPage title="Tracking Code" /> } />
                            // Flows - Routing
                            <Route path=path!("/flows/voice-menus") view=VoiceMenusPage />
                            <Route path=path!("/flows/queues") view=QueuesPage />
                            <Route path=path!("/flows/smart-routers") view=SmartRoutersPage />
                            <Route path=path!("/flows/schedules") view=SchedulesPage />
                            <Route path=path!("/flows/geo-routers") view=|| view! { <FlowsPlaceholderPage title="Geo Routers" description="Route callers based on their geographic location using area codes, states, or countries." /> } />
                            <Route path=path!("/flows/agent-scripts") view=|| view! { <FlowsPlaceholderPage title="Agent Scripts" description="Create guided scripts for agents to follow during calls." /> } />
                            <Route path=path!("/flows/routing-tables") view=|| view! { <FlowsPlaceholderPage title="Routing Tables" description="Define advanced routing rules using lookup tables for number-based call distribution." /> } />
                            <Route path=path!("/flows/voicemails") view=|| view! { <FlowsPlaceholderPage title="Voicemails" description="Configure voicemail boxes, greetings, and notification settings." /> } />
                            // Flows - Automation
                            <Route path=path!("/flows/workflows") view=|| view! { <FlowsPlaceholderPage title="Workflows" description="Build visual multi-step automation workflows with drag-and-drop logic." /> } />
                            <Route path=path!("/flows/triggers") view=TriggersPage />
                            <Route path=path!("/flows/keyword-spotting") view=|| view! { <FlowsPlaceholderPage title="Keyword Spotting" description="Automatically detect and tag calls based on keywords spoken during conversations." /> } />
                            <Route path=path!("/flows/lambdas") view=|| view! { <FlowsPlaceholderPage title="Lambdas" description="Write custom JavaScript functions that execute during call flows." /> } />
                            <Route path=path!("/flows/api-logs") view=|| view! { <FlowsPlaceholderPage title="API Logs" description="View detailed logs of all API requests, webhook deliveries, and integration activity." /> } />
                            <Route path=path!("/flows/global") view=|| view! { <FlowsPlaceholderPage title="Global" description="Configure account-wide automation settings and global variables." /> } />
                            <Route path=path!("/flows/webhooks") view=WebhooksPage />
                            // Flows - Engagement
                            <Route path=path!("/flows/bulk-messages") view=BulkMessagesPage />
                            <Route path=path!("/flows/lead-reactor") view=|| view! { <FlowsPlaceholderPage title="LeadReactor" description="Automatically respond to new leads with calls, texts, or emails." /> } />
                            <Route path=path!("/flows/smart-dialers") view=|| view! { <FlowsPlaceholderPage title="Smart Dialers" description="Set up automated outbound dialing campaigns with intelligent pacing." /> } />
                            <Route path=path!("/flows/form-reactor") view=FormReactorPage />
                            <Route path=path!("/flows/chat-widget") view=|| view! { <FlowsPlaceholderPage title="Chat Widget" description="Configure embeddable chat widgets for your website." /> } />
                            <Route path=path!("/flows/chat-ai") view=|| view! { <FlowsPlaceholderPage title="ChatAI" description="Configure AI-powered chat responses using natural language processing." /> } />
                            <Route path=path!("/flows/dialogflow") view=|| view! { <FlowsPlaceholderPage title="Dialogflow" description="Integrate Google Dialogflow for conversational AI and intent-based routing." /> } />
                            <Route path=path!("/flows/reminders") view=|| view! { <FlowsPlaceholderPage title="Reminders" description="Schedule automated reminder calls, texts, or emails to contacts." /> } />
                        </Routes>
                    </AppShellContent>
                </AppShell>
            </div>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold text-iiz-dark">"Welcome to 4iiz"</h1>
            <p class="text-iiz-gray-text mt-2">"Select a section from the navigation to get started."</p>
        </div>
    }
}
