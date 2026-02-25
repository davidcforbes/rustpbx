# 4iiz Leptos WASM UI — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the 4iiz product UI as a Leptos WASM SPA that consumes RustPBX's REST API, starting with the Activities section.

**Architecture:** Separate `ui/` crate in the RustPBX workspace. Trunk compiles to static WASM files. Axum serves them on `/app/*`. The SPA uses `gloo-net` for API calls and `leptos-daisyui-rs` (with AppShell) for components. Phase 1 uses hardcoded mock data for pixel-perfect UI fidelity before wiring real APIs.

**Tech Stack:** Leptos 0.8 (CSR), leptos-router 0.8, leptos-daisyui-rs (local path), Trunk, Tailwind CSS v4 + DaisyUI, gloo-net, leptos_icons/icondata

---

## Task 1: Scaffold the `ui/` Crate

**Files:**
- Create: `ui/Cargo.toml`
- Create: `ui/Trunk.toml`
- Create: `ui/index.html`
- Create: `ui/input.css`
- Create: `ui/src/main.rs`

**Step 1: Create `ui/Cargo.toml`**

```toml
[package]
name = "rustpbx-ui"
version = "0.1.0"
edition = "2024"

[dependencies]
leptos-daisyui-rs = { path = "../../leptos-daisyui-rs" }

leptos = { version = "0.8", features = ["csr"] }
leptos_router = "0.8"
leptos_icons = "0.6.1"
leptos_meta = "0.8.4"
icondata = "0.6.0"

wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["HtmlInputElement"] }

console_log = "1.0.0"
console_error_panic_hook = "0.1.7"
log = "0.4"
chrono = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
gloo-net = "0.6"
```

**Step 2: Create `ui/Trunk.toml`**

```toml
[build]
target = "index.html"
release = false
dist = "dist"
public_url = "/app/"
filehash = true
inject_scripts = true

[watch]
watch = ["../src/models", "./src"]

[serve]
addresses = ["127.0.0.1"]
port = 3000
open = false

[[hooks]]
stage = "pre_build"
command = "npx.cmd"
command_arguments = ["tailwindcss", "-i", "input.css", "-o", "output.css"]
```

**Step 3: Create `ui/index.html`**

```html
<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>4iiz</title>
    <link data-trunk rel="css" href="output.css" />
    <link data-trunk rel="rust" data-wasm-opt="z" data-weak-refs />
</head>
<body></body>
</html>
```

**Step 4: Create `ui/input.css`**

Copy the `@source inline(...)` declarations from `leptos-daisyui-rs/demo/input.css` and add:

```css
@charset "UTF-8";
@import "tailwindcss";
@plugin "daisyui" {
  themes: light --default;
}
@source "./src/**/*.rs";
@source "../../leptos-daisyui-rs/src/**/*.rs";

/* Copy all @source inline(...) blocks from leptos-daisyui-rs/demo/input.css */
/* ... (accordion, alert, avatar, badge, breadcrumbs, button, card, etc.) ... */

/* 4iiz custom colors */
@theme {
  --color-iiz-cyan: #00bcd4;
  --color-iiz-cyan-light: #e0f7fa;
  --color-iiz-blue: #2196f3;
  --color-iiz-blue-link: #1e88e5;
  --color-iiz-green: #4caf50;
  --color-iiz-red: #f44336;
  --color-iiz-orange: #ff9800;
  --color-iiz-gray-bg: #f8f9fa;
  --color-iiz-gray-border: #e0e0e0;
  --color-iiz-gray-text: #757575;
  --color-iiz-dark: #37474f;
}

/* 4iiz custom utility classes */
@layer components {
  .activity-row:hover { background-color: #f0fafa; }
  .activity-row { border-bottom: 1px solid #f0f0f0; }
  .nav-icon-btn {
    @apply flex flex-col items-center gap-0.5 px-2 py-3 text-xs rounded-lg transition-colors;
  }
  .side-nav-item {
    @apply block px-4 py-1.5 text-sm rounded-md transition-colors cursor-pointer;
  }
  .col-header {
    @apply text-xs font-medium text-gray-500 uppercase tracking-wide cursor-pointer select-none;
  }
  .col-header:hover { @apply text-gray-700; }
  .tag-badge {
    @apply inline-block px-2 py-0.5 text-xs rounded bg-gray-100 text-gray-600;
  }
  .detail-tab {
    @apply flex items-center gap-2 px-3 py-2 text-sm rounded-md cursor-pointer transition-colors;
  }
}
```

**Step 5: Create `ui/src/main.rs` (minimal shell)**

```rust
mod sections;

use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path,
};

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);
    mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    let section = RwSignal::new(Some("activities".to_string()));

    view! {
        <Router>
            <AppShell active_section=section>
                <AppShellIconNav class="w-16 bg-white border-r border-gray-200 flex-shrink-0 py-2">
                    // Logo
                    <div class="mb-4 py-2 flex justify-center">
                        <span class="text-iiz-cyan font-bold text-lg">"4iiz"</span>
                    </div>
                    // Nav items
                    <AppShellIconNavItem value="activities" class="nav-icon-btn [&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light [&:not(.active)]:text-gray-500 [&:not(.active)]:hover:text-iiz-cyan">
                        <Icon icon=icondata::BsTelephoneFill />
                        <span class="text-[10px]">"Activities"</span>
                    </AppShellIconNavItem>
                    <AppShellIconNavItem value="numbers" class="nav-icon-btn [&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light [&:not(.active)]:text-gray-500 [&:not(.active)]:hover:text-iiz-cyan">
                        <Icon icon=icondata::BsGrid3x3GapFill />
                        <span class="text-[10px]">"Numbers"</span>
                    </AppShellIconNavItem>
                    <AppShellIconNavItem value="flows" class="nav-icon-btn [&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light [&:not(.active)]:text-gray-500 [&:not(.active)]:hover:text-iiz-cyan">
                        <Icon icon=icondata::BsArrowLeftRight />
                        <span class="text-[10px]">"Flows"</span>
                    </AppShellIconNavItem>
                    <AppShellIconNavItem value="ai-tools" class="nav-icon-btn [&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light [&:not(.active)]:text-gray-500 [&:not(.active)]:hover:text-iiz-cyan">
                        <Icon icon=icondata::BsStars />
                        <span class="text-[10px]">"AI Tools"</span>
                    </AppShellIconNavItem>
                    <AppShellIconNavItem value="reports" class="nav-icon-btn [&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light [&:not(.active)]:text-gray-500 [&:not(.active)]:hover:text-iiz-cyan">
                        <Icon icon=icondata::BsBarChartFill />
                        <span class="text-[10px]">"Reports"</span>
                    </AppShellIconNavItem>
                    <AppShellIconNavItem value="trust-center" class="nav-icon-btn [&.active]:text-iiz-cyan [&.active]:bg-iiz-cyan-light [&:not(.active)]:text-gray-500 [&:not(.active)]:hover:text-iiz-cyan">
                        <Icon icon=icondata::BsShieldCheck />
                        <span class="text-[10px]">"Trust"</span>
                    </AppShellIconNavItem>
                </AppShellIconNav>

                <AppShellSidePanel class="w-48 bg-white border-r border-gray-200 py-4">
                    <Show when=move || section.get() == Some("activities".to_string())>
                        <sections::activities::ActivitiesSideNav />
                    </Show>
                    // Other sections will be added as we build them
                </AppShellSidePanel>

                <AppShellContent class="bg-iiz-gray-bg overflow-y-auto">
                    <Routes fallback=|| "Page not found">
                        <Route path=path!("/") view=sections::activities::CallsPage />
                        <Route path=path!("/activities/calls") view=sections::activities::CallsPage />
                    </Routes>
                </AppShellContent>
            </AppShell>
        </Router>
    }
}
```

**Step 6: Create `ui/src/sections/mod.rs`**

```rust
pub mod activities;
```

**Step 7: Verify it compiles**

```bash
cd ui && cargo check
```

Expected: compiles with no errors (activities module will be a stub at this point).

**Step 8: Commit**

```bash
git add ui/
git commit -m "feat(ui): scaffold 4iiz Leptos WASM SPA with AppShell layout"
```

---

## Task 2: Build Activities Side Navigation

**Files:**
- Create: `ui/src/sections/activities.rs`

**Step 1: Create the Activities module with side nav and stub pages**

The side nav has two groups: "Activity Logs" (Calls, Texts, Forms, Chats, Faxes, Videos, Export Log) and "Contacts" (Lists, Blocked Numbers, Do Not Call, Do Not Text).

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon;

/// Side navigation for the Activities section
#[component]
pub fn ActivitiesSideNav() -> impl IntoView {
    view! {
        <div class="px-2">
            // Activity Logs group
            <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider px-4 mb-2">"Activity Logs"</h3>
            <nav class="space-y-0.5 mb-4">
                <a href="/activities/calls" class="side-nav-item active">"Calls"</a>
                <a href="/activities/texts" class="side-nav-item">"Texts"</a>
                <a href="/activities/forms" class="side-nav-item">"Forms"</a>
                <a href="/activities/chats" class="side-nav-item">"Chats"</a>
                <a href="/activities/faxes" class="side-nav-item">"Faxes"</a>
                <a href="/activities/videos" class="side-nav-item">"Videos"</a>
                <a href="/activities/export" class="side-nav-item">"Export Log"</a>
            </nav>
            // Contacts group
            <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider px-4 mb-2">"Contacts"</h3>
            <nav class="space-y-0.5">
                <a href="/contacts" class="side-nav-item">"Lists"</a>
                <a href="/contacts/blocked" class="side-nav-item">"Blocked Numbers"</a>
                <a href="/contacts/dnc" class="side-nav-item">"Do Not Call List"</a>
                <a href="/contacts/dnt" class="side-nav-item">"Do Not Text List"</a>
            </nav>
        </div>
    }
}

/// Calls page — main activity log view with call list and detail panel
#[component]
pub fn CallsPage() -> impl IntoView {
    view! {
        <div class="p-4">
            <h1 class="text-xl font-bold">"Calls"</h1>
            <p class="text-gray-500">"Call activity logs — content coming in next task"</p>
        </div>
    }
}
```

**Step 2: Verify compilation**

```bash
cd ui && cargo check
```

**Step 3: Try building with Trunk (first WASM build)**

```bash
cd ui && trunk build
```

Expected: Builds successfully, creates `ui/dist/` with WASM files.

**Step 4: Commit**

```bash
git add ui/src/sections/
git commit -m "feat(ui): add Activities side navigation and stub calls page"
```

---

## Task 3: Build the Calls Page — Top Filter Bar

**Files:**
- Modify: `ui/src/sections/activities.rs`
- Create: `ui/src/components/mod.rs`
- Create: `ui/src/components/filter_bar.rs`

**Step 1: Create the shared FilterBar component**

This is the top bar with search, date range, filters, and view toggle — reused across all activity pages.

Reference: `.UI/prototype/index.html` lines 130-185 (the top bar with search input, date filter, "All Sources" dropdown, column visibility, and Phone button).

```rust
// ui/src/components/filter_bar.rs
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon;

#[component]
pub fn FilterBar(
    #[prop(optional, into)] search_placeholder: String,
) -> impl IntoView {
    let placeholder = if search_placeholder.is_empty() {
        "Search...".to_string()
    } else {
        search_placeholder
    };

    view! {
        <div class="flex items-center gap-3 px-4 py-2 bg-white border-b border-gray-200">
            // Search
            <div class="relative flex-1 max-w-xs">
                <Icon icon=icondata::BsSearch class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
                <input
                    type="text"
                    placeholder=placeholder
                    class="input input-sm input-ghost w-full pl-9 bg-gray-50 border-gray-200"
                />
            </div>
            // Date range
            <div class="flex items-center gap-2 text-sm text-gray-600">
                <Icon icon=icondata::BsCalendar3 class="w-4 h-4" />
                <span>"Last 30 Days"</span>
                <Icon icon=icondata::BsChevronDown class="w-3 h-3" />
            </div>
            // All Sources filter
            <button class="btn btn-sm btn-ghost text-gray-600 gap-1">
                "All Sources"
                <Icon icon=icondata::BsChevronDown class="w-3 h-3" />
            </button>
            // Spacer
            <div class="flex-1"></div>
            // Column visibility
            <button class="btn btn-sm btn-ghost btn-square">
                <Icon icon=icondata::BsLayoutThreeColumns class="w-4 h-4" />
            </button>
            // Phone button
            <button class="btn btn-sm bg-iiz-cyan text-white hover:bg-iiz-cyan/90 gap-1">
                <Icon icon=icondata::BsTelephoneFill class="w-3 h-3" />
                "Phone"
            </button>
        </div>
    }
}
```

**Step 2: Wire up in `main.rs`**

Add `mod components;` to `main.rs`.

**Step 3: Use FilterBar in CallsPage**

Update `CallsPage` to include `FilterBar` at the top.

**Step 4: Verify compilation**

```bash
cd ui && cargo check
```

**Step 5: Commit**

```bash
git add ui/src/
git commit -m "feat(ui): add shared FilterBar component with search, date, and filters"
```

---

## Task 4: Build the Calls Page — Call List Table

**Files:**
- Modify: `ui/src/sections/activities.rs`

**Step 1: Add mock call data and the call list table**

Reference: `.UI/prototype/index.html` lines 190-370 (the call activity list with columns: caller info, tags, source, duration, recording, time, score).

Create a `CallRecord` struct with mock data, then render a table matching the prototype layout. Each row has:
- Checkbox + call direction icon
- Caller name, number, location
- Tags (badge pills)
- Tracking source
- Duration
- Audio player mini
- Timestamp
- Score (colored circle)

Use hardcoded mock data matching the prototype screenshots.

**Step 2: Verify compilation and visual output**

```bash
cd ui && trunk serve
```

Open `http://localhost:3000` and verify the call list renders.

**Step 3: Commit**

```bash
git add ui/src/sections/activities.rs
git commit -m "feat(ui): add calls page with mock data table matching 4iiz layout"
```

---

## Task 5: Build the Calls Page — Detail Panel

**Files:**
- Create: `ui/src/components/detail_panel.rs`
- Modify: `ui/src/sections/activities.rs`

**Step 1: Create the slide-out detail panel**

Reference: `.UI/prototype/index.html` lines 400-900 (the right-side detail panel with tabs: Contact, Score, Text Messages, Email, Flow, Reminder, Voice Analysis, Visitor Details).

The detail panel:
- Slides in from the right when a call row is clicked
- Has a header with caller name, number, close button
- Tab navigation on the left side (vertical tabs)
- Content area that switches based on selected tab
- Contact tab shows: call summary, caller info grid, timeline

**Step 2: Wire click handler on call rows to open detail panel**

Add a `selected_call: RwSignal<Option<usize>>` signal. When a row is clicked, set it. When set, show the detail panel.

**Step 3: Verify compilation and interaction**

```bash
cd ui && trunk serve
```

Click a call row — detail panel should slide in. Click close — should slide out.

**Step 4: Commit**

```bash
git add ui/src/
git commit -m "feat(ui): add call detail panel with tabs and contact info"
```

---

## Task 6: Visual Comparison with Chrome DevTools

**Step 1: Serve the app**

```bash
cd ui && trunk serve
```

**Step 2: Open in Chrome and take screenshots**

Use Chrome DevTools MCP to:
1. Navigate to `http://localhost:3000`
2. Take a screenshot of the full page
3. Compare against original screenshots in `.UI/` directory

**Step 3: Compare and note differences**

Read the original 4iiz screenshots:
- `.UI/4iiz-home page.jpg` — main calls page
- `.UI/4iiz-home page - call details - contact.jpg` — detail panel

**Step 4: Iterate on CSS/layout until matching**

Adjust spacing, colors, font sizes, icon sizes, border weights, etc. until the Leptos render matches the original screenshots pixel-for-pixel.

**Step 5: Commit after each round of adjustments**

```bash
git add ui/src/
git commit -m "fix(ui): adjust calls page styling to match 4iiz screenshots"
```

---

## Task 7: Add Remaining Activity Log Pages

**Files:**
- Modify: `ui/src/sections/activities.rs`
- Modify: `ui/src/main.rs` (add routes)

**Step 1: Add stub pages for Texts, Forms, Chats, Faxes, Videos, Export Log**

Each follows the same pattern: FilterBar + table with section-specific columns + detail panel. Start with mock data matching the prototype screenshots.

**Step 2: Add routes in main.rs**

```rust
<Route path=path!("/activities/texts") view=sections::activities::TextsPage />
<Route path=path!("/activities/forms") view=sections::activities::FormsPage />
// etc.
```

**Step 3: Wire up side nav active state**

Use `leptos_router::hooks::use_location` to highlight the active side nav item.

**Step 4: Visual comparison for each page**

Screenshot each page and compare against the original `.UI/` screenshots.

**Step 5: Commit**

```bash
git add ui/src/
git commit -m "feat(ui): add all activity log pages (texts, forms, chats, faxes, videos, export)"
```

---

## Task 8: Contacts Section

**Files:**
- Create: `ui/src/sections/contacts.rs`
- Modify: `ui/src/sections/mod.rs`
- Modify: `ui/src/main.rs`

Reference prototypes: `.UI-Contacts/prototype/contacts.html`

**Step 1: Build Contacts pages**

- Lists page (contact table with search, pagination)
- Blocked Numbers page
- Do Not Call List page
- Do Not Text List page

**Step 2: Visual comparison against `.UI-Contacts/` screenshots**

**Step 3: Commit**

---

## Future Tasks (Sections 3-7)

Each section follows the same pattern:
1. Create `ui/src/sections/<section>.rs`
2. Add side nav component
3. Build each sub-page with mock data
4. Add routes
5. Visual comparison against screenshots
6. Iterate until matching

Sections in priority order:
- **Numbers** — `.UI-Numbers/prototype/numbers.html` (10 pages)
- **Reports** — `.UI-Reports/prototype/reports.html` (chart-heavy)
- **AI Tools** — `.UI-AITools/prototype/aitools.html` (form-heavy)
- **Flows** — `.UI-Flows/prototype/flows.html` (complex flow builder)
- **Trust Center** — `.UI-TrustCenter/prototype/trustcenter.html` (compliance cards)

---

## API Wiring (Phase 2 — after UI fidelity is confirmed)

Once all sections render with mock data and match the screenshots:
1. Create `ui/src/api/` module with typed API client
2. Replace mock data with `gloo-net` HTTP calls to `/api/*`
3. Add loading states (Skeleton components)
4. Add error states (Alert components)
5. Wire up form submissions
6. Add WebSocket for real-time monitoring
