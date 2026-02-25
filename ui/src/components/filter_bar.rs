use leptos::prelude::*;
use leptos_icons::Icon;

use super::PhoneDrawer;

/// Top filter bar for activity pages.
/// Matches the 4iiz prototype top bar layout with search, filter, date range,
/// source selector, view toggles, and action buttons.
#[component]
pub fn FilterBar() -> impl IntoView {
    let show_phone = RwSignal::new(false);

    view! {
        <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
            // Back button
            <button class="text-gray-400 hover:text-gray-600">
                <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsChevronLeft /></span>
            </button>

            // Filter + Search
            <div class="flex items-center gap-2">
                <button class="btn btn-sm btn-ghost gap-1 text-gray-600">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsFunnel /></span>
                    "Filter"
                </button>
                <div class="join">
                    <input
                        type="text"
                        placeholder="Search"
                        class="input input-sm input-bordered join-item w-48"
                    />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsSearch /></span>
                    </button>
                </div>
                // Active filters badge
                <span class="badge badge-sm bg-iiz-cyan text-white border-none">"22 active"</span>
                // Count
                <span class="text-sm text-gray-500 flex items-center gap-1">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsBarChartFill /></span>
                    "3,700,569 calls"
                </span>
                // Auto Load
                <label class="flex items-center gap-1 text-sm text-gray-500 cursor-pointer">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsArrowRepeat /></span>
                    "Auto Load"
                </label>
            </div>

            // Spacer
            <div class="flex-1"></div>

            // Right controls
            <div class="flex items-center gap-2">
                <label class="flex items-center gap-1 text-sm text-gray-500">
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsDisplay /></span>
                    "Desk Mode"
                </label>
                // Notification bell
                <button class="btn btn-sm btn-ghost btn-circle relative">
                    <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsBell /></span>
                    <span class="badge badge-xs bg-iiz-cyan text-white border-none absolute -top-1 -right-1">
                        "4950"
                    </span>
                </button>
                // View toggles
                <div class="join">
                    <button class="btn btn-sm btn-ghost join-item">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsPerson /></span>
                    </button>
                    <button class="btn btn-sm btn-ghost join-item">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGrid /></span>
                    </button>
                    <button class="btn btn-sm btn-ghost join-item">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGear /></span>
                    </button>
                </div>
                // Phone button — toggles the softphone drawer
                <button
                    class="btn btn-sm bg-emerald-500 hover:bg-emerald-600 text-white border-none gap-1"
                    on:click=move |_| show_phone.update(|v| *v = !*v)
                >
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsTelephoneFill /></span>
                    "Phone"
                </button>
                // Account
                <div class="flex items-center gap-2 pl-2 border-l border-gray-200">
                    <span class="text-xs text-gray-500 text-right leading-tight">
                        "Account ID 155169 - 4iiz"
                        <br />
                        "chris@forbesassetmanagement.com"
                    </span>
                    <div class="avatar placeholder">
                        <div class="bg-gray-600 text-white w-8 h-8 rounded-full flex items-center justify-center">
                            <span class="text-xs">"CF"</span>
                        </div>
                    </div>
                </div>
            </div>
        </header>

        // Phone drawer overlay
        <Show when=move || show_phone.get()>
            <PhoneDrawer on_close=move |_| show_phone.set(false) />
        </Show>
    }
}
