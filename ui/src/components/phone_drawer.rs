use leptos::prelude::*;
use leptos_icons::Icon;

/// Active tab within the phone drawer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhoneTab {
    Phone,
    Actions,
}

const KEYPAD: [(&str, &str); 12] = [
    ("1", ""),    ("2", "ABC"),  ("3", "DEF"),
    ("4", "GHI"), ("5", "JKL"),  ("6", "MNO"),
    ("7", "PQRS"),("8", "TUV"),  ("9", "WXYZ"),
    ("*", ""),    ("0", "+"),    ("#", ""),
];

/// 4iiz Softphone drawer — slides out from the right when the green Phone
/// button is pressed.  Mirrors the real 4iiz UI: status bar, dialer,
/// stats, quick-action grid, and a left tab strip (Phone / Actions).
#[component]
pub fn PhoneDrawer(
    on_close: impl Fn(leptos::ev::MouseEvent) + 'static + Clone,
) -> impl IntoView {
    let active_tab = RwSignal::new(PhoneTab::Phone);
    let dial_value = RwSignal::new("(516) 398-771".to_string());

    let on_close_inner = on_close.clone();

    view! {
        // Backdrop
        <div
            class="fixed inset-0 bg-black/20 z-40"
            on:click=move |e| on_close_inner(e)
        ></div>

        // Drawer panel — fixed right, slides in
        <div class="fixed top-0 right-0 h-full w-[360px] bg-white shadow-2xl z-50 flex animate-slide-in-right">
            // Left tab strip
            <div class="w-[72px] bg-gray-50 border-r border-gray-200 flex flex-col items-center pt-3 gap-1 flex-shrink-0">
                <button
                    class=move || {
                        if active_tab.get() == PhoneTab::Phone {
                            "flex flex-col items-center gap-0.5 px-2 py-2 rounded-lg bg-red-500 text-white text-[10px] w-[60px]"
                        } else {
                            "flex flex-col items-center gap-0.5 px-2 py-2 rounded-lg hover:bg-gray-100 text-gray-500 text-[10px] w-[60px]"
                        }
                    }
                    on:click=move |_| active_tab.set(PhoneTab::Phone)
                >
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsTelephoneFill /></span>
                    <span>"Phone"</span>
                    <span class="text-[9px]">">"</span>
                </button>
                <button
                    class=move || {
                        if active_tab.get() == PhoneTab::Actions {
                            "flex flex-col items-center gap-0.5 px-2 py-2 rounded-lg bg-gray-200 text-gray-800 text-[10px] w-[60px]"
                        } else {
                            "flex flex-col items-center gap-0.5 px-2 py-2 rounded-lg hover:bg-gray-100 text-gray-500 text-[10px] w-[60px]"
                        }
                    }
                    on:click=move |_| active_tab.set(PhoneTab::Actions)
                >
                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsListUl /></span>
                    <span>"Actions"</span>
                </button>

                // Spacer
                <div class="flex-1"></div>

                // Bottom action buttons
                <div class="flex flex-col items-center gap-2 pb-4">
                    <button class="w-9 h-9 rounded-lg bg-gray-100 hover:bg-gray-200 flex items-center justify-center text-gray-500">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsEnvelope /></span>
                    </button>
                    <span class="text-[9px] text-gray-400 -mt-1">"Email"</span>
                    <button class="w-9 h-9 rounded-lg bg-gray-100 hover:bg-gray-200 flex items-center justify-center text-red-400">
                        <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsFlagFill /></span>
                    </button>
                    <span class="text-[9px] text-gray-400 -mt-1">"Flag"</span>
                </div>
            </div>

            // Main content area
            <div class="flex-1 flex flex-col min-w-0">
                // Red status header
                <div class="bg-red-500 text-white px-4 py-2.5 flex items-center gap-3 flex-shrink-0">
                    <div class="flex items-center gap-1.5">
                        <span class="w-2 h-2 rounded-full border border-white inline-block"></span>
                        <span class="font-semibold text-sm">"Not Ready"</span>
                    </div>
                    <div class="flex-1"></div>
                    <div class="flex items-center gap-2">
                        <span class="text-sm font-mono">"04:46"</span>
                        <button class="text-white/80 hover:text-white">
                            <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsChevronDown /></span>
                        </button>
                        <button class="text-white/80 hover:text-white">
                            <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsGearFill /></span>
                        </button>
                    </div>
                </div>

                // Phone tab content
                <Show when=move || active_tab.get() == PhoneTab::Phone>
                    <div class="flex-1 flex flex-col">
                        // Dialer input area
                        <div class="px-4 py-4 border-b border-gray-100">
                            <div class="flex items-center gap-2">
                                // Country flag + code
                                <div class="flex items-center gap-1 text-sm text-gray-600 flex-shrink-0">
                                    <span class="text-base">{"\u{1F1FA}\u{1F1F8}"}</span>
                                    <span class="text-xs text-gray-500">"US +1"</span>
                                </div>
                                // Number input
                                <input
                                    type="text"
                                    class="flex-1 text-xl font-light text-gray-800 border-none outline-none bg-transparent"
                                    prop:value=move || dial_value.get()
                                    on:input=move |ev| {
                                        dial_value.set(event_target_value(&ev));
                                    }
                                    placeholder="Enter number..."
                                />
                                // Chat button
                                <button class="w-9 h-9 rounded-lg bg-red-500 hover:bg-red-600 flex items-center justify-center text-white flex-shrink-0">
                                    <span class="w-4 h-4 inline-flex"><Icon icon=icondata::BsChatDotsFill /></span>
                                </button>
                            </div>
                        </div>

                        // Stats row
                        <div class="grid grid-cols-3 text-center py-4 border-b border-gray-100">
                            <div>
                                <div class="text-2xl font-light text-gray-800">"0"</div>
                                <div class="text-[11px] text-gray-400">"Inbound"</div>
                            </div>
                            <div>
                                <div class="text-2xl font-light text-gray-800">"0"</div>
                                <div class="text-[11px] text-gray-400">"Outbound"</div>
                            </div>
                            <div>
                                <div class="text-lg font-light text-gray-800 font-mono">"00:00:00"</div>
                                <div class="text-[11px] text-gray-400">"Talk Time"</div>
                            </div>
                        </div>

                        // Quick action grid
                        <div class="grid grid-cols-3 text-center py-4 border-b border-gray-100 gap-y-2">
                            <button class="flex flex-col items-center gap-1 text-gray-500 hover:text-iiz-cyan">
                                <span class="w-6 h-6 inline-flex"><Icon icon=icondata::BsTelephoneInboundFill /></span>
                                <span class="text-[11px]">"Parked"</span>
                            </button>
                            <button class="flex flex-col items-center gap-1 text-gray-500 hover:text-iiz-cyan">
                                <span class="w-6 h-6 inline-flex"><Icon icon=icondata::BsGrid3x3GapFill /></span>
                                <span class="text-[11px]">"Keypad"</span>
                            </button>
                            <button class="flex flex-col items-center gap-1 text-gray-500 hover:text-iiz-cyan">
                                <span class="w-6 h-6 inline-flex"><Icon icon=icondata::BsPeopleFill /></span>
                                <span class="text-[11px]">"Team"</span>
                            </button>
                        </div>

                        // Keypad (hidden by default, shown when Keypad tapped)
                        <div class="flex-1 px-6 py-4">
                            <div class="grid grid-cols-3 gap-3">
                                {KEYPAD.iter().map(|(digit, letters)| {
                                    let d = digit.to_string();
                                    let l = letters.to_string();
                                    let d_click = d.clone();
                                    view! {
                                        <button
                                            class="w-full aspect-square rounded-full bg-gray-50 hover:bg-gray-100 flex flex-col items-center justify-center border border-gray-200"
                                            on:click=move |_| {
                                                dial_value.update(|v| v.push_str(&d_click));
                                            }
                                        >
                                            <span class="text-xl font-light text-gray-800">{d}</span>
                                            <span class="text-[8px] text-gray-400 tracking-widest">{l}</span>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>

                            // Call button
                            <div class="flex justify-center mt-4">
                                <button class="w-14 h-14 rounded-full bg-emerald-500 hover:bg-emerald-600 flex items-center justify-center text-white shadow-lg">
                                    <span class="w-6 h-6 inline-flex"><Icon icon=icondata::BsTelephoneFill /></span>
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>

                // Actions tab content
                <Show when=move || active_tab.get() == PhoneTab::Actions>
                    <div class="flex-1 p-4">
                        <h3 class="text-sm font-semibold text-gray-700 mb-3">"Quick Actions"</h3>
                        <div class="space-y-2">
                            <button class="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 text-sm text-gray-600 flex items-center gap-2">
                                <span class="w-4 h-4 inline-flex text-iiz-cyan"><Icon icon=icondata::BsTelephoneForwardFill /></span>
                                "Transfer Call"
                            </button>
                            <button class="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 text-sm text-gray-600 flex items-center gap-2">
                                <span class="w-4 h-4 inline-flex text-iiz-cyan"><Icon icon=icondata::BsPauseFill /></span>
                                "Hold"
                            </button>
                            <button class="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 text-sm text-gray-600 flex items-center gap-2">
                                <span class="w-4 h-4 inline-flex text-iiz-cyan"><Icon icon=icondata::BsMicMuteFill /></span>
                                "Mute"
                            </button>
                            <button class="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 text-sm text-gray-600 flex items-center gap-2">
                                <span class="w-4 h-4 inline-flex text-iiz-cyan"><Icon icon=icondata::BsRecordCircle /></span>
                                "Record"
                            </button>
                            <button class="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 text-sm text-gray-600 flex items-center gap-2">
                                <span class="w-4 h-4 inline-flex text-red-500"><Icon icon=icondata::BsTelephoneXFill /></span>
                                "End Call"
                            </button>
                        </div>

                        <h3 class="text-sm font-semibold text-gray-700 mt-6 mb-3">"After Call Work"</h3>
                        <div class="space-y-2">
                            <button class="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 text-sm text-gray-600 flex items-center gap-2">
                                <span class="w-4 h-4 inline-flex text-gray-400"><Icon icon=icondata::BsTagFill /></span>
                                "Add Tags"
                            </button>
                            <button class="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 text-sm text-gray-600 flex items-center gap-2">
                                <span class="w-4 h-4 inline-flex text-gray-400"><Icon icon=icondata::BsPencilSquare /></span>
                                "Add Notes"
                            </button>
                            <button class="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 text-sm text-gray-600 flex items-center gap-2">
                                <span class="w-4 h-4 inline-flex text-gray-400"><Icon icon=icondata::BsCalendarPlus /></span>
                                "Schedule Follow-up"
                            </button>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
