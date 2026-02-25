# 4iiz Leptos WASM UI — Design Document

**Date:** 2026-02-24
**Status:** Approved

## Overview

Build the 4iiz product UI as a parallel Leptos WASM SPA alongside the existing RustPBX admin console. The SPA consumes RustPBX's REST API and runs entirely client-side.

## Architecture

```
Browser
  /console/*  →  MiniJinja admin UI (existing, untouched)
  /app/*      →  Leptos WASM SPA (new 4iiz product UI)
  /api/*      →  REST API (shared, some endpoints may need adding)
```

The Leptos app compiles to static files (HTML + JS + WASM) via Trunk. Axum serves them with `ServeDir` on `/app/`. No server-side Leptos required.

## Project Structure

```
RustPBX/
  ui/                           # 4iiz Leptos SPA
    Cargo.toml                  # leptos, leptos-router, leptos-daisyui-rs, gloo-net
    Trunk.toml                  # Build config, Tailwind pre-build hook
    index.html                  # SPA entry point
    input.css                   # Tailwind + DaisyUI config
    tailwind.config.js          # Tailwind content paths
    src/
      main.rs                   # mount_to_body, Router, AppShell layout
      api/
        mod.rs                  # HTTP client, auth, error handling
        calls.rs                # Call records API
        contacts.rs             # Contacts API
        extensions.rs           # Extensions API
      sections/
        mod.rs                  # Section exports
        activities.rs           # Activity logs (calls, texts, forms, chats, faxes, videos)
        contacts.rs             # Contacts, lists, blocked, DNC/DNT
        flows.rs                # Routing, automation, engagement
        ai_tools.rs             # AskAI, summaries, knowledge banks, VoiceAI
        numbers.rs              # Tracking, receiving, target numbers, pools
        reports.rs              # Analytics, usage, exports
        trust_center.rs         # Compliance, A2P, STIR/SHAKEN
      components/               # App-specific shared components
        mod.rs
        filter_bar.rs           # Top filter/search bar
        master_list.rs          # Scrollable list with row selection
        detail_panel.rs         # Slide-out detail view with tabs
        page_header.rs          # Section title + action buttons
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `leptos` | Reactive UI framework |
| `leptos_router` | Client-side routing |
| `leptos-daisyui-rs` | DaisyUI component library (includes AppShell) |
| `leptos_icons` | Icon library |
| `gloo-net` | WASM-compatible HTTP client |
| `serde` / `serde_json` | JSON serialization |
| `chrono` | Date/time handling |
| `web-sys` | Browser APIs |

## Routing

```
/app/                         → Activities (default)
/app/activities/calls         → Call logs
/app/activities/texts         → Text logs
/app/activities/forms         → Form submissions
/app/activities/chats         → Chat logs
/app/activities/faxes         → Fax logs
/app/activities/videos        → Video logs
/app/activities/export        → Export log
/app/contacts                 → Contact lists
/app/contacts/blocked         → Blocked numbers
/app/contacts/dnc             → Do Not Call
/app/contacts/dnt             → Do Not Text
/app/flows                    → Flow builder
/app/ai-tools                 → AI tools
/app/numbers                  → Number management
/app/reports                  → Reports/analytics
/app/trust-center             → Compliance
```

## API Integration Strategy

**Phase 1 (UI First):** Use hardcoded mock data matching the prototypes. This lets us focus on pixel-perfect UI fidelity without backend dependencies.

**Phase 2 (Wire Up):** Replace mock data with `gloo-net` API calls to RustPBX endpoints. The existing API covers: call records, extensions, trunks, routing, settings, monitoring, voicemail, presence.

**Phase 3 (Extend):** Add any missing API endpoints as discovered (contacts CRUD, number management, flow builder state, report aggregations).

## Existing API Coverage

| 4iiz Section | RustPBX API Status |
|-------------|-------------------|
| Activities (calls) | Full coverage — call records CRUD, recordings, transcripts |
| Activities (texts/forms/chats) | Partial — may need new endpoints |
| Contacts | Missing — need contacts CRUD endpoints |
| Flows | Partial — routing exists, automation/engagement need endpoints |
| AI Tools | Missing — need AI config endpoints |
| Numbers | Missing — need number management endpoints |
| Reports | Partial — call records have stats, need aggregation endpoints |
| Trust Center | Missing — need compliance endpoints |

## Visual Comparison Workflow

After building each section:
1. Serve the Leptos app via `trunk serve`
2. Use Chrome DevTools MCP to take screenshots
3. Compare against original 4iiz screenshots in `.UI*` directories
4. Iterate on CSS/layout until matching

## Build & Dev Workflow

```bash
# Development (hot-reload)
cd ui && trunk serve --proxy-backend=http://localhost:8080/api

# Production build
cd ui && trunk build --release
# Output: ui/dist/ → served by Axum on /app/*
```

## Conversion Priority

1. **Activities** — Most representative, establishes all shared patterns
2. **Contacts** — Simple CRUD, validates table/list patterns
3. **Numbers** — Multi-page forms, validates wizard patterns
4. **Reports** — Charts, validates data visualization
5. **AI Tools** — Form-heavy, validates input patterns
6. **Flows** — Complex (flow builder), defer visual builder
7. **Trust Center** — Simple compliance cards
