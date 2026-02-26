# 4iiz Data Storage Architecture Design

> **Status:** Approved
> **Date:** 2026-02-25
> **Prerequisite:** [Data Element Dictionary](../database_requirements/00-data-dictionary-index.md)
> **Next Step:** Implementation plan (schema DDL, SeaORM migrations, connection pooling)

---

## 1. Purpose

Define WHERE each data entity is stored and in WHAT FORM, based on the data element dictionary (~99 entities across 9 domain shards). The design must guarantee no blocked processes and no deadly embrace (deadlocks) under concurrent production workloads.

---

## 2. Data Access Pattern Classification

Every entity in the system falls into one of six categories based on its write frequency, mutation pattern, read pattern, and lifecycle.

### 2.1 Event Log (Immutable / Append-Only / WORM)

Written once at event completion and never modified. The dominant pattern in telecom — CDRs, transcription segments, webhook deliveries, API logs. Industry standard is WORM (Write Once, Read Many).

- **Write pattern:** Single INSERT at event completion
- **Read pattern:** Heavy — reporting, analytics, search, replay
- **Mutation:** Never. Corrections are compensating records, not UPDATEs
- **Volume:** Highest — millions to billions of rows
- **Retention:** Months to years with hot/warm/cold tiering
- **Industry precedent:** Amazon Connect streams Contact Records as append-only JSON to Kinesis/S3. Cisco CDRs are immutable event logs. Splunk ingests CDR data as immutable time-series events.

### 2.2 Ephemeral Real-Time State (Volatile / In-Memory)

Active calls, agent presence, queue depths. Exists only while the event is in progress, then transitions to an Event Log record or is discarded.

- **Write pattern:** Frequent updates (status changes every seconds)
- **Read pattern:** Constant polling or pub/sub for dashboards
- **Mutation:** Continuous — state changes in place
- **Volume:** Small (bounded by concurrent activity)
- **Retention:** None — evaporates on completion
- **Industry precedent:** Redis is the universal standard for this tier.

### 2.3 Configuration Data (Mutable / Low-Volume / Transactional)

Queues, routers, IVR menus, schedules, webhook configs, AI agent settings. Written rarely by admins, read on every call. Classic OLTP.

- **Write pattern:** Infrequent (admin creates/edits)
- **Read pattern:** Every call evaluates routing config — read-hot
- **Mutation:** Full CRUD with transactional consistency
- **Volume:** Low (hundreds to low thousands per account)
- **Retention:** Indefinite (until explicitly deleted)

### 2.4 Reference / Compliance (Write-Rarely / Audit-Critical)

Business identity, carrier registrations, CNAM records, DNC/DNT lists. Written during onboarding or regulatory events. Every change must be auditable.

- **Write pattern:** Very infrequent (onboarding, annual re-verification)
- **Read pattern:** Low (compliance checks, status display)
- **Mutation:** Rare but requires full audit trail
- **Volume:** Very low (single digits to low hundreds per account)
- **Retention:** Regulatory — 5-7 years minimum

### 2.5 Aggregated / Analytical (Computed / Read-Optimized)

Daily summaries, source attribution rollups, agent performance scores. Not written by users — computed from Event Log data by batch jobs or materialized views.

- **Write pattern:** Batch recompute (hourly/daily) or incremental
- **Read pattern:** Heavy — every report page hits this tier
- **Mutation:** Overwritten on recompute, not user-editable
- **Volume:** Moderate (one row per dimension-combination per day)
- **Retention:** Matches source event data

### 2.6 Binary / Object Data (Write-Once / Large-Payload)

Call recordings, fax documents, voicemail audio, knowledge bank uploads. Written once, referenced by metadata pointers in the relational layer.

- **Write pattern:** Single write at creation
- **Read pattern:** On-demand playback/download
- **Mutation:** Never (new version = new object)
- **Volume:** Largest by bytes (TB-scale for recordings)
- **Retention:** Configurable with lifecycle tiering (hot SSD → warm HDD → cold glacier)

---

## 3. Entity-to-Category Classification

### 3.1 Event Log Entities

| Entity | Shard | Hybrid Aspect |
|--------|-------|---------------|
| call_records | 01 | Core CDR fields only — immutable (see Section 5.1) |
| call_annotations | 01 | Mutable overlay — split from call_records (see Section 5.1) |
| call_flow_events | 01 | Pure append-only |
| call_visitor_sessions | 01 | Written once per call |
| call_transcription_segments | 01 | Written once after ASR |
| call_ai_summaries | 01 | Written once per summary type |
| call_keyword_hits | 01 | Written once after analysis |
| call_tags | 07 | Insert + delete (no update) |
| text_records | 01 | Immutable |
| text_messages | 01 | Immutable once sent |
| form_records | 01 | Immutable |
| chat_records | 01 | Written at session close |
| fax_records | 01 | Immutable |
| video_records | 01 | Immutable |
| export_records | 01 | Status transitions during processing, then immutable |
| agent_state_log | 02 | Append-only; duration_secs backfilled once |
| notifications | 02 | Append-only creation; is_read is mutable (single boolean) |
| voicemail_messages | 04 | Append-only; is_read is mutable (single boolean) |
| webhook_deliveries | 05 | Append-only delivery log |
| voice_registration_history | 08 | Append-only audit trail |
| api_log_entries | 09 | Pure append-only |
| monitoring_events | 09 | Append-only supervision log |

### 3.2 Ephemeral Real-Time State Entities

| Entity | Shard | Notes |
|--------|-------|-------|
| active_calls | 09 | Evaporates on completion → becomes call_record |
| presence | 09 | Mutable in-place; one row per identity |
| locations | 09 | SIP registrations with TTL expiration |
| frequency_limits | 09 | Sliding windows; needs durability across restarts |

### 3.3 Configuration Entities

| Entity | Shard |
|--------|-------|
| accounts | 02 |
| users | 02 |
| contact_lists, contact_list_members | 02 |
| blocked_numbers | 02 |
| tracking_numbers | 03 |
| receiving_numbers | 03 |
| text_numbers | 03 |
| target_numbers | 03 |
| tracking_sources | 03 |
| number_pools, number_pool_members | 03 |
| call_settings | 03 |
| voice_menus, voice_menu_options | 04 |
| queues, queue_agents | 04 |
| smart_routers, smart_router_rules | 04 |
| geo_routers, geo_router_rules | 04 |
| schedules, schedule_holidays | 04 |
| routing_tables, routing_table_routes | 04 |
| agent_scripts | 04 |
| voicemail_boxes | 04 |
| workflows, workflow_nodes, workflow_edges | 05 |
| triggers, trigger_conditions, trigger_actions | 05 |
| lambdas, lambda_env_vars | 05 |
| webhooks, webhook_subscriptions | 05 |
| bulk_messages | 05 |
| lead_reactor_configs, lead_reactor_actions | 05 |
| smart_dialer_configs | 05 |
| form_reactor_entries | 05 |
| reminders | 05 |
| keyword_spotting_configs, keyword_spotting_keywords, keyword_spotting_numbers | 05 |
| chat_widgets | 05 |
| ask_ai_configs | 06 |
| summary_configs | 06 |
| knowledge_banks, knowledge_bank_documents | 06 |
| voice_ai_agents | 06 |
| chat_ai_agents, chat_ai_configs | 06 |
| dialogflow_configs | 06 |
| tags | 07 |
| scoring_configs | 07 |
| custom_reports | 07 |
| notification_rules | 07 |
| account_variables | 09 |

### 3.4 Reference / Compliance Entities

| Entity | Shard |
|--------|-------|
| business_info | 08 |
| authorized_contacts | 08 |
| a2p_campaigns | 08 |
| toll_free_registrations | 08 |
| voice_registrations | 08 |
| caller_id_cnam | 08 |
| compliance_requirements | 08 |
| compliance_applications | 08 |
| compliance_addresses | 08 |
| port_requests | 03 |
| dnc_entries | 02 |
| dnt_entries | 02 |

### 3.5 Aggregated / Analytical Entities

| Entity | Shard | Notes |
|--------|-------|-------|
| call_daily_summary | 07 | Pre-aggregated star-schema fact table |
| appointments | 07 | User-created but primarily consumed by reports |

### 3.6 Binary / Object Data

| Referenced By | Field | Content |
|---------------|-------|---------|
| call_records | recording_url | Call recordings (WAV/MP3) |
| fax_records | document_url | Fax documents (PDF/TIFF) |
| voicemail_messages | audio_url | Voicemail audio |
| knowledge_bank_documents | file_ref | RAG source documents |
| voice_menus | greeting_audio_url | IVR greetings |
| voicemail_boxes | greeting_audio_url | Voicemail greetings |
| queues | moh_audio_url | Music on hold |
| export_records | download_url | CSV/PDF/Excel exports |
| video_records | recording_url | Video recordings |

---

## 4. Storage Engine Selection

### 4.1 Approach: Unified PostgreSQL, Design for Extraction

Single PostgreSQL instance with different schema patterns per data category. Redis sidecar for ephemeral state. S3-compatible object store for binary data. pgvector extension for knowledge bank embeddings.

This is the simplest operational footprint. The design enforces clean boundaries between categories so that any tier can be extracted to a specialized engine later (ClickHouse for analytics, dedicated vector DB for embeddings) without application-layer rewrites.

| Category | Data Form | Storage Engine | Schema Pattern |
|----------|-----------|---------------|----------------|
| Event Log | Denormalized wide table | PostgreSQL (partitioned by month) | Append-only, time-partitioned |
| Ephemeral State | Key-Value / Hash | Redis | Hash per entity, TTL for expiry |
| Configuration | 3rd Normal Form | PostgreSQL | Standard relational with FKs |
| Compliance | 3NF + audit columns | PostgreSQL | Same as config, with created_at/updated_at |
| Analytical | Star schema (fact + dimension) | PostgreSQL materialized views | Dimension tables are config entities |
| Binary Objects | Object store with metadata pointers | S3-compatible / local filesystem | file_ref columns in relational layer |
| Vector Embeddings | Vector index | pgvector extension | Co-located in PostgreSQL |

### 4.2 PostgreSQL Standardization

This design standardizes on PostgreSQL and drops SQLite/MySQL multi-backend support for the 4iiz application layer. Rationale:

- **Partitioning** (required for event log tables) is PostgreSQL-specific
- **Materialized views** with `CONCURRENTLY` refresh are PostgreSQL-specific
- **pgvector** extension is PostgreSQL-only
- **LISTEN/NOTIFY** for cache invalidation is PostgreSQL-specific
- **`SKIP LOCKED`** for queue-like patterns is PostgreSQL-specific

RustPBX core SIP/media functionality can retain multi-backend support. The 4iiz data layer is PostgreSQL-only.

### 4.3 Data Form Rationale

**Event Log → Denormalized Wide Table (not 3NF)**

Event log entities are written once and read many times in aggregate. Normalizing them (e.g., separate tables for caller info, source info, agent info on each call) would require joins on every report query across millions of rows. Instead:

- Inline the most-queried fields directly on the wide row (caller_phone, source_name, agent_name)
- Accept denormalization cost (storage) in exchange for query performance
- The source-of-truth for entities like TrackingSource and User remains in the 3NF config layer; the wide table stores snapshot copies at event time

**Configuration → 3NF**

Configuration entities have clean entity-relationship structure, low volume, and need referential integrity. Full normalization is appropriate. JOINs are cheap at this volume (hundreds of rows).

**Analytical → Star Schema**

CallDailySummary is a fact table with dimension keys (date, source_id, agent_id, queue_id). The dimension tables are the configuration entities themselves (TrackingSource, User, Queue). This is a natural star schema that enables fast aggregation queries for all 30 report pages.

**Workflow Graphs → Hybrid (JSON + Normalized)**

Workflow canvas layout is stored as a JSON blob (canvas_json) for UI rendering. The execution engine reads from normalized workflow_nodes and workflow_edges tables. Dual representation avoids parsing JSON at execution time while preserving the full visual layout.

---

## 5. Critical Design Decisions

### 5.1 CallRecord Split: Immutable CDR + Mutable Annotations

The highest-volume entity (CallRecord) is split into two tables to eliminate write-write contention:

**call_records** (immutable after INSERT):
- call_id, caller_phone, callee_phone, direction, status
- source_id, source_number_id, agent_id, queue_id
- started_at, answered_at, ended_at, duration_secs, ring_duration_secs, hold_duration_secs
- recording_url, has_audio, is_first_time_caller
- location, automation_id
- created_at

No UPDATE statements ever execute against this table. Corrections are new compensating records.

**call_annotations** (mutable, 1:1 with call_records):
- call_id (PK, FK to call_records)
- score, converted, outcome, reporting_tag
- category, appointment_set, notes
- updated_at, updated_by_id

Agent scoring, tagging, and outcome tracking touch only this table. The call processing pipeline never writes here. The annotation pipeline never writes to call_records. No lock contention between the two.

Queries that need both (e.g., CallDetailPanel, filtered reports) use a `LEFT JOIN` on call_id. This is a PK-to-PK join — effectively free.

### 5.2 Event Bus for Post-Call Processing

No transaction spans multiple entity types. The call completion flow is:

```
Call completes
  └─ TX1: INSERT call_records → COMMIT (< 1ms)
  └─ Event: "call.completed" → in-process channel / Redis stream
       ├─ TX2: INSERT call_flow_events (batch) → COMMIT
       ├─ TX3: INSERT call_visitor_session → COMMIT
       ├─ TX4: Trigger evaluation → INSERT call_tags → COMMIT
       ├─ TX5: INSERT webhook_deliveries → COMMIT
       ├─ TX6: Queue ASR transcription job → async
       └─ TX7: UPDATE call_daily_summary (increment) → COMMIT
```

Each transaction is independent, short (< 10ms), and retriable. No cascading locks. No deadly embrace.

The event bus can be:
- **In-process tokio channel** for single-instance deployments
- **Redis Streams** for multi-instance deployments
- **PostgreSQL LISTEN/NOTIFY** as a lightweight alternative

### 5.3 Connection Pool Segregation

Four connection pools prevent workload starvation:

| Pool | Purpose | Max Connections | Statement Timeout |
|------|---------|:---------------:|:-----------------:|
| `call_processing` | CDR inserts, routing lookups | 20 | 5 seconds |
| `api_crud` | UI/API config reads and writes | 10 | 30 seconds |
| `background` | Exports, bulk sends, aggregation, transcription | 5 | 300 seconds |
| `reports` | Dashboard and report queries | 5 | 60 seconds |

A runaway export or report query can never exhaust connections needed for call processing. Each pool has its own statement timeout — a background job gets 5 minutes, but a call-processing query that exceeds 5 seconds is killed.

### 5.4 Configuration Caching

Routing configuration is cached in application memory to eliminate DB round-trips from the call setup path:

- **Cache scope:** Queue, SmartRouter, GeoRouter, Schedule, VoiceMenu, CallSettings
- **TTL:** 10-30 seconds (configurable)
- **Invalidation:** PostgreSQL LISTEN/NOTIFY on config table changes, or Redis pub/sub
- **Fallback:** On cache miss, read from DB via `call_processing` pool
- **Consistency model:** Eventual (seconds). Config changes propagate within one TTL cycle.

Call setup reads from cache. Zero DB round-trips on the hot path.

### 5.5 Asynchronous Counter Aggregation

Denormalized counters (BlockedNumber.calls_blocked, TrackingSource.call_count, Tag.usage_count, DntEntry.rejected_count) are NOT incremented synchronously in the call path.

Instead:
1. The call/event processing pipeline writes a lightweight counter event to a buffer (Redis INCR or in-memory accumulator)
2. A background flush task periodically (every 5-30 seconds) batches accumulated increments and applies them to the parent rows in a single UPDATE per entity
3. Display values may lag by seconds — acceptable for informational counters

This eliminates hot-row serialization under concurrent load.

### 5.6 Time-Based Partitioning for Event Log Tables

All event log tables are partitioned by month on their primary timestamp column:

| Table | Partition Key | Estimated Rows/Month |
|-------|--------------|:--------------------:|
| call_records | started_at | 300K - 3M |
| call_flow_events | occurred_at | 3M - 30M |
| call_transcription_segments | created_at | 3M - 30M |
| call_ai_summaries | generated_at | 300K - 3M |
| text_messages | sent_at | 100K - 1M |
| agent_state_log | changed_at | 100K - 1M |
| webhook_deliveries | delivered_at | 50K - 500K |
| api_log_entries | timestamp | 50K - 500K |

Benefits:
- Current-month writes and historical-month reads hit different physical partitions — no I/O contention
- Old partitions can be archived or dropped for retention management without affecting active data
- Index maintenance is scoped to partition size, not total table size
- VACUUM operates per-partition

Smaller event log tables (notifications, voicemail_messages, monitoring_events, call_keyword_hits, call_visitor_sessions, form_records, chat_records, fax_records, video_records, export_records, voice_registration_history) do not need partitioning unless volume warrants it. They use standard tables with indexes on their timestamp columns.

---

## 6. Schema Pattern Summary

### 6.1 Event Log Tables (Denormalized Wide)

```
call_records (partitioned by month on started_at)
  ├── PK: id (uuid)
  ├── Immutable CDR fields (caller, callee, direction, status, timestamps, durations)
  ├── Snapshot denormalized fields (source_name, agent_name — copied at write time)
  ├── FK references (source_id, agent_id, queue_id — for joins when needed)
  └── Indexes: started_at, caller_phone, callee_phone, source_id, agent_id, status

call_annotations (unpartitioned, 1:1 with call_records)
  ├── PK/FK: call_id → call_records
  ├── Mutable fields (score, converted, outcome, notes, category)
  └── Indexes: call_id (implicit PK), outcome, score
```

### 6.2 Ephemeral State (Redis)

```
active_call:{call_id}     → Hash (caller, agent_id, queue_id, status, started_at)
presence:{identity}        → Hash (status, note, activity, current_call_id, last_updated)
location:{aor}             → Hash (destination, expires, transport) with TTL
freq:{policy}:{scope}      → String (counter) with EXPIREAT matching window_end
counter:{entity}:{id}      → String (accumulated increment) — flushed to PG periodically
```

### 6.3 Configuration Tables (3NF)

Standard normalized relational tables with:
- UUID primary keys
- Foreign key constraints with appropriate CASCADE/SET NULL
- `account_id` on every table for multi-tenant isolation
- `created_at` and `updated_at` timestamps
- Optimistic concurrency via `updated_at` comparison (not row locks)
- Polymorphic routing: `destination_type` (text) + `destination_id` (uuid) pattern where routing targets vary by entity type

### 6.4 Compliance Tables (3NF + Audit)

Same as configuration tables, plus:
- `encrypted_text` columns for sensitive fields (EIN, credentials, API keys)
- History/audit trail tables where required (voice_registration_history pattern)
- Soft delete via `deleted_at` timestamp (compliance data must not be hard-deleted within retention period)

### 6.5 Analytical Tables (Star Schema)

```
call_daily_summary (fact table)
  ├── Dimensions: summary_date, account_id, source_id, agent_id, queue_id
  ├── Measures: total_calls, answered_calls, missed_calls, voicemail_calls,
  │             total_duration_secs, avg_duration_secs, unique_callers,
  │             first_time_callers, converted_calls, appointments_set
  ├── UQ: (account_id, summary_date, source_id, agent_id, queue_id)
  └── Populated by: background job or REFRESH MATERIALIZED VIEW CONCURRENTLY
```

Dimension tables are the configuration entities themselves (tracking_sources, users, queues). No separate dimension tables needed — the star schema shares dimensions with the OLTP layer.

### 6.6 Object Storage

```
recordings/{account_id}/{year}/{month}/{call_id}.wav
voicemail/{account_id}/{mailbox_id}/{message_id}.wav
fax/{account_id}/{fax_id}.pdf
exports/{account_id}/{export_id}.csv
knowledge/{account_id}/{bank_id}/{document_id}.{ext}
greetings/{account_id}/{entity_type}/{entity_id}.wav
```

Presigned URLs for browser-direct access. Lifecycle policies for hot → warm → cold tiering.

### 6.7 Vector Storage (pgvector)

```
knowledge_bank_embeddings
  ├── id: uuid (FK to knowledge_bank_documents)
  ├── chunk_index: integer
  ├── chunk_text: text
  ├── embedding: vector(1536)  — or vector(768) depending on model
  └── Index: IVFFlat or HNSW on embedding column
```

Co-located in PostgreSQL via pgvector extension. Queried via cosine similarity for RAG retrieval. If volume exceeds pgvector performance limits (> ~5M vectors), extract to a dedicated vector DB (Qdrant, Weaviate).

---

## 7. Concurrency Guarantees

The design eliminates every identified contention scenario:

| Scenario | Risk | Mitigation | Guarantee |
|----------|------|------------|-----------|
| CDR writes vs. report reads | I/O contention on shared table | Time-based partitioning separates current writes from historical reads | Writers and readers hit different partitions |
| Annotation updates vs. CDR inserts | Row-level lock contention | Split into call_records (immutable) + call_annotations (mutable) | Different tables, different lock spaces |
| Counter increments (hot rows) | Serialized row locks under high concurrency | Async counter aggregation via Redis buffer + periodic flush | No synchronous row locks in call path |
| Config reads during call setup | Read blocked by admin UPDATE | Config caching in application memory with pub/sub invalidation | Zero DB access on call setup hot path |
| Bulk operations starving real-time | Connection/IO exhaustion | 4-pool connection segregation with per-pool limits and timeouts | Call processing pool is never exhausted |
| Analytical aggregation blocking writes | Lock held during materialized view refresh | CONCURRENTLY refresh or staging table swap | No read locks during refresh |
| Automation cascade deadlocks | Multi-entity transaction holding locks | Event bus: one entity per transaction, post-commit dispatch | No transaction spans multiple entity types |

### 7.1 Transaction Discipline

- **Maximum transaction scope:** One entity type per transaction
- **Maximum transaction duration:** < 10ms for call processing, < 100ms for API CRUD
- **Isolation level:** READ COMMITTED (PostgreSQL default)
- **Locking strategy:** Optimistic concurrency via `updated_at` for config; no explicit row locks
- **Retry policy:** Serialization failures retry with exponential backoff (max 3 attempts)

### 7.2 Failure Modes

| Failure | Impact | Recovery |
|---------|--------|----------|
| PostgreSQL down | Call processing uses cached config; CDRs buffer in memory | Flush buffered CDRs on reconnection |
| Redis down | Active call state lost; presence stale | Rebuild from PostgreSQL call_records (in-progress) and SIP registrations |
| Event bus backlog | Post-call processing delayed (tags, webhooks, aggregation) | Events are durable; process on drain. CDR is already persisted. |
| Background pool exhausted | Exports and aggregation queued | Auto-recovery when connections return. Call processing unaffected. |

---

## 8. Migration Path: A → B

If analytical query performance degrades at scale (> 10M CDRs, > 50 concurrent report users), the design supports extraction without application rewrites:

**Step 1:** Add a read replica. Route `reports` pool to the replica.

**Step 2:** Replace materialized views with a ClickHouse/TimescaleDB analytical sidecar. CDC (Change Data Capture) via PostgreSQL logical replication streams event log inserts to the analytical engine.

**Step 3:** If pgvector performance degrades (> 5M vectors), extract knowledge bank embeddings to Qdrant or Weaviate. The KnowledgeBankDocument entity already has embedding_status lifecycle management — the vector store is behind an abstraction.

Each step is a configuration/infrastructure change. No application code rewrites. The pool segregation and event bus boundaries make the extraction points clean.

---

## 9. Relationship to Existing RustPBX Schema

The existing RustPBX tables (14 entities in SeaORM) map to the new schema as follows:

| Existing Table | Decision | Rationale |
|---------------|----------|-----------|
| rustpbx_call_records | **Extend** → becomes call_records (immutable) + call_annotations (new) | Add source attribution, snapshot fields; split mutable annotations |
| rustpbx_users | **Extend** → becomes users | Add role, initials, avatar_color, account_id |
| rustpbx_extensions | **Keep** | SIP-level entity; not directly exposed in 4iiz UI |
| rustpbx_departments | **Keep** | Organizational grouping; may map to account hierarchy |
| rustpbx_extension_departments | **Keep** | M:M junction |
| rustpbx_sip_trunks | **Keep** | SIP infrastructure; TrackingNumber is a layer above |
| rustpbx_routes | **Keep + Add** | Keep for SIP routing; add SmartRouter/GeoRouter for marketing routing |
| rustpbx_queues | **Extend** → becomes queues | Add strategy, schedule, agent assignment, real-time metrics support |
| rustpbx_frequency_limits | **Move to Redis** | Ephemeral counters belong in Redis, not relational DB |
| rustpbx_voicemails | **Extend** → becomes voicemail_messages | Add call_id linkage, richer metadata |
| rustpbx_voicemail_greetings | **Merge** into voicemail_boxes | Greeting config is part of mailbox config |
| rustpbx_monitoring_events | **Keep** | Direct match |
| rustpbx_locations | **Move to Redis** | Ephemeral SIP registrations with TTL |
| presence_states | **Move to Redis** | Real-time mutable state |

New tables (~80) are created for all entities not covered by existing RustPBX tables.
