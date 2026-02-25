# 03 — Telephony & Numbers

This domain covers all entities related to phone number provisioning, marketing attribution, and call routing configuration within the 4iiz call tracking platform. It is the connective tissue between raw telephony infrastructure and the marketing intelligence layer that gives the platform its core value proposition.

Unlike a general-purpose PBX (such as RustPBX's SIP trunk model), 4iiz treats phone numbers primarily as attribution instruments. Every inbound call arrives on a TrackingNumber, which carries metadata about the marketing source that prompted the call. This source attribution feeds directly into reporting, enabling marketers to measure the ROI of individual campaigns, channels, and creatives. The routing layer (queues, smart routers, voice menus) determines where the call is delivered, but the attribution layer determines why it arrived.

The domain is organized around three functional groups. First, the number inventory itself: TrackingNumbers (what the caller dials), ReceivingNumbers (where calls ultimately land at the business), TargetNumbers (intermediate routing endpoints), and TextNumbers (SMS-capable lines). Second, the attribution scaffolding: TrackingSources define the marketing channels, NumberPools enable Dynamic Number Insertion for session-level tracking, and NumberPoolMembers manage pool membership. Third, the operational layer: PortRequests manage number portability workflows, and CallSettings profiles control per-account call handling behavior.

---

### TrackingNumber

**UI References:** Numbers > Tracking Numbers page, Numbers > Number Pools, CallDetailPanel (source_number), Reports (source attribution)

**Relationships:**
- Many-to-one with Account (each number belongs to one account)
- Many-to-one with TrackingSource (each number is assigned to one marketing source)
- Many-to-one with ReceivingNumber (forwarding destination, optional)
- Many-to-one with NumberPool (pool membership, optional — null when Offsite Static)
- Polymorphic many-to-one with routing target (Queue, VoiceMenu, SmartRouter, etc.) via routing_target_type + routing_target_id

**Notes:** Routing is polymorphic — routing_target_type and routing_target_id together point to a Queue, VoiceMenu, SmartRouter, GeoRouter, or other flow entity. A number in a DNI pool (pool_id is set) is dynamically assigned to web sessions and should not be displayed as a static number in any marketing material. Offsite Static numbers are the classical call tracking model; Onsite Dynamic numbers are managed by the NumberPool subsystem. The routing_description field is a human-readable summary computed from the routing configuration and displayed in list views without requiring a join.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| number | e164 | NN, UQ | The E.164 phone number provisioned for tracking |
| source_id | uuid | FK(TrackingSource) | Marketing source this number is attributed to |
| routing_description | short_text | MAX(255) | Human-readable summary of the routing destination (e.g., "Queue: Sales Team") |
| routing_type | enum(Queue, Smart Router, Direct, Voice Menu, Geo Router, Schedule) | NN | Discriminator describing the class of routing applied |
| routing_target_type | short_text | MAX(64) | Entity type name of the polymorphic routing destination |
| routing_target_id | uuid | | Entity ID of the polymorphic routing destination |
| text_enabled | boolean | NN | Whether SMS/MMS is enabled on this number |
| receiving_number_id | uuid | FK(ReceivingNumber) | Direct forwarding destination when routing_type is Direct |
| number_type | enum(Offsite Static, Onsite Dynamic) | NN | Static tracking number vs. DNI pool member |
| number_class | enum(Local, Toll-Free) | NN | Local geographic number or toll-free number |
| pool_id | uuid | FK(NumberPool) | Pool this number belongs to; null for Offsite Static numbers |
| billing_date | integer | MAX(31) | Day of month for billing cycle (1–31) |
| is_active | boolean | NN | Whether this number is currently active and billable |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### ReceivingNumber

**UI References:** Numbers > Receiving Numbers page, TrackingNumber configuration

**Relationships:**
- Many-to-one with Account (each receiving number belongs to one account)
- One-to-many with TrackingNumber (multiple tracking numbers may forward here)

**Notes:** tracking_count and total_calls are denormalized counters maintained for display efficiency. tracking_count is incremented or decremented by application logic when TrackingNumbers are assigned or unassigned to this receiving number. total_calls is incremented by the call processing pipeline on call completion. The number field is unique within an account, preventing duplicate registrations of the same business line.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| number | e164 | NN, UQ(account_id) | The E.164 destination number at the business |
| description | short_text | MAX(255) | Human-readable label for this business line |
| tracking_count | counter | NN | Number of TrackingNumbers currently forwarding to this destination |
| total_calls | counter | NN | Lifetime call count received on this number |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### TextNumber

**UI References:** Numbers > Text Numbers page

**Relationships:**
- Many-to-one with Account (each text number belongs to one account)

**Notes:** The UI presents a dual-list picker with "Available" and "Assigned" columns. Setting is_assigned to true moves the number into the active messaging pool. TextNumbers may overlap with TrackingNumbers (i.e., the same E.164 number can be both a tracking number and text-enabled), but they are managed as separate records to allow independent lifecycle management of voice and messaging capabilities.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| number | e164 | NN, UQ(account_id) | The E.164 number capable of SMS/MMS |
| name | short_text | MAX(128) | Display label for this text number |
| is_assigned | boolean | NN | True when this number is actively assigned to a messaging workflow |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### TargetNumber

**UI References:** Numbers > Target Numbers page

**Relationships:**
- Many-to-one with Account (each target belongs to one account)
- Implicitly referenced by routing rules and call distribution queues

**Notes:** TargetNumbers represent the actual endpoints in a call distribution scenario — individual agents, ring groups, or SIP extensions. Multiple TrackingNumbers may route to the same TargetNumber via queue or router configurations. The concurrency_cap field prevents a single destination from being overwhelmed; null means unlimited simultaneous calls are permitted. The weight field supports weighted round-robin distribution when multiple targets are listed in the same routing tier.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| number | e164 | NN | The E.164 number or SIP address for this target |
| name | short_text | MAX(128), NN | Descriptive label (e.g., "East Coast Sales") |
| description | short_text | MAX(255) | Extended description of this target |
| target_type | enum(Phone Match, SIP, Agent) | NN | Delivery mechanism for calls routed to this target |
| priority | integer | NN | Routing priority order; lower integer = higher priority |
| concurrency_cap | integer | | Maximum simultaneous calls; null means unlimited |
| weight | integer | NN | Relative weight for weighted distribution algorithms |
| status | enum(Active, Inactive) | NN | Whether this target accepts calls |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### TrackingSource

**UI References:** Numbers > Tracking Sources page, Reports > Calls by Source, CallDetailPanel (source field)

**Relationships:**
- Many-to-one with Account (each source belongs to one account)
- One-to-many with TrackingNumber (a source can have many tracking numbers assigned)
- One-to-many with NumberPool (a source can have multiple DNI pools)

**Notes:** TrackingSource is the primary attribution anchor for the entire call tracking model. Every inbound call is associated with a source through its tracking number, and reports aggregate call volume, duration, conversion, and quality metrics by source. The call_count counter is incremented by the call processing pipeline and is used in list views to avoid expensive aggregation queries. The last_touch flag controls whether this source participates in last-touch attribution models vs. first-touch. Position controls display order in UI lists and reports.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | MAX(128), NN | Display name for this source (e.g., "Google Organic", "Facebook Paid") |
| source_type | short_text | MAX(64) | Category label (e.g., "Search", "Social", "Direct", "Referral", "Paid") |
| position | integer | | Display sort order in list views and reports |
| last_touch | boolean | NN | Whether this source uses last-touch attribution logic |
| number_count | counter | NN | Denormalized count of TrackingNumbers currently assigned to this source |
| call_count | counter | NN | Lifetime call count attributed to this source |
| status | enum(Active, Inactive) | NN | Whether this source is active for tracking and reporting |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### NumberPool

**UI References:** Numbers > Number Pools page

**Relationships:**
- Many-to-one with Account (each pool belongs to one account)
- Many-to-one with TrackingSource (each pool is tied to one marketing source)
- One-to-many with NumberPoolMember (pool contains many tracking numbers)

**Notes:** A NumberPool is the core mechanism for Dynamic Number Insertion. When auto_manage is enabled, the platform monitors pool utilization and adds or removes TrackingNumbers to maintain the target_accuracy rate. The target_accuracy percentage represents the desired probability that any given web session sees a unique tracking number — higher accuracy requires more numbers in the pool. Pool sizing is a function of site traffic volume, average session duration, and number recycle delay.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | MAX(128), NN | Display name for this pool |
| description | text | | Extended description of the pool's purpose or configuration |
| source_id | uuid | FK(TrackingSource), NN | The marketing source this pool tracks |
| auto_manage | boolean | NN | Whether the platform automatically manages pool size based on demand |
| target_accuracy | percentage | NN | Desired per-session attribution accuracy (0.0–1.0 representing 0%–100%) |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### NumberPoolMember

**UI References:** Numbers > Number Pools page (pool detail view)

**Relationships:**
- Many-to-one with NumberPool (membership record belongs to one pool)
- One-to-one with TrackingNumber (a tracking number belongs to at most one pool)

**Notes:** This is the junction table between NumberPool and TrackingNumber. The UQ constraint on tracking_number_id enforces that a given tracking number can only belong to one pool at a time, which is required for correct session attribution — a number seen by two different pool sessions simultaneously would produce ambiguous attribution. On pool deletion, all NumberPoolMember records for that pool should be cascade-deleted, and the associated TrackingNumbers should have their pool_id foreign key cleared. The call_count field tracks attribution events originating from this specific pool slot.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| pool_id | uuid | FK(NumberPool), NN | The pool this membership record belongs to |
| tracking_number_id | uuid | FK(TrackingNumber), NN, UQ | The tracking number assigned to this pool slot |
| status | enum(Active, Inactive) | NN | Whether this pool slot is currently active |
| call_count | counter | NN | Number of calls attributed through this pool slot |
| added_at | timestamp_tz | NN | Timestamp when this number was added to the pool |

---

### PortRequest

**UI References:** Numbers > Buy Numbers page (Port Numbers tab / wizard)

**Relationships:**
- Many-to-one with Account (each port request belongs to one account)

**Notes:** PortRequests capture the information required for a Local Number Portability submission to the carrier. The numbers_to_port field stores an array of E.164 numbers because a single port order can transfer multiple numbers from the losing carrier simultaneously. The authorized_signature field captures the name of the person authorized to execute the Letter of Authorization (LOA) on behalf of the account. Billing address fields must match the losing carrier's records exactly or the port will be rejected. The rejection_reason field is populated by the platform operator or carrier when status transitions to Rejected.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account submitting the port request |
| numbers_to_port | json | NN | Array of E.164 numbers to be ported in this order |
| first_name | short_text | MAX(64), NN | First name of the account holder on record with the losing carrier |
| last_name | short_text | MAX(64), NN | Last name of the account holder on record with the losing carrier |
| email | email | NN | Contact email for port status notifications |
| phone | e164 | NN | Contact phone number for the port authorization process |
| billing_address_line1 | short_text | MAX(128), NN | Street address line 1 matching the losing carrier's billing records |
| billing_address_line2 | short_text | MAX(128) | Street address line 2 (suite, unit, etc.) |
| city | short_text | MAX(64), NN | City matching the losing carrier's billing records |
| state | short_text | MAX(64), NN | State or province matching the losing carrier's billing records |
| zip | short_text | MAX(16), NN | Postal code matching the losing carrier's billing records |
| authorized_signature | short_text | MAX(128), NN | Full name of the person authorized to sign the Letter of Authorization |
| status | enum(Draft, Submitted, In Progress, Completed, Rejected) | NN | Current lifecycle state of the port order |
| submitted_at | timestamp_tz | | Timestamp when the port order was submitted to the carrier |
| completed_at | timestamp_tz | | Timestamp when the port completed and numbers became active |
| rejection_reason | text | | Reason provided by the carrier or operator when status is Rejected |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

### CallSettings

**UI References:** Numbers > Call Settings page

**Relationships:**
- Many-to-one with Account (each profile belongs to one account)
- Optionally referenced by TrackingNumber (a tracking number may specify a non-default profile)

**Notes:** CallSettings profiles allow an account to maintain different call handling configurations for different teams, campaigns, or business contexts. Exactly one profile per account must have is_default set to true; application logic enforces this constraint when a new default is selected by clearing the flag on all other profiles first. When a TrackingNumber does not reference a specific CallSettings profile, the account's default profile is applied. The enhanced_caller_id flag enables CNAM (Caller Name) database lookups, which incur per-lookup billing. The caller_id_override flag, when true, presents the tracking number (rather than the caller's actual number) on the agent's phone display, which can be used to indicate which campaign prompted the call.

| Attribute | Semantic Type | Constraints | Description |
|-----------|--------------|-------------|-------------|
| id | uuid | PK, NN | Surrogate primary key |
| account_id | uuid | FK(Account), NN | Owning account |
| name | short_text | MAX(128), NN | Profile name displayed in the settings UI (e.g., "Default", "Sales Team") |
| is_default | boolean | NN | True for the account's default settings profile; only one per account |
| greeting_enabled | boolean | NN | Whether a greeting message is played to the caller before connecting |
| whisper_enabled | boolean | NN | Whether a whisper message is played to the agent before the caller is connected |
| inbound_recording | boolean | NN | Whether inbound calls are recorded |
| outbound_recording | boolean | NN | Whether outbound calls are recorded |
| transcription_enabled | boolean | NN | Whether call recordings are automatically transcribed |
| caller_id_enabled | boolean | NN | Whether caller ID information is captured and displayed |
| enhanced_caller_id | boolean | NN | Whether CNAM lookup is performed to enrich caller ID with caller name |
| caller_id_override | boolean | NN | When true, presents the tracking number as the caller ID on the agent's device |
| spam_filter_enabled | boolean | NN | Whether known spam and robocall numbers are filtered before reaching the agent |
| created_at | timestamp_tz | NN | Record creation time |
| updated_at | timestamp_tz | NN | Last modification time |

---

## Number Lifecycle & DNI Considerations

### Number Provisioning Lifecycle

A TrackingNumber begins its life as a provisioning request. The platform (or a carrier integration layer) reserves an available E.164 number, assigns it to an account, and creates the TrackingNumber record in the Draft or Active state. At provisioning time, the number receives a marketing source assignment (source_id), a routing configuration (routing_type + routing_target_id), and a call settings profile reference. Once active, the number is billed on its billing_date each month.

When a number is decommissioned, it transitions to is_active = false. The historical call records that referenced this number as the inbound tracking number are retained; source attribution reporting remains accurate even after a number is retired. The number itself may be released back to the carrier pool after a regulatory cooling-off period.

### Offsite Static vs. Onsite Dynamic Numbers

Offsite Static numbers represent the classical call tracking model: a single dedicated phone number is printed on an advertisement (billboard, direct mail, radio spot). Every call to that number is attributed to the associated marketing source. These numbers are long-lived and their pool_id is null.

Onsite Dynamic numbers (DNI) are a more sophisticated mechanism designed for digital channels. A JavaScript snippet on the advertiser's website inspects the web session's UTM parameters, referrer, or other session attributes to determine which marketing source drove the visit. It then swaps the phone number displayed on the page with one from the appropriate NumberPool. When the visitor calls that number, attribution flows back to the correct campaign. The number is "claimed" for the session duration and returned to the available pool after the session expires.

### NumberPool Sizing and the Accuracy Model

The target_accuracy percentage on a NumberPool represents a probabilistic guarantee. If a pool has too few numbers relative to the concurrent session volume and average session duration, two different web sessions may be shown the same number simultaneously. When both sessions result in calls, attribution is ambiguous. The auto_manage flag instructs the platform to monitor the pool utilization rate and automatically request additional TrackingNumber provisioning when the observed accuracy drops below target_accuracy.

Pool sizing is computed from three inputs: the peak concurrent session count for the source, the average session duration (how long a visitor stays on the site before the number can be recycled), and the desired accuracy. A rough estimate for required pool size is: pool_size >= peak_concurrent_sessions * (average_session_duration_seconds / number_recycle_delay_seconds). The NumberPoolMember.call_count field provides per-slot utilization data that feeds into this sizing model.

### Attribution Integrity

The integrity of the attribution model depends on several invariants enforced across this domain. A TrackingNumber may belong to at most one NumberPool at any time (enforced by the UQ constraint on NumberPoolMember.tracking_number_id). A TrackingNumber in a DNI pool should not be published in any static marketing material, because doing so would mix dynamic-session attribution with offline attribution on the same number. TrackingSource.call_count and TrackingSource.number_count are denormalized counters that must be kept consistent with the canonical state via application-level event handling or periodic reconciliation.

### Port Requests and Number Ownership Transfer

When a business brings existing numbers onto the platform via Local Number Portability, the PortRequest entity tracks the full lifecycle of the carrier-level transfer. Once a port completes (status = Completed), the ported numbers can be provisioned as TrackingNumbers within the platform. The billing address and authorized signature fields in PortRequest are carrier requirements, not platform requirements — they must exactly match the losing carrier's records. A mismatch on any field is the most common cause of port rejections (status = Rejected).

### Receiving Numbers and Target Numbers

The distinction between ReceivingNumber and TargetNumber reflects two different roles in the call flow. A ReceivingNumber is a passive destination — it is the business's main phone line, and calls arrive there as the final step of a simple direct-forward routing configuration. A TargetNumber is an active participant in a routing algorithm — it has priority, weight, and concurrency controls that govern how it competes with other targets for call distribution. In a complex routing scenario, a call may pass through a queue or smart router that evaluates multiple TargetNumbers before selecting the final destination, which may itself be a ReceivingNumber.
