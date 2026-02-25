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
