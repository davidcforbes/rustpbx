# 4iiz Functional Requirements Specification

> Generated from UI component audit of the Leptos/WASM mockup prototype.
> Each page's requirements are derived from the UI elements, data models, and interactions visible in the implemented components.

## Document Index

| Chapter | Section | Pages | Status |
|---------|---------|-------|--------|
| [01 - Activities](01-activities.md) | Activity Logs | 7 | All well-defined |
| [02 - Contacts](02-contacts.md) | Contact Management | 4 | All well-defined |
| [03 - Numbers](03-numbers.md) | Number Management | 10 | All well-defined |
| [04 - Flows](04-flows.md) | Routing, Automation, Engagement | 23 | 22 well-defined, 1 stub |
| [05 - AI Tools](05-ai-tools.md) | AI Insights & Agents | 5 | 4 well-defined, 1 stub |
| [06 - Reports](06-reports.md) | Analytics, Connect, Usage, Settings | 30 | 10 well-defined, 20 need definition |
| [07 - Trust Center](07-trust-center.md) | Compliance & Registration | 8 | All well-defined |
| [08 - Shared Components](08-shared-components.md) | FilterBar, CallDetailPanel | 2 | Well-defined |
| [09 - Summary & Gaps](09-summary-and-gaps.md) | Status classification & open items | - | - |

## Application Architecture

- **Framework:** Leptos (Rust) compiled to WASM
- **Styling:** DaisyUI 5 + Tailwind CSS v4 via `leptos-daisyui-rs`
- **Routing:** `leptos_router` 0.8 with SPA client-side routing
- **State:** `RwSignal` / `Signal` reactive primitives
- **Icons:** `leptos_icons` with Bootstrap icon set (`Bs*`)
- **Data:** All data is static/mock -- no API calls, no backend integration yet

## Shell Layout

Three-column layout:
- **Column 1** (64px): Icon navigation rail with 6 section icons + Help/Settings at bottom
- **Column 2** (192px): Section side navigation panel (contextual per active section)
- **Column 3** (remaining): Main content area

Root route `/` redirects to `/activities/calls`.

URL-based section detection syncs the icon nav and side panel with the current page.

## Page Count Summary

| Section | Pages | Well-Defined | Needs Definition | Stub Only |
|---------|-------|-------------|-----------------|-----------|
| Activities | 7 | 7 | 0 | 0 |
| Contacts | 4 | 4 | 0 | 0 |
| Numbers | 10 | 10 | 0 | 0 |
| Flows | 23 | 22 | 0 | 1 |
| AI Tools | 5 | 4 | 0 | 1 |
| Reports | 30 | 10 | 20 | 0 |
| Trust Center | 8 | 8 | 0 | 0 |
| **TOTAL** | **87** | **65** | **20** | **2** |

## Status Definitions

- **Well-Defined**: Complete UI layout, data model, mock data, and clear implied functionality. Ready for backend integration.
- **Needs Definition**: UI exists but uses a generic template. Specific data models, chart types, metrics, filters, and columns need to be defined.
- **Stub Only**: Table headers or layout skeleton only, with no data or meaningful content. Requires full design.
