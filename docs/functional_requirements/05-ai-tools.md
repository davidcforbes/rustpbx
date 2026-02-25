# 05 - AI Tools Section

> 5 pages | 4 well-defined, 1 stub
> Source: `ui/src/sections/ai_tools.rs`

## Side Navigation

Two groups:
- **AI Insights:** AskAI, Summaries
- **AI Agents:** ChatAI (BETA), VoiceAI, Knowledge Banks (BETA)

---

## 5.1 AskAI Page

- **Route:** `/ai-tools/askai`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, "Save" button
- Trigger card:
  - AskAI Preset select (Custom Question / Summarize Call / Sentiment Analysis / Lead Scoring / Compliance Check)
  - Tracking Number Assignment select
  - Delay select (Immediately / After 1 min / After 5 min)
- Workflows card: "Add Workflow" button, empty state "No workflows configured"

### Actions
Save, select preset, assign tracking numbers, configure delay, add workflow

### Implied Functionality
AI-powered post-call analysis with preset templates, per-tracking-number configuration, delay-based triggering

---

## 5.2 Summaries Page

- **Route:** `/ai-tools/summaries`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, "Save" button
- Channels card: Phone toggle, Video toggle
- Summary Types card (9 checkboxes):
  - Classic Summary, Customer Success Summary, Key Insights, Action Items, Sentiment Analysis, Lead Qualification, Compliance Review, Topic Classification, Custom Template
- Transcription card:
  - "Transcribe all calls" toggle
  - Transcription Rules textarea (e.g., "Redact SSN and credit card numbers")

### Actions
Save, toggle channels, select summary types, configure transcription rules

### Implied Functionality
Automated call summarization with multiple AI-generated summary formats, channel filtering, transcription with PII redaction rules

---

## 5.3 Knowledge Banks Page

- **Route:** `/ai-tools/knowledge-banks`
- **Status:** STUB ONLY

### UI Elements
- Header with breadcrumbs, search input, "New Knowledge Bank" button
- Data table headers only: Name, Documents (count), Category, Last Import, Updated, Created, Actions
- Empty table body (no mock data)

### Actions
"New Knowledge Bank" button (not wired), search

### Implied Functionality
Document/knowledge base management for AI agent grounding (RAG)

> **NEEDS DEFINITION:**
> - Document upload flow (file types: PDF, DOCX, TXT, URLs?)
> - Category taxonomy management
> - Import mechanism (bulk upload, URL scraping, API sync?)
> - Document viewer / content preview
> - Embedding/indexing status tracking
> - Integration with ChatAI and VoiceAI agents

---

## 5.4 VoiceAI Page

- **Route:** `/ai-tools/voiceai`
- **Status:** Well-defined

### UI Elements
- Header with breadcrumbs, "Save" button
- General card: Name input, Description textarea
- Welcome Message card: textarea for initial greeting
- Instructions card: textarea (6 rows) for AI behavior instructions
- Personality card: Voice selection with 6 radio options:
  - Allison, Aria, Davis, Emily, Guy, Jenny
  - Each with "Preview" play button

### Actions
Save, select voice, preview voice samples, configure instructions

### Implied Functionality
AI voice agent creation with personality selection, customizable behavior via natural language instructions, welcome message configuration

---

## 5.5 ChatAI Page

- **Route:** `/ai-tools/chatai`
- **Status:** Well-defined (BETA)

### UI Elements
- Header with breadcrumbs (marked BETA), "Save" button
- General card: Name input, Description textarea
- Knowledge Bank card: Select dropdown (General Knowledge / Product FAQ / Support Docs)
- Instructions card: textarea (6 rows) for AI behavior instructions

### Actions
Save, select knowledge bank, configure instructions

### Implied Functionality
AI chat agent configuration with knowledge base grounding and customizable behavior via natural language instructions
