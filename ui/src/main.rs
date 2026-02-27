mod api;
mod components;
mod sections;

use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::{
    components::{Redirect, Route, Router, Routes},
    hooks::use_location,
    path,
};
use sections::activities::{ActivitiesSideNav, CallsPage, ChatsPage, ExportLogPage, FaxesPage, FormsPage, TextsPage, VideosPage};
use sections::ai_tools::{
    AIToolsSideNav, AskAIPage, ChatAIPage, KnowledgeBanksPage,
    SummariesPage, VoiceAIPage,
};
use sections::contacts::{BlockedNumbersPage, ContactListsPage, DoNotCallPage, DoNotTextPage};
use sections::flows::{
    AgentScriptsPage, ApiLogsPage, BulkMessagesPage, ChatWidgetPage, DialogflowPage,
    FlowsChatAIPage, FlowsSideNav, FormReactorPage, GeoRoutersPage,
    GlobalPage, KeywordSpottingPage, LambdasPage, LeadReactorPage, QueuesPage, RemindersPage,
    RoutingTablesPage, SchedulesPage, SmartDialersPage, SmartRoutersPage, TriggersPage,
    VoicemailsPage, VoiceMenusPage, WebhooksPage, WorkflowsPage,
};
use sections::numbers::{
    BuyNumbersPage, CallSettingsPage, NumberPoolsPage, NumbersSideNav,
    PortNumbersPage, ReceivingNumbersPage, TargetNumbersPage, TextNumbersPage,
    TrackingCodePage, TrackingNumbersPage, TrackingSourcesPage,
};
use sections::reports::{
    AccuracyReportPage, ActivityMapPage, ActivityReportPage, AgencyUsagePage,
    AgentActivityPage, AppointmentsPage, CoachingPage, CSDailyMissed2Page,
    CSDailyMissedPage, CustomReportsPage, DailyCallsPage, DailyCollectionPage,
    GoogleCAPage, MissedCallsPage, MissedDaily1stPage, NotificationsPage,
    OverviewPage, PositiveDailyPage, PowerBIPage, PrimingCallsPage,
    PrimingMissedPage, QueueReportPage, ROIReportPage, RealTimeAgentsPage,
    RealTimePage, ReportsSideNav, SaturdayCallsPage, ScoringPage, TagsPage,
    TodaysMissedPage, WeeklyMissedPage,
};
use sections::trust_center::{
    AddressesPage, ApplicationsPage, BusinessInfoPage, CallerIdPage,
    LocalTextPage, RequirementsPage, TollFreeTextPage, TrustCenterSideNav,
    VoiceRegPage,
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

    view! {
        <Router>
            <AppLayout />
        </Router>
    }
}

#[component]
fn AppLayout() -> impl IntoView {
    let location = use_location();
    let section = RwSignal::new(Some("activities".to_string()));

    // Sync section signal from current URL path
    Effect::new(move |_| {
        let path = location.pathname.get();
        let new_section = if path.starts_with("/numbers") {
            "numbers"
        } else if path.starts_with("/flows") {
            "flows"
        } else if path.starts_with("/ai-tools") {
            "ai-tools"
        } else if path.starts_with("/reports") {
            "reports"
        } else if path.starts_with("/trust-center") {
            "trust-center"
        } else {
            "activities"
        };
        section.set(Some(new_section.to_string()));
    });

    view! {
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
                        <Show when=move || section.get() == Some("ai-tools".to_string())>
                            <AIToolsSideNav />
                        </Show>
                        <Show when=move || section.get() == Some("reports".to_string())>
                            <ReportsSideNav />
                        </Show>
                        <Show when=move || section.get() == Some("trust-center".to_string())>
                            <TrustCenterSideNav />
                        </Show>
                    </AppShellSidePanel>

                    <AppShellContent class="bg-iiz-gray-bg">
                        <Routes fallback=|| "Page not found">
                            <Route path=path!("/") view=|| view! { <Redirect path="/activities/calls" /> } />
                            // Activities
                            <Route path=path!("/activities/calls") view=CallsPage />
                            <Route path=path!("/activities/texts") view=TextsPage />
                            <Route path=path!("/activities/forms") view=FormsPage />
                            <Route path=path!("/activities/chats") view=ChatsPage />
                            <Route path=path!("/activities/faxes") view=FaxesPage />
                            <Route path=path!("/activities/videos") view=VideosPage />
                            <Route path=path!("/activities/export") view=ExportLogPage />
                            // Contacts
                            <Route path=path!("/contacts/lists") view=ContactListsPage />
                            <Route path=path!("/contacts/blocked") view=BlockedNumbersPage />
                            <Route path=path!("/contacts/do-not-call") view=DoNotCallPage />
                            <Route path=path!("/contacts/do-not-text") view=DoNotTextPage />
                            // Numbers
                            <Route path=path!("/numbers/tracking") view=TrackingNumbersPage />
                            <Route path=path!("/numbers/buy") view=BuyNumbersPage />
                            <Route path=path!("/numbers/receiving") view=ReceivingNumbersPage />
                            <Route path=path!("/numbers/call-settings") view=CallSettingsPage />
                            <Route path=path!("/numbers/sources") view=TrackingSourcesPage />
                            <Route path=path!("/numbers/text") view=TextNumbersPage />
                            <Route path=path!("/numbers/port") view=PortNumbersPage />
                            <Route path=path!("/numbers/pools") view=NumberPoolsPage />
                            <Route path=path!("/numbers/targets") view=TargetNumbersPage />
                            <Route path=path!("/numbers/code") view=TrackingCodePage />
                            // Flows - Routing
                            <Route path=path!("/flows/voice-menus") view=VoiceMenusPage />
                            <Route path=path!("/flows/queues") view=QueuesPage />
                            <Route path=path!("/flows/smart-routers") view=SmartRoutersPage />
                            <Route path=path!("/flows/schedules") view=SchedulesPage />
                            <Route path=path!("/flows/geo-routers") view=GeoRoutersPage />
                            <Route path=path!("/flows/agent-scripts") view=AgentScriptsPage />
                            <Route path=path!("/flows/routing-tables") view=RoutingTablesPage />
                            <Route path=path!("/flows/voicemails") view=VoicemailsPage />
                            // Flows - Automation
                            <Route path=path!("/flows/workflows") view=WorkflowsPage />
                            <Route path=path!("/flows/triggers") view=TriggersPage />
                            <Route path=path!("/flows/keyword-spotting") view=KeywordSpottingPage />
                            <Route path=path!("/flows/lambdas") view=LambdasPage />
                            <Route path=path!("/flows/api-logs") view=ApiLogsPage />
                            <Route path=path!("/flows/global") view=GlobalPage />
                            <Route path=path!("/flows/webhooks") view=WebhooksPage />
                            // Flows - Engagement
                            <Route path=path!("/flows/bulk-messages") view=BulkMessagesPage />
                            <Route path=path!("/flows/lead-reactor") view=LeadReactorPage />
                            <Route path=path!("/flows/smart-dialers") view=SmartDialersPage />
                            <Route path=path!("/flows/form-reactor") view=FormReactorPage />
                            <Route path=path!("/flows/chat-widget") view=ChatWidgetPage />
                            <Route path=path!("/flows/chat-ai") view=FlowsChatAIPage />
                            <Route path=path!("/flows/dialogflow") view=DialogflowPage />
                            <Route path=path!("/flows/reminders") view=RemindersPage />
                            // AI Tools
                            <Route path=path!("/ai-tools/askai") view=AskAIPage />
                            <Route path=path!("/ai-tools/summaries") view=SummariesPage />
                            <Route path=path!("/ai-tools/knowledge-banks") view=KnowledgeBanksPage />
                            <Route path=path!("/ai-tools/voiceai") view=VoiceAIPage />
                            <Route path=path!("/ai-tools/chatai") view=ChatAIPage />
                            // Reports - Analytics
                            <Route path=path!("/reports/activity") view=ActivityReportPage />
                            <Route path=path!("/reports/roi") view=ROIReportPage />
                            <Route path=path!("/reports/accuracy") view=AccuracyReportPage />
                            <Route path=path!("/reports/map") view=ActivityMapPage />
                            <Route path=path!("/reports/overview") view=OverviewPage />
                            <Route path=path!("/reports/todays-missed") view=TodaysMissedPage />
                            <Route path=path!("/reports/positive-daily") view=PositiveDailyPage />
                            <Route path=path!("/reports/google-ca") view=GoogleCAPage />
                            <Route path=path!("/reports/saturday-calls") view=SaturdayCallsPage />
                            <Route path=path!("/reports/daily-calls") view=DailyCallsPage />
                            <Route path=path!("/reports/weekly-missed") view=WeeklyMissedPage />
                            <Route path=path!("/reports/priming") view=PrimingCallsPage />
                            <Route path=path!("/reports/missed") view=MissedCallsPage />
                            <Route path=path!("/reports/missed-daily-1st") view=MissedDaily1stPage />
                            <Route path=path!("/reports/cs-daily-missed") view=CSDailyMissedPage />
                            <Route path=path!("/reports/cs-daily-missed-2") view=CSDailyMissed2Page />
                            <Route path=path!("/reports/priming-missed") view=PrimingMissedPage />
                            <Route path=path!("/reports/daily-collection") view=DailyCollectionPage />
                            <Route path=path!("/reports/power-bi") view=PowerBIPage />
                            <Route path=path!("/reports/realtime") view=RealTimePage />
                            <Route path=path!("/reports/appointments") view=AppointmentsPage />
                            // Reports - Connect
                            <Route path=path!("/reports/realtime-agents") view=RealTimeAgentsPage />
                            <Route path=path!("/reports/coaching") view=CoachingPage />
                            <Route path=path!("/reports/queue-report") view=QueueReportPage />
                            <Route path=path!("/reports/agent-activity") view=AgentActivityPage />
                            // Reports - Usage
                            <Route path=path!("/reports/agency-usage") view=AgencyUsagePage />
                            // Reports - Settings
                            <Route path=path!("/reports/custom-reports") view=CustomReportsPage />
                            <Route path=path!("/reports/notifications") view=NotificationsPage />
                            <Route path=path!("/reports/scoring") view=ScoringPage />
                            <Route path=path!("/reports/tags") view=TagsPage />
                            // Trust Center - US Outbound Compliance
                            <Route path=path!("/trust-center/business") view=BusinessInfoPage />
                            <Route path=path!("/trust-center/local-text") view=LocalTextPage />
                            <Route path=path!("/trust-center/toll-free-text") view=TollFreeTextPage />
                            <Route path=path!("/trust-center/voice-reg") view=VoiceRegPage />
                            <Route path=path!("/trust-center/caller-id") view=CallerIdPage />
                            // Trust Center - Global Compliance
                            <Route path=path!("/trust-center/requirements") view=RequirementsPage />
                            <Route path=path!("/trust-center/applications") view=ApplicationsPage />
                            <Route path=path!("/trust-center/addresses") view=AddressesPage />
                        </Routes>
                    </AppShellContent>
                </AppShell>
            </div>
    }
}
