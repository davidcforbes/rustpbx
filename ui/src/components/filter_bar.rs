use leptos::prelude::*;
use leptos_icons::Icon;

use super::PhoneDrawer;

/// Top filter bar for activity pages.
/// Matches the legacy 4iiz top bar layout with search, filter, date range,
/// source selector, view toggles, and action buttons.
#[component]
pub fn FilterBar() -> impl IntoView {
    let show_phone = RwSignal::new(false);
    let auto_load = RwSignal::new(true);

    view! {
        <header class="h-14 bg-white border-b border-gray-200 flex items-center px-4 gap-3 flex-shrink-0">
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
                // Date range picker placeholder
                <div class="join">
                    <input
                        type="text"
                        placeholder="Date range..."
                        class="input input-sm input-bordered join-item w-56 text-xs"
                    />
                    <button class="btn btn-sm btn-ghost join-item border border-gray-300">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsCalendar /></span>
                    </button>
                </div>
                // Call count
                <span class="text-sm text-gray-500 flex items-center gap-1 whitespace-nowrap">
                    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsBarChartFill /></span>
                    "0 calls"
                </span>
                // Auto Load toggle
                <button
                    class="btn btn-sm btn-ghost gap-1 text-gray-500"
                    on:click=move |_| auto_load.update(|v| *v = !*v)
                >
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsArrowRepeat /></span>
                    {move || if auto_load.get() { "Stop Auto Load" } else { "Auto Load" }}
                </button>
            </div>

            // Spacer
            <div class="flex-1"></div>

            // Right controls
            <div class="flex items-center gap-2">
                // Saved filters dropdown
                <select class="select select-sm select-bordered w-44 text-xs text-gray-500">
                    <option selected disabled>"Choose saved filters"</option>
                </select>
                // Desk Mode toggle
                <label class="flex items-center gap-1 text-sm text-gray-500 cursor-pointer">
                    <input type="checkbox" class="checkbox checkbox-xs" />
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsDisplay /></span>
                    "Desk Mode"
                </label>
                // Notification bell
                <button class="btn btn-sm btn-ghost btn-circle relative">
                    <span class="w-5 h-5 inline-flex"><Icon icon=icondata::BsBell /></span>
                </button>
                // Column visibility dropdown
                <div class="dropdown dropdown-end">
                    <div tabindex="0" role="button" class="btn btn-sm btn-ghost gap-1 text-gray-500">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGrid3x3GapFill /></span>
                    </div>
                    <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-[1] w-52 p-2 shadow text-sm">
                        <li><label class="flex items-center gap-2"><input type="checkbox" class="checkbox checkbox-xs" checked />"Contact"</label></li>
                        <li><label class="flex items-center gap-2"><input type="checkbox" class="checkbox checkbox-xs" checked />"Source"</label></li>
                        <li><label class="flex items-center gap-2"><input type="checkbox" class="checkbox checkbox-xs" checked />"Session Data"</label></li>
                        <li><label class="flex items-center gap-2"><input type="checkbox" class="checkbox checkbox-xs" checked />"Score"</label></li>
                        <li><label class="flex items-center gap-2"><input type="checkbox" class="checkbox checkbox-xs" checked />"Audio"</label></li>
                        <li><label class="flex items-center gap-2"><input type="checkbox" class="checkbox checkbox-xs" checked />"Metrics"</label></li>
                        <li><label class="flex items-center gap-2"><input type="checkbox" class="checkbox checkbox-xs" checked />"Routing"</label></li>
                    </ul>
                </div>
                // Phone button — toggles the softphone drawer
                <button
                    class="btn btn-sm bg-emerald-500 hover:bg-emerald-600 text-white border-none gap-1"
                    on:click=move |_| show_phone.update(|v| *v = !*v)
                >
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsTelephoneFill /></span>
                    "Phone"
                </button>
                // Account (compact)
                <div class="flex items-center gap-1.5 pl-2 border-l border-gray-200 flex-shrink-0">
                    <span class="text-[10px] text-gray-500 text-right leading-tight hidden xl:block">
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
