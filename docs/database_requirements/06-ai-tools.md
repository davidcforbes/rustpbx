# 06 — AI Tools

## Overview

This domain covers the configuration layer for all AI-powered capabilities in the 4iiz platform. It encompasses ask-AI call analysis configurations, account-level summarization settings, knowledge bank management (documents used as RAG context for agents), autonomous voice AI agents, autonomous chat AI agents, chat AI assistance configurations with CRM integration, and Dialogflow NLU integration configurations.

A critical architectural distinction governs this entire shard: these entities are **configuration** records only. They define what AI analysis to run, which documents to index, how agents should behave, and which models to invoke. The **outputs** of that AI processing — call summaries, transcripts, sentiment scores, keyword hits, lead scores — are stored as results attached to Communication Records and are documented in shard 01. The entities here answer "how is the AI configured?" while shard 01 entities answer "what did the AI produce?"

This domain also covers the infrastructure for Retrieval Augmented Generation (RAG): KnowledgeBank and KnowledgeBankDocument track document metadata and indexing lifecycle, but the actual vector embeddings live in an external vector store, not in the relational database.

---

### AskAIConfig

**UI References:** AI Tools > Ask AI page

**Relationships:**
- Belongs to one Account (many-to-one)
- Optionally scoped to one TrackingNumber (many-to-one, nullable)
- References zero or more Workflows via workflow_ids json array (many-to-many, denormalized)

**Notes:** The delay attribute enables deferred batch processing of AI analysis after call completion, which reduces per-call latency and may be more cost-effective than real-time synchronous analysis. When tracking_number_id is null, the configuration applies to all calls on the account. The workflow_ids array holds references to Workflow records that should be triggered after the AI analysis completes and results are available, allowing downstream automation to act on AI-generated insights.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable label for this analysis configuration |
| preset | enum(Custom Question, Summarize Call, Sentiment Analysis, Lead Scoring, Compliance Check) | NN | Analysis type; determines the AI prompt template used unless overridden by custom_prompt |
| custom_prompt | long_text | | Free-form question or instruction sent to the AI model; required and authoritative when preset = Custom Question, optional override for other presets |
| tracking_number_id | uuid | FK(TrackingNumber) | Scopes analysis to calls received on a specific tracking number; null means apply to all calls on the account |
| delay | enum(Immediately, After 1 min, After 5 min, After 30 min) | NN | Processing delay after call completion before AI analysis is enqueued; supports batch cost optimization |
| output_action | short_text | MAX(120) | Describes what to do with analysis results, e.g., "Apply Tag", "Send Webhook", "Store Only" |
| workflow_ids | json | | Array of Workflow UUIDs to trigger after AI analysis completes and results are stored |
| is_active | boolean | NN | When false, this configuration is disabled and calls will not be analyzed under it |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### SummaryConfig

**UI References:** AI Tools > Summaries page

**Relationships:**
- Belongs to one Account (one-to-one, enforced by UQ on account_id)
- Governs generation of CallAISummary records (one-to-many, indirect — see shard 01)

**Notes:** This is a singleton per account; the UQ constraint on account_id enforces one configuration row per account. The enabled_summary_types array is the authoritative list of which summary format types the system will generate for each newly completed call. Actual summary content is stored in CallAISummary (shard 01), not here. The pii_redaction_rules field is a free-text instruction string passed to the AI model to guide PII handling behavior (e.g., "replace phone numbers with [PHONE], replace SSNs with [SSN]"). The default_model field allows accounts to prefer a specific AI model (e.g., "gpt-4o", "claude-3-5-sonnet") over the platform default.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN, UQ | Owning account; unique enforces singleton per account |
| phone_enabled | boolean | NN | When true, AI summarization is run on completed phone calls |
| video_enabled | boolean | NN | When true, AI summarization is run on completed video calls |
| chat_enabled | boolean | NN | When true, AI summarization is run on completed chat sessions |
| enabled_summary_types | json | NN | Array of summary type identifiers to generate; valid values: Classic, Customer Success, Key Insights, Action Items, Sentiment Analysis, Lead Qualification, Compliance Review, Topic Classification, Custom |
| transcribe_all | boolean | NN | When true, every call is transcribed regardless of whether a summary type requires it |
| transcription_language | short_text | MAX(20) | BCP-47 language tag for transcription, e.g., "en-US", "es-MX"; defaults to "en-US" |
| pii_redaction_enabled | boolean | NN | When true, PII redaction instructions are applied during transcription and summarization |
| pii_redaction_rules | text | | Free-text instructions for the AI model describing which PII types to redact and how to represent them in output |
| default_model | short_text | MAX(80) | Preferred AI model identifier for summary generation; overrides platform default when set |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### KnowledgeBank

**UI References:** AI Tools > Knowledge Banks page

**Relationships:**
- Belongs to one Account (many-to-one)
- Has many KnowledgeBankDocuments (one-to-many, cascade delete)
- Referenced by VoiceAIAgent (many-to-one, optional)
- Referenced by ChatAIAgent (many-to-one, optional)
- Referenced by ChatAIConfig (many-to-one, optional)

**Notes:** A KnowledgeBank functions as a named collection of documents that AI agents can query via Retrieval Augmented Generation (RAG). The document_count and total_size_bytes fields are denormalized counters maintained by triggers or application logic when KnowledgeBankDocument records are inserted or deleted. The status field reflects the overall indexing health of the bank: Ready means all documents are indexed, Indexing means at least one document is currently being processed, Error means at least one document failed indexing. The used_by field is a human-readable description of which agents or flows reference this bank; it is informational only and not a foreign key constraint.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable name for this knowledge bank |
| description | text | | Optional description of the bank's content and purpose |
| category | enum(General, Support, Legal, Training, Product, Custom) | NN | Content category used for organizational grouping and UI filtering |
| document_count | counter | | Denormalized count of KnowledgeBankDocument records belonging to this bank |
| total_size_bytes | integer | | Denormalized sum of file_size_bytes across all documents in this bank |
| status | enum(Ready, Indexing, Error) | NN | Overall indexing health of the bank; reflects aggregate document embedding status |
| last_import_at | timestamp_tz | | Timestamp of the most recent document import or bulk upload operation |
| used_by | short_text | MAX(255) | Human-readable description of which AI agents or flows reference this bank |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### KnowledgeBankDocument

**UI References:** AI Tools > Knowledge Banks page (document management panel)

**Relationships:**
- Belongs to one KnowledgeBank (many-to-one, cascade delete)

**Notes:** This entity tracks document metadata and the indexing lifecycle only. The actual vector embeddings produced during indexing are stored externally in a vector store (e.g., pgvector extension, Qdrant, or Pinecone) and are referenced by the document's id. The content_hash field (SHA-256) enables deduplication: before indexing a new upload, the system checks whether a document with the same hash already exists in the bank. When file_type = URL, the source_url field is the canonical document source and file_ref may be null (content is fetched at index time). When file_type is a file format, file_ref holds the object store path and source_url may optionally record the origin URL. The error_message field should be populated whenever embedding_status transitions to Failed to enable user-visible diagnostics.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key; also used as the external document reference key in the vector store |
| bank_id | uuid | FK(KnowledgeBank), NN | Owning knowledge bank |
| filename | short_text | NN, MAX(255) | Display name of the document; for URL imports, may be derived from the page title or URL path |
| file_type | enum(PDF, DOCX, TXT, HTML, URL, CSV) | NN | Format of the source document; determines parsing strategy during indexing |
| source_url | url | | Original URL if the document was imported from the web or if file_type = URL |
| file_ref | file_ref | | Object store path for the uploaded binary file; null for URL-type documents |
| content_hash | short_text | MAX(64) | SHA-256 hash of document content; used for deduplication within the bank |
| file_size_bytes | integer | NN | Size of the source document in bytes |
| page_count | integer | | Number of pages for paginated formats (PDF, DOCX); null for plain text or HTML |
| chunk_count | integer | | Number of text chunks produced after content splitting; updated when embedding completes |
| embedding_status | enum(Pending, Processing, Indexed, Failed) | NN | Current lifecycle state of vector embedding for this document |
| embedding_model | short_text | MAX(80) | Identifier of the model used to generate embeddings, e.g., "text-embedding-3-small" |
| error_message | text | | Human-readable error details populated when embedding_status = Failed |
| indexed_at | timestamp_tz | | Timestamp when embedding completed and document became queryable |
| created_at | timestamp_tz | NN | Record creation time |

---

### VoiceAIAgent

**UI References:** AI Tools > Voice AI page (stub)

**Relationships:**
- Belongs to one Account (many-to-one)
- References one KnowledgeBank for RAG context (many-to-one, optional)
- handoff_destination_id references a Queue, Agent, or TrackingNumber depending on handoff_destination_type (polymorphic, optional)

**Notes:** A VoiceAIAgent handles inbound phone calls autonomously using a text-to-speech voice and a large language model. The instructions field is the system prompt that governs agent behavior, persona, and constraints. The knowledge_bank_id links to a KnowledgeBank whose indexed documents are searched at each conversational turn to augment the AI response with relevant context. The max_turns field acts as a safety ceiling: if the conversation reaches the turn limit without resolution, the call is automatically routed to the handoff destination. The handoff_threshold controls how aggressively the agent routes to a human when it detects low confidence or user frustration; Low means the agent routes readily, High means it attempts to handle more cases autonomously. The handoff_destination_type and handoff_destination_id pair form a polymorphic foreign key that must resolve to a valid entity of the named type.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable name for this voice agent |
| description | text | | Optional description of this agent's purpose or deployment context |
| welcome_message | long_text | | Greeting text spoken via TTS when a call connects to this agent |
| instructions | long_text | NN | System prompt defining agent persona, behavioral rules, and task constraints |
| voice | enum(Allison, Aria, Davis, Emily, Guy, Jenny) | NN | TTS voice selection for spoken responses |
| language | short_text | MAX(20) | BCP-47 language tag for speech recognition and synthesis, e.g., "en-US" |
| knowledge_bank_id | uuid | FK(KnowledgeBank) | Optional RAG context source; when set, relevant document chunks are retrieved and injected at each turn |
| max_turns | integer | | Maximum number of conversational turns before automatic handoff is triggered |
| handoff_threshold | enum(Low, Medium, High) | | Confidence threshold at which the agent routes to a human; Low routes more readily, High routes less readily |
| handoff_destination_type | short_text | MAX(40) | Discriminator for polymorphic handoff target: Queue, Agent, or Number |
| handoff_destination_id | uuid | | Foreign key to the entity identified by handoff_destination_type |
| is_active | boolean | NN | When false, this agent will not accept calls and cannot be assigned to a routing step |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### ChatAIAgent

**UI References:** AI Tools > Chat AI page (stub)

**Relationships:**
- Belongs to one Account (many-to-one)
- References one KnowledgeBank for RAG context (many-to-one, optional)
- References one Queue for human handoff (many-to-one, optional)

**Notes:** A ChatAIAgent handles web chat conversations autonomously within the 4iiz chat widget. The instructions field is the system prompt governing agent behavior. The welcome_message is the first message displayed to a visitor when the chat widget opens. The handoff_queue_id routes the conversation to a human agent queue when the AI confidence falls below the handoff_threshold or the turn limit is reached. For schema consolidation considerations, see the AI Pipeline section at the end of this document.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable name for this chat agent |
| description | text | | Optional description of this agent's purpose or deployment context |
| instructions | long_text | NN | System prompt defining agent persona, behavioral rules, and task constraints |
| knowledge_bank_id | uuid | FK(KnowledgeBank) | Optional RAG context source; when set, relevant document chunks are retrieved and injected at each turn |
| welcome_message | long_text | | Initial message shown to the visitor when the chat widget opens |
| max_turns | integer | | Maximum number of chat turns before automatic handoff to a human queue |
| handoff_threshold | enum(Low, Medium, High) | | Confidence threshold at which the agent routes to a human queue |
| handoff_queue_id | uuid | FK(Queue) | Target queue for human agent handoff when confidence or turn limits are reached |
| is_active | boolean | NN | When false, this agent is disabled and will not be assigned to chat sessions |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### ChatAIConfig

**UI References:** Flows > Chat AI page

**Relationships:**
- Belongs to one Account (many-to-one)
- References one KnowledgeBank for RAG context (many-to-one, optional)

**Notes:** ChatAIConfig configures AI-assisted chat behavior within the broader Flows section of the platform, with a primary differentiator being CRM integration support. While ChatAIAgent (AI Tools section) is a standalone autonomous chat agent, ChatAIConfig is positioned as a flow component that can coordinate with Salesforce, HubSpot, Zoho, or other CRM systems to enrich conversations with customer context and push outcomes back to the CRM. The crm_config json field holds connection credentials, field mappings, and sync behavior settings specific to the selected crm_type. See the AI Pipeline section for consolidation considerations.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable label for this chat AI configuration |
| description | text | | Optional description of intended use and deployment context |
| knowledge_bank_id | uuid | FK(KnowledgeBank) | Optional RAG context source for grounding AI responses |
| instructions | long_text | | System prompt or behavioral guidance for the AI model |
| max_turns | integer | | Maximum number of chat turns before automatic handoff |
| handoff_threshold | enum(Low, Medium, High) | | Confidence threshold at which the AI routes to a human |
| crm_integration_enabled | boolean | NN | When true, the CRM integration is active for this configuration |
| crm_type | short_text | MAX(60) | CRM platform identifier, e.g., "Salesforce", "HubSpot", "Zoho" |
| crm_config | json | | CRM-specific connection parameters, OAuth tokens, and field mapping rules; encrypted at rest |
| is_active | boolean | NN | When false, this configuration is disabled and will not be applied to chat sessions |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### DialogflowConfig

**UI References:** Flows > Dialogflow page (stub)

**Relationships:**
- Belongs to one Account (many-to-one)

**Notes:** DialogflowConfig stores the credentials and behavioral settings required to connect the 4iiz platform to a Google Cloud Dialogflow NLU project. The service_account_json field contains a GCP service account key in JSON format and MUST be encrypted at rest using application-level encryption before storage. API responses must never return the full service_account_json value; instead, return a masked boolean indicator confirming whether credentials are configured. The connection_status field reflects the result of the most recent connectivity test against the Dialogflow API and should be updated to Connected or Failed by the test action, and reset to Untested when credentials are changed. The default_intent field specifies the Dialogflow intent to invoke when no user speech maps to a recognized intent; the fallback_message is returned to the caller or chat participant in that case.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | NN, MAX(120) | Human-readable label for this Dialogflow integration |
| project_id | short_text | NN, MAX(120) | Google Cloud project ID where the Dialogflow agent is deployed |
| service_account_json | encrypted_text | NN | GCP service account key (JSON format) encrypted at rest; never returned in full via API |
| language | short_text | NN, MAX(20) | BCP-47 language tag for NLU processing, e.g., "en-US" |
| default_intent | short_text | MAX(120) | Dialogflow intent name to invoke when no recognized intent matches user input |
| fallback_message | long_text | | Message delivered to the user when Dialogflow returns no matching intent and no default_intent is configured |
| connection_status | enum(Connected, Failed, Untested) | NN | Result of the most recent API connectivity test; reset to Untested when credentials change |
| last_tested_at | timestamp_tz | | Timestamp of the most recent connectivity test attempt |
| is_active | boolean | NN | When false, Dialogflow NLU is not invoked for calls or chats that reference this configuration |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

## AI Pipeline & Vector Storage Considerations

### Separation of AI Configuration (Shard 06) and AI Outputs (Shard 01)

The entities in this shard are exclusively configuration records. They answer design-time questions: What analysis should be run? What prompt should be used? Which voice should the agent speak in? Which documents form the agent's knowledge? None of these entities store the results of AI processing.

All AI-generated outputs — call summaries, transcripts, sentiment scores, lead scores, keyword detection results, compliance flags — are stored as result records attached to call and communication entities in shard 01 (Communication Records). The link from output back to configuration is typically a nullable foreign key on the output record (e.g., a CallAISummary may reference the SummaryConfig that produced it, but the absence of that reference does not invalidate the summary). This separation ensures that configuration changes do not invalidate historical outputs, and that output tables can be queried without joining to configuration tables.

### Vector Embedding Storage for Knowledge Banks

KnowledgeBank and KnowledgeBankDocument represent the relational metadata layer of a RAG pipeline. The actual vector embeddings — high-dimensional floating-point arrays generated by embedding models from document text chunks — are stored outside the relational database in a dedicated vector store. Candidate backends include:

- pgvector (PostgreSQL extension): keeps embeddings in the same database cluster, simplifying transactions and backups, with some query performance trade-offs at large scale
- Qdrant or Weaviate: purpose-built vector databases with optimized approximate nearest-neighbor search, appropriate for large knowledge banks or high query throughput
- Pinecone: fully managed vector store with a serverless tier suitable for variable load

Regardless of backend, the KnowledgeBankDocument.id serves as the external document identifier in the vector store, enabling the platform to resolve a retrieved chunk back to its source document metadata (filename, bank, account) for display and attribution. The embedding_status and indexed_at fields in KnowledgeBankDocument track the lifecycle and allow the platform to identify documents that need reindexing when the embedding model changes or when content is updated.

### Relationship Between SummaryConfig and CallAISummary

SummaryConfig (this shard) is the account-level policy that governs which summary types are generated. CallAISummary (shard 01) is the per-call output record storing a generated summary's text, type, and metadata.

The processing pipeline flows as follows: when a call completes, the platform reads the account's SummaryConfig to determine (a) whether the communication type is enabled, (b) which summary types are requested, and (c) which AI model and PII redaction rules to apply. For each enabled summary type, the platform enqueues an AI generation job. Each completed job writes one CallAISummary row. The SummaryConfig row is read at job dispatch time; its id may optionally be stored on CallAISummary rows to support auditing (e.g., "this summary was generated using config version X"). If the SummaryConfig is later updated, existing CallAISummary records are not retroactively altered.

### Potential Consolidation of ChatAIAgent and ChatAIConfig

ChatAIAgent (AI Tools section, standalone autonomous agent) and ChatAIConfig (Flows section, CRM-integrated AI assistance) share a substantial attribute overlap: both have account_id, name, description, instructions, knowledge_bank_id, max_turns, handoff_threshold, and is_active. The primary differentiators are:

- ChatAIAgent adds: welcome_message, voice-adjacent fields, and a Queue foreign key for handoff routing
- ChatAIConfig adds: crm_integration_enabled, crm_type, and crm_config for CRM system connectivity

During schema design, consider whether these can be unified into a single AIAgentConfig table with a mode or agent_type discriminator (e.g., Standalone, CRMAssisted), with nullable columns for the mode-specific fields. The benefit is a single table to query and maintain. The risk is that the semantics of the two use cases diverge further as the platform evolves, making a union table harder to reason about. If kept separate, a shared abstract entity or view over common columns can reduce duplication in API and ORM layers.
