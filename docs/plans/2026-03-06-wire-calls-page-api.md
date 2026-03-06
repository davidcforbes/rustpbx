# Wire Calls Page API Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enrich the `/api/v1/activities/calls` endpoint to return all fields the new Calls page UI needs (source type, agent initials, tracking number, routing info, annotation category, tags) and update the UI types to consume them.

**Architecture:** Batch-enrichment pattern. Query call_records first (25 rows), then batch-fetch related data from tracking_sources, users, tracking_numbers, call_annotations, and call_tags+tags using `WHERE id IN (...)`. Assemble into a `CallRecordListItem` response DTO. This avoids complex multi-table JOINs while keeping queries fast (all indexed lookups).

**Tech Stack:** Diesel 2.2 + diesel-async (backend), Leptos 0.8 (frontend), PostgreSQL

---

### Task 1: Create CallRecordListItem Response DTO

**Files:**
- Modify: `src/iiz/models/activities.rs` (add new struct after existing CallRecord)

**Step 1: Add the response DTO struct**

Add after the existing `CallRecord` struct:

```rust
/// Enriched call record for list responses — includes joined data from
/// tracking_sources, users, tracking_numbers, call_annotations, and tags.
#[derive(Debug, Clone, Serialize)]
pub struct CallRecordListItem {
    // Core fields from call_records
    pub id: Uuid,
    pub call_id: String,
    pub caller_phone: Option<String>,
    pub callee_phone: Option<String>,
    pub direction: CallDirection,
    pub status: CallStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub answered_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_secs: i32,
    pub ring_duration_secs: i32,
    pub hold_duration_secs: i32,
    pub has_audio: bool,
    pub is_first_time_caller: bool,
    pub location: Option<String>,
    pub recording_url: Option<String>,
    // Denormalized from call_records
    pub source_name: Option<String>,
    pub agent_name: Option<String>,
    pub queue_name: Option<String>,
    // Enriched from tracking_sources (via source_id)
    pub source_type: Option<String>,
    // Enriched from tracking_numbers (via source_number_id)
    pub tracking_number: Option<String>,
    pub routing_description: Option<String>,
    pub receiving_number: Option<String>,
    // Enriched from users (via agent_id)
    pub agent_initials: Option<String>,
    pub agent_avatar_color: Option<String>,
    // Enriched from call_annotations (via call_id)
    pub annotation_category: Option<String>,
    pub annotation_score: Option<i32>,
    // Enriched from call_tags + tags (via call_id)
    pub tags: Vec<String>,
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p rustpbx 2>&1 | grep -E "^error" | head -5`
Expected: No errors (new struct is unused so far)

**Step 3: Commit**

```bash
git add src/iiz/models/activities.rs
git commit -m "feat(iiz): add CallRecordListItem response DTO for enriched call data"
```

---

### Task 2: Implement Batch Enrichment in Calls List Handler

**Files:**
- Modify: `src/iiz/api/activities/calls.rs` (rewrite `list` function)

**Step 1: Rewrite the list handler with batch enrichment**

Replace the `list` function (lines 28-60) with:

```rust
use std::collections::HashMap;
use crate::iiz::models::activities::CallRecordListItem;

/// Paginated list of call records with enriched data from related tables.
///
/// GET `/activities/calls?page=1&per_page=25&sort=started_at:desc`
pub async fn list(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<CallRecordListItem>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::call_records::dsl::*;

    // 1. Count + fetch base call records
    let total: i64 = call_records
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let base_items: Vec<CallRecord> = call_records
        .order(started_at.desc())
        .offset(offset)
        .limit(limit)
        .load(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    if base_items.is_empty() {
        let meta = PaginationMeta::new(params.page.max(1), limit, total);
        return Ok(axum::Json(ListResponse { pagination: meta, items: vec![] }));
    }

    // Collect FKs for batch lookups
    let call_ids: Vec<Uuid> = base_items.iter().map(|c| c.id).collect();
    let source_ids: Vec<Uuid> = base_items.iter().filter_map(|c| c.source_id).collect();
    let agent_ids: Vec<Uuid> = base_items.iter().filter_map(|c| c.agent_id).collect();
    let source_number_ids: Vec<Uuid> = base_items.iter().filter_map(|c| c.source_number_id).collect();

    // 2. Batch fetch tracking_sources for source_type
    let source_types: HashMap<Uuid, String> = if !source_ids.is_empty() {
        use crate::iiz::schema::iiz::tracking_sources::dsl as ts;
        let rows: Vec<(Uuid, Option<String>)> = ts::tracking_sources
            .filter(ts::id.eq_any(&source_ids))
            .select((ts::id, ts::source_type))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect()
    } else {
        HashMap::new()
    };

    // 3. Batch fetch users for agent initials + avatar_color
    let agent_info: HashMap<Uuid, (Option<String>, Option<String>)> = if !agent_ids.is_empty() {
        use crate::iiz::schema::iiz::users::dsl as u;
        let rows: Vec<(Uuid, Option<String>, Option<String>)> = u::users
            .filter(u::id.eq_any(&agent_ids))
            .select((u::id, u::initials, u::avatar_color))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().map(|(k, ini, clr)| (k, (ini, clr))).collect()
    } else {
        HashMap::new()
    };

    // 4. Batch fetch tracking_numbers for number + routing_description + receiving_number_id
    let tn_info: HashMap<Uuid, (String, Option<String>, Option<Uuid>)> = if !source_number_ids.is_empty() {
        use crate::iiz::schema::iiz::tracking_numbers::dsl as tn;
        let rows: Vec<(Uuid, String, Option<String>, Option<Uuid>)> = tn::tracking_numbers
            .filter(tn::id.eq_any(&source_number_ids))
            .select((tn::id, tn::number, tn::routing_description, tn::receiving_number_id))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().map(|(k, num, rd, rn)| (k, (num, rd, rn))).collect()
    } else {
        HashMap::new()
    };

    // 5. Batch fetch receiving_numbers for the actual phone number
    let recv_number_ids: Vec<Uuid> = tn_info.values().filter_map(|(_, _, rn)| *rn).collect();
    let recv_numbers: HashMap<Uuid, String> = if !recv_number_ids.is_empty() {
        use crate::iiz::schema::iiz::receiving_numbers::dsl as rn;
        let rows: Vec<(Uuid, String)> = rn::receiving_numbers
            .filter(rn::id.eq_any(&recv_number_ids))
            .select((rn::id, rn::number))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().collect()
    } else {
        HashMap::new()
    };

    // 6. Batch fetch call_annotations for category + score
    let annotations: HashMap<Uuid, (Option<String>, Option<i32>)> = {
        use crate::iiz::schema::iiz::call_annotations::dsl as ca;
        let rows: Vec<(Uuid, Option<String>, Option<i32>)> = ca::call_annotations
            .filter(ca::call_id.eq_any(&call_ids))
            .select((ca::call_id, ca::category, ca::score))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().map(|(k, cat, sc)| (k, (cat, sc))).collect()
    };

    // 7. Batch fetch tags (call_tags JOIN tags)
    let tag_map: HashMap<Uuid, Vec<String>> = {
        use crate::iiz::schema::iiz::call_tags::dsl as ct;
        use crate::iiz::schema::iiz::tags::dsl as tg;
        let rows: Vec<(Uuid, String)> = ct::call_tags
            .inner_join(tg::tags.on(tg::id.eq(ct::tag_id)))
            .filter(ct::call_id.eq_any(&call_ids))
            .filter(ct::deleted_at.is_null())
            .select((ct::call_id, tg::name))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        let mut map: HashMap<Uuid, Vec<String>> = HashMap::new();
        for (cid, name) in rows {
            map.entry(cid).or_default().push(name);
        }
        map
    };

    // 8. Assemble enriched items
    let items: Vec<CallRecordListItem> = base_items
        .into_iter()
        .map(|c| {
            let src_type = c.source_id.and_then(|sid| source_types.get(&sid).cloned());
            let (agent_ini, agent_clr) = c.agent_id
                .and_then(|aid| agent_info.get(&aid).cloned())
                .unwrap_or((None, None));
            let (tn_number, tn_routing, tn_recv_id) = c.source_number_id
                .and_then(|tnid| tn_info.get(&tnid).cloned())
                .unwrap_or((String::new(), None, None));
            let recv_num = tn_recv_id.and_then(|rid| recv_numbers.get(&rid).cloned());
            let (ann_cat, ann_score) = annotations.get(&c.id).cloned().unwrap_or((None, None));
            let call_tags = tag_map.get(&c.id).cloned().unwrap_or_default();

            CallRecordListItem {
                id: c.id,
                call_id: c.call_id,
                caller_phone: c.caller_phone,
                callee_phone: c.callee_phone,
                direction: c.direction,
                status: c.status,
                started_at: c.started_at,
                answered_at: c.answered_at,
                ended_at: c.ended_at,
                duration_secs: c.duration_secs,
                ring_duration_secs: c.ring_duration_secs,
                hold_duration_secs: c.hold_duration_secs,
                has_audio: c.has_audio,
                is_first_time_caller: c.is_first_time_caller,
                location: c.location,
                recording_url: c.recording_url,
                source_name: c.source_name,
                agent_name: c.agent_name,
                queue_name: c.queue_name,
                source_type: src_type,
                tracking_number: if tn_number.is_empty() { None } else { Some(tn_number) },
                routing_description: tn_routing,
                receiving_number: recv_num,
                agent_initials: agent_ini,
                agent_avatar_color: agent_clr,
                annotation_category: ann_cat,
                annotation_score: ann_score,
                tags: call_tags,
            }
        })
        .collect();

    let meta = PaginationMeta::new(params.page.max(1), limit, total);
    Ok(axum::Json(ListResponse { pagination: meta, items }))
}
```

**Step 2: Add sort support to ListParams**

In `src/iiz/api/pagination.rs`, the `ListParams` already has `sort: Option<String>`. Apply it in the handler if provided (parse `"field:dir"` format). For now, default to `started_at.desc()`.

**Step 3: Verify it compiles**

Run: `cargo check -p rustpbx 2>&1 | grep "^error" | head -10`
Expected: Clean compile (or fix any type mismatches from schema column types)

**Step 4: Commit**

```bash
git add src/iiz/api/activities/calls.rs
git commit -m "feat(iiz): enrich calls list with source type, agent info, routing, annotations, tags"
```

---

### Task 3: Verify Schema Joinability

**Files:**
- Check: `src/iiz/schema.rs` for `allow_tables_to_appear_in_same_query!` and `joinable!` macros

**Step 1: Check that call_tags can JOIN tags**

Grep for existing joinable macros. If `call_tags -> tags` join is missing, add it:

```rust
diesel::joinable!(call_tags -> tags (tag_id));
```

And ensure both tables appear in `allow_tables_to_appear_in_same_query!`.

**Step 2: Verify compile after any schema changes**

Run: `cargo check -p rustpbx 2>&1 | grep "^error" | head -10`

**Step 3: Commit if schema changes needed**

```bash
git add src/iiz/schema.rs
git commit -m "fix(iiz): add joinable macro for call_tags -> tags"
```

---

### Task 4: Update UI CallRecordItem Type

**Files:**
- Modify: `ui/src/api/types.rs` (update CallRecordItem struct)

**Step 1: Update the struct to match new API response**

Replace the existing `CallRecordItem` struct with:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecordItem {
    pub id: String,
    pub call_id: String,
    pub caller_phone: Option<String>,
    pub callee_phone: Option<String>,
    pub direction: String,
    pub status: String,
    pub started_at: String,
    pub answered_at: Option<String>,
    pub ended_at: Option<String>,
    pub duration_secs: i32,
    pub ring_duration_secs: Option<i32>,
    pub hold_duration_secs: Option<i32>,
    pub has_audio: bool,
    pub is_first_time_caller: Option<bool>,
    pub location: Option<String>,
    pub recording_url: Option<String>,
    // Denormalized
    pub source_name: Option<String>,
    pub agent_name: Option<String>,
    pub queue_name: Option<String>,
    // Enriched
    pub source_type: Option<String>,
    pub tracking_number: Option<String>,
    pub routing_description: Option<String>,
    pub receiving_number: Option<String>,
    pub agent_initials: Option<String>,
    pub agent_avatar_color: Option<String>,
    pub annotation_category: Option<String>,
    pub annotation_score: Option<i32>,
    pub tags: Option<Vec<String>>,
}
```

Note: Use `Option<Vec<String>>` for tags with `#[serde(default)]` so missing field defaults to None.

**Step 2: Verify WASM compile**

Run: `cd ui && cargo check --target wasm32-unknown-unknown 2>&1 | grep "^error" | head -10`

**Step 3: Commit**

```bash
git add ui/src/api/types.rs
git commit -m "feat(ui): update CallRecordItem type with enriched fields from API"
```

---

### Task 5: Update UI call_record_from_api Mapping

**Files:**
- Modify: `ui/src/sections/activities.rs` (update `call_record_from_api` function)

**Step 1: Map all the new fields**

Update the mapping function to use the new API fields:

```rust
fn call_record_from_api(item: CallRecordItem) -> CallRecord {
    let location = item.location.clone().unwrap_or_default();
    let duration = fmt_duration(item.duration_secs);
    let date = format_friendly_date(&item.started_at);
    let time = format_friendly_time(&item.started_at);
    let name = item.caller_phone.clone().unwrap_or_default(); // Use phone as name if no contact name
    let contact_initials = initials_from_name(&name);
    let contact_color = color_from_string(&name);
    let source = item.source_name.clone().unwrap_or_default();
    let src_type = item.source_type.clone().unwrap_or_default();

    CallRecord {
        id: item.id,
        contact_initials,
        contact_color,
        name,
        phone: item.caller_phone.unwrap_or_default(),
        location,
        source: source.clone(),
        source_number: item.tracking_number.unwrap_or_default(),
        source_name: item.source_name.unwrap_or_default(),
        source_type: src_type,
        has_audio: item.has_audio,
        duration,
        date,
        time,
        status: item.status,
        agent: item.agent_name.clone().unwrap_or_default(),
        agent_initials: item.agent_initials.unwrap_or_default(),
        agent_color: item.agent_avatar_color.unwrap_or_else(|| "#0277bd".to_string()),
        automation: String::new(),
        tags: item.tags.unwrap_or_default(),
        receiving_number: item.receiving_number.unwrap_or_default(),
        routing_destination: item.routing_description.unwrap_or_default(),
        case_description: String::new(),
        contact_category: item.annotation_category.unwrap_or_default(),
        crm_contact_id: String::new(),
        crm_matter_id: String::new(),
        case_subtype: String::new(),
        matter_status: String::new(),
        answered_by: String::new(),
    }
}
```

**Step 2: Verify WASM compile**

Run: `cd ui && cargo check --target wasm32-unknown-unknown 2>&1 | grep "^error" | head -10`

**Step 3: Commit**

```bash
git add ui/src/sections/activities.rs
git commit -m "feat(ui): wire CallRecord mapping to enriched API fields"
```

---

### Task 6: Make FilterBar Call Count Dynamic

**Files:**
- Modify: `ui/src/components/filter_bar.rs` (add total_count prop)
- Modify: `ui/src/sections/activities.rs` (pass count to FilterBar)

**Step 1: Add optional count prop to FilterBar**

```rust
#[component]
pub fn FilterBar(
    #[prop(optional, into)] total_count: Signal<Option<i64>>,
) -> impl IntoView {
```

Replace the hardcoded "0 calls" span with:

```rust
<span class="text-sm text-gray-500 flex items-center gap-1 whitespace-nowrap">
    <span class="w-3.5 h-3.5 inline-flex"><Icon icon=icondata::BsBarChartFill /></span>
    {move || {
        total_count.get()
            .map(|n| format!("{} calls", n))
            .unwrap_or_else(|| "-- calls".to_string())
    }}
</span>
```

**Step 2: Update CallsPage to pass count**

In CallsPage, create a derived signal from the API response and pass it:

```rust
let total_count = Signal::derive(move || {
    data.get()
        .and_then(|r| r.ok())
        .map(|r| r.pagination.total_items)
});

// In the view:
<FilterBar total_count=total_count />
```

**Step 3: Update other pages that use FilterBar**

Other activity pages (Texts, Forms, etc.) should pass their own counts or use the default.

**Step 4: Verify WASM compile + commit**

```bash
cd ui && cargo check --target wasm32-unknown-unknown
git add ui/src/components/filter_bar.rs ui/src/sections/activities.rs
git commit -m "feat(ui): make FilterBar call count dynamic from API pagination"
```

---

### Task 7: Add Column Sorting

**Files:**
- Modify: `ui/src/sections/activities.rs` (add sort signals and header click handlers)
- Modify: `src/iiz/api/activities/calls.rs` (apply sort param in query)

**Step 1: Backend — Apply sort param**

In the calls list handler, parse `params.sort` and apply ordering:

```rust
// After fetching total count, before fetching items:
let mut query = call_records.into_boxed();

// Apply sort
match params.sort.as_deref() {
    Some("caller_phone:asc") => { query = query.order(caller_phone.asc()); }
    Some("caller_phone:desc") => { query = query.order(caller_phone.desc()); }
    Some("status:asc") => { query = query.order(status.asc()); }
    Some("status:desc") => { query = query.order(status.desc()); }
    Some("duration_secs:asc") => { query = query.order(duration_secs.asc()); }
    Some("duration_secs:desc") => { query = query.order(duration_secs.desc()); }
    _ => { query = query.order(started_at.desc()); }
}

let base_items: Vec<CallRecord> = query
    .offset(offset)
    .limit(limit)
    .load(&mut *conn)
    .await
    .map_err(ApiError::from)?;
```

**Step 2: Frontend — Add sort state and header clicks**

In CallsPage, add sort signals:

```rust
let sort_field = RwSignal::new(String::from("started_at"));
let sort_dir = RwSignal::new(String::from("desc"));

let data = LocalResource::new(move || {
    let field = sort_field.get();
    let dir = sort_dir.get();
    async move {
        let url = format!("/activities/calls?page=1&per_page=25&sort={}:{}", field, dir);
        api_get::<ListResponse<CallRecordItem>>(&url).await
    }
});
```

Add click handlers on column headers that toggle sort direction.

**Step 3: Verify both compiles + commit**

```bash
cargo check -p rustpbx
cd ui && cargo check --target wasm32-unknown-unknown
git add src/iiz/api/activities/calls.rs ui/src/sections/activities.rs
git commit -m "feat: add column sorting to calls list endpoint and UI"
```

---

### Task 8: Build, Deploy, and Test

**Step 1: Full backend build**

```bash
cargo build --release 2>&1 | tail -5
```

**Step 2: Full UI build**

```bash
cd ui && trunk build --release 2>&1 | tail -5
```

**Step 3: Fix any warnings**

Address all compiler warnings in modified files.

**Step 4: Deploy to Linode**

```bash
# Deploy UI
scp -i ~/.ssh/rustpbx_server ui/dist/* root@74.207.251.126:/root/rustpbx/ui/dist/
ssh -i ~/.ssh/rustpbx_server root@74.207.251.126 "systemctl restart rustpbx-ui"

# Deploy backend (if building on Linode or via WSL)
# wsl -d Ubuntu -- bash -c "cd ~/rustpbx && git pull fork main && cargo build --release"
# Then scp binary and restart
```

**Step 5: Verify in browser**

Open `https://74.207.251.126:3000/activities/calls` and verify:
- Column headers render correctly
- Data rows populate with enriched fields
- Source type icons show correct brand icons
- Agent initials and avatar colors display
- Tags render in contact column
- Routing column shows receiving number
- Date format is friendly ("Thu Mar 5th")
- Sort clicking works on column headers

**Step 6: Close beads issues**

```bash
bd close IIZ-cm7 IIZ-pc8 IIZ-lqy IIZ-n0w IIZ-zrd IIZ-t11 IIZ-x9z IIZ-3xh
bd sync
```

**Step 7: Final commit and push**

```bash
git add -A
git commit -m "feat: wire calls page UI to enriched API data — all beads tasks complete"
git push
```
