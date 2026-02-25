# 4iiz AI Tools Section - UI Analysis & Prototype Plan

## Section Overview

The **AI Tools** section is accessible from the left icon navigation bar (sparkle/star icon). It provides AI-powered analytics, conversational agents, and knowledge management capabilities. The section is divided into two subsection groups: **AI Insights** (analysis and reporting tools) and **AI Agents** (interactive AI capabilities). This section represents 4iiz's integration with LLM-powered services (ChatGPT) for call analysis, summarization, and customer-facing voice/chat AI agents.

---

## Page Inventory (4 pages, 4 screenshots + 1 sidebar-only)

| # | Page | Screenshot | Layout Type | Key Action |
|---|------|-----------|-------------|------------|
| 1 | AskAI | `AITools - AI Insigts - AskAI.jpg` | Form + workflow builder | Save Changes, Add Workflow |
| 2 | Summaries | `AITools - AI Insigts - Summaries.jpg` | Settings cards with toggles | Save Changes (per section) |
| 3 | Knowledge Banks | `AITools - AI Insigts - Knowledge Banks.jpg` | Empty data table | New Knowledge Bank |
| 4 | VoiceAI | `AITools - AI Insigts - VoiceAI.jpg` | Multi-section form | Save Changes |
| 5 | ChatAI (BETA) | No screenshot (sidebar only) | Unknown | N/A |

---

## Sidebar Navigation Structure

The secondary navigation panel has two subsection groups, each with a category icon and header:

### AI INSIGHTS (blue diamond icon)
- **AskAI** - Configure AI-powered trigger questions
- **Summaries** - Conversation analysis settings

### AI AGENTS (robot/gear icon)
- **ChatAI** `BETA` - Chat-based AI agent (beta badge, gray text)
- **VoiceAI** - Voice-based AI agent for callers
- **Knowledge Bank** `BETA` - Document knowledge base (beta badge, gray text)

**Note**: BETA badges appear as small gray uppercase text next to the nav item name.

---

## Detailed Page Analysis

### Page 1: AskAI

**Breadcrumb**: Triggers > New > General (dropdown)

**Purpose**: Configure AI-powered analysis triggers that use ChatGPT to answer natural language questions about calls. Results are stored in custom fields for reporting.

**Layout**: Vertical form layout with cards, followed by a workflow builder section.

**Top bar**: "AI Tools" section label on left. Breadcrumb navigation in content area. "Info" link (with info icon) at top right.

**Info banner** (card with light background):
- Title: **AskAI** (bold)
- Description: "AskAI allows you to ask any natural language question and get a concise ChatGPT powered answer, which can then be entered into a custom field for easy reporting. Please note that at least one custom field must be created."
- Contains cyan links: "AskAI" and "custom field" link text, plus "knowledge base" link
- "See our knowledge base for more information."

**Form Card: General**:

| Field | Type | Notes |
|-------|------|-------|
| Name | Text input | Standard input field |
| Use a Preset | Dropdown button | Gray outlined button with gear icon, opens preset selector |

**Preset Dropdown** (overlay panel, visible in screenshot as open):
- Header: "Select a preset"
- Description: "These are suggested configurations. Please review and make the necessary changes according to your company needs."
- Scrollable list with 3 presets:

| Preset Name | Description |
|-------------|-------------|
| AskAI: Call Summary *Additional Rates will Apply* | Summarize your phone calls into two or three sentences *Additional Rates will Apply* (Further configuration is needed) |
| AskAI: Qualified Lead *Additional Rates will Apply* | Determine if call activity is a qualified lead *Additional Rates will Apply* (Further configuration is required) |
| AskAI: Call Outcome | Automatically tags calls with an outcome label |

**Tracking Numbers Assignment**:
- Button: "Edit Assigned Tracking Numbers" (disabled state, gray/muted with cyan text)
- Help text below: "(save first to assign)"
- Implied: Must save the trigger configuration before assigning tracking numbers

**Delay Workflow**:
- Label: "Delay workflow"
- Input: Number field (value: 0) + "seconds" dropdown selector
- Help text: "This allows you to delay the start of your workflow by the given amount of time. This is useful when you need to wait for some conditions to become true on your activities."

**Actions**:
- "Save Changes" button (cyan filled)

**Workflows Section** (separate card below form):
- Header: **Workflows** (bold) + "perform actions in response to this trigger"
- CTA: "+ Add Workflow" button (cyan filled, right-aligned)
- Secondary: "Switch to Visualization" link (gray, right of button)
- Empty state: "**No workflows added.** Click the 'Add Workflow' button above to get started."

---

### Page 2: Summaries

**Breadcrumb**: Account Settings > Conversation Analysis Setting > Channel Filter (dropdown)

**Purpose**: Configure which conversation types to analyze and select summary output formats. This is an account-level settings page for AI-powered conversation summarization.

**Top bar**: "AI Tools" section label. "Versions" link (with gear icon) at top right.

**Info Banner** (blue-outlined card at top):
- Icon: Blue info circle (i)
- Text: "Enhance your insights with AskAI Summaries, which lets you select from various topic types to tailor the summaries for maximum impact. This feature ensures you receive the most valuable information from your agents' conversations, providing clear and actionable insights."
- Links: "Close" (cyan) and "More Info" (cyan with external link icon)

**Card 1: Channel Filter**:
- Description: "What type of activities do you want to analyze? Choose which channel(s) you want to connect with AskAI Summaries below."
- Warning text (orange/red): "Toggle one of the options in order to select the Summary Type below."

| Toggle | Label | State |
|--------|-------|-------|
| Toggle 1 | Analyze phone calls | OFF |
| Toggle 2 | Analyze video calls | OFF |

- Help text: "Have you completed the Zoom Integration (link) which is required to summarize video calls?"
- Action: "Save Changes" button (cyan filled)

**Card 2: Summary Type Selector**:
- Top right link: "We Want Your Feedback!" (cyan with external link icon)
- Description: "Choose one or more summaries* to get key information from a conversation, based on your specific needs or interests. You can integrate these summaries into your call log, upload them to Salesforce, or have them emailed to you for easy access and review."
- Warning note (orange/red): "*Note: Summaries carry an additional fee per summarized activity."

**Summary Types Grid** (2-column layout, 9 total toggles, all OFF):

| Left Column | Description | Right Column | Description |
|-------------|-------------|--------------|-------------|
| Classic Summary | Narrative recap of the meeting from beginning to end | Customer Success | Experiences, challenges, and goals discussed on the call |
| Key Insights | Core conversation takeaways and insights | Project Kick-Off | Project vision, target goals, and resources |
| Question-Answer | Questions asked and their answers | Action Items | List of next steps based on the conversation |
| Sales | The prospect's needs, challenges, and buying journey | Pain Points | Issues the prospect is facing and wants to resolve |
| Demo | Demo overview and success rating | | |

- Action: "Save Changes" button (cyan filled)

**Card 3: Summary Transcription Rules** (partially visible):
- Header: **Summary Transcription Rules** (bold) + "Select which call settings you'd like to use to run AskAI summaries."
- Description: "To analyze your conversations, please enable call transcriptions and select which activity types you'd like analyzed. If your selected call setting does not already have call transcriptions turned on, you can do so by clicking the warning icon."
- Action: "Edit Assigned Call Settings" button (cyan filled)

---

### Page 3: Knowledge Banks

**Purpose**: Manage document knowledge bases used by ChatAI and VoiceAI agents to provide informed, context-aware responses.

**Top bar**: Title "Knowledge Banks" (bold) + subtitle "Knowledge Banks for ChatAI and VoiceAI" (gray). "New Knowledge Bank" CTA button (cyan filled) and "Info" link (with icon) at top right.

**Layout**: Data table with search, currently empty.

**Search**: Standard search input field with clear (x) icon and search (magnifying glass) icon.

**Table Columns**:

| Column | Type | Sortable | Notes |
|--------|------|----------|-------|
| Name | Text | Yes | Sort arrows visible |
| Documents | Number | Yes | Count of uploaded documents |
| Category | Text | Yes | Classification/grouping |
| Last Import | Datetime | Yes | When documents were last imported |
| Updated | Datetime | Yes | Last modification time |
| Created | Datetime | Yes | Creation time |

**Pagination**: "Per page: 10" selector (bottom right). No page numbers shown (empty state).

**Empty state**: No rows displayed, no explicit empty-state message.

---

### Page 4: VoiceAI

**Breadcrumb**: VoiceAI Agents > New > Name (dropdown)

**Purpose**: Create and configure AI-powered voice agents that handle inbound calls, with customizable personality, welcome messages, and behavioral instructions.

**Top bar**: "AI Tools" section label. "Info" and "Feedback" links at top right.

**Card 1: Name**:

| Field | Type | Default Value | Help Text |
|-------|------|---------------|-----------|
| Name your AI | Text input + red speech bubble icon | (empty) | "A reference for your AI Agent" |
| Description (optional) | Text input | (empty) | "Additional details about your AI Agent to help with documentation or if you have multiple agents" |
| Welcome message for callers | Text input | "Thank you for contacting us. How can I help you?" | "Your AI Agent will repeat this message when a customer first connects" |
| Instructions: | Large textarea (resizable) | (empty) | Italic prompt: "Describe how VoiceAI should engage with callers, such as tone, behavior, and expected outcomes." + "Learn more about prompt engineering." link (cyan, external) |

- Action: "Save Changes" button (cyan filled)

**Card 2: Personality** (partially visible):
- Header: **Personality** (bold)
- Subsection: **Select a Voice** (bold)
- Description: "Preview and choose from a variety of voice types."
- Note: "Emotion Aware AI uses a curated set of voices optimized for real-time emotion detection."
- (Card is cut off at bottom of screenshot - additional voice selection UI likely below)

---

### Page 5: ChatAI (BETA)

**No screenshot available.** This page is visible in the sidebar navigation under AI AGENTS with a "BETA" badge. Based on the sidebar placement alongside VoiceAI and its beta status, it likely provides a text-based chat AI agent configuration interface, potentially with similar fields to VoiceAI (name, instructions, knowledge bank assignment) but oriented toward web chat or SMS interactions rather than voice calls.

---

## Shared UI Patterns

### Pattern: Form-Based Settings Pages
Pages 1 (AskAI), 2 (Summaries), and 4 (VoiceAI) all use a vertical card-based form layout:
1. Breadcrumb navigation at top of content area
2. One or more card sections with white background, light border, rounded corners
3. Form fields within cards (text inputs, textareas, toggles)
4. "Save Changes" cyan button at bottom of each card
5. Help text below each field in gray

### Pattern: Card Sections
All form pages use distinct card containers:
- White background with subtle border/shadow
- Bold section title at top of card
- Descriptive text below title
- Form fields with labels above
- Save button at card bottom

### Pattern: Breadcrumb Navigation
Three pages show breadcrumb trails:
- AskAI: `Triggers > New > General` (dropdown on last segment)
- Summaries: `Account Settings > Conversation Analysis Setting > Channel Filter` (dropdown on last segment)
- VoiceAI: `VoiceAI Agents > New > Name` (dropdown on last segment)

The last breadcrumb segment has a dropdown chevron, suggesting step/section navigation within the form.

### Pattern: Info/Help Links
- "Info" link with circle-i icon (AskAI, Knowledge Banks)
- "Versions" link with gear icon (Summaries)
- "Feedback" link (VoiceAI)
- These appear at the top-right of the content area

### Pattern: BETA Badges
- Small gray uppercase "BETA" text
- Appears next to nav item name in sidebar
- Applied to ChatAI and Knowledge Bank

### Pattern: Toggle Switches
Used on Summaries page:
- DaisyUI-style toggle with "OFF" label
- Gray when off, cyan when on (implied)
- Label text to the right of toggle

### Pattern: Disabled/Gated Controls
- "Edit Assigned Tracking Numbers" button on AskAI is disabled with "(save first to assign)" message
- Demonstrates progressive disclosure: save first, then configure dependent settings

---

## Component Mapping to DaisyUI

| 4iiz Component | DaisyUI Equivalent |
|----------------|-------------------|
| Form card | `card` with `card-body` |
| Text input | `input input-bordered` |
| Textarea | `textarea textarea-bordered` |
| Toggle switch | `toggle` with label |
| Dropdown button | `dropdown` + `btn btn-outline` |
| Preset list | `dropdown-content menu` with scrollable items |
| CTA button | `btn bg-iiz-cyan text-white` |
| Save button | `btn bg-iiz-cyan text-white` |
| Disabled button | `btn btn-disabled` or `btn` with opacity |
| Breadcrumb | `breadcrumbs text-sm` |
| Info banner | `alert` with blue border/outline |
| Data table | `table` with sortable headers |
| Search bar | `input input-bordered` + search icon |
| Per-page dropdown | `select select-sm select-bordered` |
| BETA badge | `badge badge-ghost badge-sm` or plain styled `<span>` |
| Info link | `link` with SVG info icon |
| Section header | `text-xs font-semibold uppercase tracking-wider` |
| Help text | `text-sm text-gray-500` below field |
| Warning text | `text-sm text-orange-600` or `text-red-500` |

---

## Prototype

**File**: `.UI-AITools/prototype/aitools.html`

Covers all 4 pages with Alpine.js state switching (`aiPage`):
- `askai` - AskAI trigger configuration with preset dropdown
- `summaries` - Conversation analysis settings with channel filter and summary type grid
- `knowledgebanks` - Knowledge Banks data table (empty state)
- `voiceai` - VoiceAI agent creation form with personality section

ChatAI is listed in sidebar navigation with BETA badge but navigates to a placeholder since no screenshot is available.

---

## Observations for Production Implementation

1. **LLM Integration Costs**: Multiple UI elements warn about "Additional Rates" for AI features. The Summaries page notes "*Summaries carry an additional fee per summarized activity.*" This suggests a metered billing model for AI-powered features, requiring usage tracking and billing integration.

2. **Preset System**: The AskAI preset dropdown provides pre-configured templates that populate form fields. This pattern could be extended to VoiceAI (voice agent templates for different industries/use cases).

3. **Workflow Builder**: The AskAI page has a "Workflows" section with "+ Add Workflow" and "Switch to Visualization" options, suggesting a visual workflow builder (likely a node-based editor) for chaining AI analysis actions with other triggers.

4. **Knowledge Bank Dependency**: Knowledge Banks serve both ChatAI and VoiceAI, meaning the KB management interface is a shared foundation that must be built before the agents can function with custom knowledge.

5. **Progressive Disclosure**: The "save first to assign" pattern on AskAI tracking numbers suggests a two-step creation flow: create the trigger first, then configure its associations. This prevents orphaned configurations.

6. **Voice Selection UI**: The VoiceAI "Personality" card mentions "Emotion Aware AI" with "a curated set of voices optimized for real-time emotion detection," suggesting integration with an emotion-aware TTS service (possibly ElevenLabs or a similar provider).

7. **Zoom Integration**: The Summaries page references Zoom Integration for video call analysis, indicating 4iiz has a Zoom app/OAuth integration for pulling video call transcripts.

8. **Summary Types as Product Differentiators**: The 9 summary types (Classic, Key Insights, Q&A, Sales, Demo, Customer Success, Project Kick-Off, Action Items, Pain Points) are tailored to specific business workflows, suggesting these were designed around customer research into common call analysis needs.

9. **Transcription Dependency**: Summary Transcription Rules indicates that summaries require call transcription to be enabled first. This creates a dependency chain: Recording -> Transcription -> Summarization.

10. **ChatAI as Early-Stage Feature**: The BETA badge on ChatAI suggests it is a newer, less mature feature compared to VoiceAI and AskAI. No screenshot was available, which may indicate limited functionality or restricted availability.
