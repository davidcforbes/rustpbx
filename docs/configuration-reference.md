# RustPBX Configuration Reference

This document provides a complete reference for every configuration option in RustPBX. The configuration file uses TOML format and is loaded from `rustpbx.toml` by default, or from a custom path via the `--conf` CLI argument.

**Source of truth**: `src/config.rs` and related struct definitions.

---

## Table of Contents

1. [Top-Level Options](#top-level-options)
2. [Proxy Core (`[proxy]`)](#proxy-core-proxy)
3. [User Backends (`[[proxy.user_backends]]`)](#user-backends-proxyuser_backends)
4. [Locator (`[proxy.locator]`)](#locator-proxylocator)
5. [Locator Webhook (`[proxy.locator_webhook]`)](#locator-webhook-proxylocator_webhook)
6. [HTTP Router (`[proxy.http_router]`)](#http-router-proxyhttp_router)
7. [Routes (`[[proxy.routes]]`)](#routes-proxyroutes)
8. [Trunks (`[proxy.trunks.<name>]`)](#trunks-proxytrunksname)
9. [Queues (`[proxy.queues.<name>]`)](#queues-proxyqueuesname)
10. [Transcript (`[proxy.transcript]`)](#transcript-proxytranscript)
11. [Recording (`[recording]`)](#recording-recording)
12. [Call Records (`[callrecord]`)](#call-records-callrecord)
13. [SIP Flow (`[sipflow]`)](#sip-flow-sipflow)
14. [Quality Monitoring (`[quality]`)](#quality-monitoring-quality)
15. [Voicemail (`[voicemail]`)](#voicemail-voicemail)
16. [Console (`[console]`)](#console-console)
17. [AMI (`[ami]`)](#ami-ami)
18. [Archive (`[archive]`)](#archive-archive)
19. [Storage (`[storage]`)](#storage-storage)
20. [ICE Servers (`[[ice_servers]]`)](#ice-servers-ice_servers)
21. [Addons (`[addons]`)](#addons-addons)
22. [Environment Variables](#environment-variables)
23. [CLI Arguments](#cli-arguments)
24. [Example Configurations](#example-configurations)

---

## Top-Level Options

These options are defined at the root level of the TOML file, outside any section.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `http_addr` | string | `"0.0.0.0:8080"` | HTTP listener address for API, console, and webhooks. |
| `http_gzip` | bool | `false` | Enable gzip compression for HTTP responses. |
| `https_addr` | string | _(none)_ | HTTPS listener address. Requires `ssl_certificate` and `ssl_private_key`. Example: `"0.0.0.0:8443"` |
| `ssl_certificate` | string | _(none)_ | Path to the TLS certificate file (PEM format) for HTTPS. |
| `ssl_private_key` | string | _(none)_ | Path to the TLS private key file (PEM format) for HTTPS. |
| `log_level` | string | _(none)_ | Global log level. Values: `"debug"`, `"info"`, `"warn"`, `"error"`, `"trace"`. If unset, controlled by `RUST_LOG` environment variable. |
| `log_file` | string | _(none)_ | Path to a log file. If unset, logs to stderr. |
| `http_access_skip_paths` | list of strings | `[]` | HTTP paths to exclude from access logging (e.g., `["/health", "/metrics"]`). |
| `external_ip` | string | _(none)_ | Public IP address advertised in SDP for RTP media. **Required if behind NAT**. Do not include a port. |
| `rtp_start_port` | integer | `12000` | Start of the RTP UDP port range. |
| `rtp_end_port` | integer | `42000` | End of the RTP UDP port range. |
| `webrtc_port_start` | integer | `30000` | Start of the WebRTC UDP port range. |
| `webrtc_port_end` | integer | `40000` | End of the WebRTC UDP port range. |
| `database_url` | string | `"sqlite://rustpbx.sqlite3"` | Database connection URL. Supports `sqlite://`, `mysql://`, and `postgres://` schemes. |
| `demo_mode` | bool | `false` | Enable demo mode (restricts certain operations). Also toggled by `RUSTPBX_DEMO_MODE=true` environment variable. |

---

## Proxy Core (`[proxy]`)

The `[proxy]` section controls the SIP signaling engine, transport bindings, modules, and core behavior.

### Transport and Binding

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `addr` | string | `"0.0.0.0"` | IP address for SIP transport listeners. |
| `udp_port` | integer | `5060` | SIP over UDP port. Set to omit/null to disable. |
| `tcp_port` | integer | _(none)_ | SIP over TCP port. |
| `tls_port` | integer | _(none)_ | SIP over TLS (SIPS) port. Requires `ssl_certificate` and `ssl_private_key`. |
| `ws_port` | integer | _(none)_ | SIP over WebSocket port (for WebRTC clients). |
| `ws_handler` | string | _(none)_ | WebSocket handler path (e.g., `"/ws"`). Required if `ws_port` is set. |
| `ssl_certificate` | string | _(none)_ | Path to TLS certificate for SIP TLS/WSS. Overrides top-level if set. |
| `ssl_private_key` | string | _(none)_ | Path to TLS private key for SIP TLS/WSS. Overrides top-level if set. |

### Identity and Behavior

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `useragent` | string | auto-generated | SIP User-Agent header value. Defaults to `"RustPBX/<version>"`. |
| `callid_suffix` | string | `"miuda.ai"` | Suffix appended to generated Call-ID values (e.g., `"id@miuda.ai"`). |
| `realms` | list of strings | `[]` | SIP realms (domains) served by this proxy. Used for authentication challenges and routing. If empty, the proxy accepts requests to any domain. |
| `modules` | list of strings | `["acl", "auth", "registrar", "call"]` | Internal module pipeline. Available: `"acl"`, `"auth"`, `"presence"`, `"registrar"`, `"call"`. |
| `addons` | list of strings | _(none)_ | Additional addon modules to load (e.g., `["transcript", "wholesale", "queue"]`). |

### Security and Limits

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_concurrency` | integer | _(none)_ | Maximum simultaneous SIP transactions. |
| `ensure_user` | bool | `true` | If true, silently ignore requests to unknown users (prevents user enumeration). |
| `ua_white_list` | list of strings | `[]` | Only allow SIP requests from these User-Agent strings. Empty means allow all. |
| `ua_black_list` | list of strings | `[]` | Reject SIP requests from these User-Agent strings (e.g., `["friendly-scanner", "pplsip"]`). |
| `acl_rules` | list of strings | `["allow all", "deny all"]` | Inline ACL rules. Evaluated in order. Format: `"allow <ip/cidr>"` or `"deny <ip/cidr>"` or `"allow all"` / `"deny all"`. |
| `acl_files` | list of strings | `[]` | Glob patterns for external ACL rule files (e.g., `["config/acl/*.toml"]`). |
| `frequency_limiter` | string | _(none)_ | Rate limiting configuration string. |
| `nat_fix` | bool | `true` | Enable NAT inspection and Contact header rewriting for private IP addresses. |

### Registration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `registrar_expires` | integer | `60` | Default registration expiry time in seconds. |

### Session Timers (RFC 4028)

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `session_timer` | bool | `false` | Enable SIP session timers. |
| `session_expires` | integer | _(none)_ | Session expiry in seconds (e.g., `1800` for 30 minutes). |

### Media

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `media_proxy` | string | `"auto"` | Media proxy mode. Values: `"all"` (always proxy), `"auto"` (proxy when needed, e.g., WebRTC to RTP), `"nat"` (proxy only for NAT), `"none"` (direct media). |
| `codecs` | list of strings | _(none)_ | Allowed codecs for negotiation (e.g., `["opus", "pcmu", "pcma", "g729"]`). If unset, all supported codecs are allowed. |
| `enable_latching` | bool | `false` | Enable media latching (learn remote RTP address from incoming packets). |

### External Files

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `routes_files` | list of strings | `[]` | Glob patterns for external route rule files (e.g., `["config/routes/*.toml"]`). |
| `trunks_files` | list of strings | `[]` | Glob patterns for external trunk config files (e.g., `["config/trunks/*.toml"]`). |
| `generated_dir` | string | `"./config"` | Root directory for auto-generated configuration (managed by UI/API). |
| `queue_dir` | string | _(none)_ | Directory for queue configuration files. Defaults to `<generated_dir>/queue`. |
| `sip_flow_max_items` | integer | _(none)_ | Maximum SIP flow items to keep in memory. |

---

## User Backends (`[[proxy.user_backends]]`)

User backends provide SIP user authentication and lookup. Multiple backends can be chained; they are queried in definition order.

### Memory Backend

Static user list, ideal for testing.

```toml
[[proxy.user_backends]]
type = "memory"
users = [
    { username = "1001", password = "secret", realm = "example.com", display_name = "Alice", enabled = true, allow_guest_calls = false },
    { username = "1002", password = "secret2" },
]
```

**User fields:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `username` | string | **required** | SIP username. |
| `password` | string | _(none)_ | SIP password. |
| `realm` | string | _(none)_ | SIP realm/domain. |
| `display_name` | string | _(none)_ | Display name for caller ID. |
| `email` | string | _(none)_ | User email address. |
| `phone` | string | _(none)_ | User phone number. |
| `note` | string | _(none)_ | Free-form note. |
| `enabled` | bool | `true` | Whether the user account is active. |
| `allow_guest_calls` | bool | `false` | Allow calls without SIP registration. |
| `is_support_webrtc` | bool | `false` | Mark user as a WebRTC client. |
| `departments` | list of strings | _(none)_ | Department memberships. |
| `call_forwarding_mode` | string | _(none)_ | Call forwarding mode: `"always"`, `"when_busy"` / `"busy"`, `"when_not_answered"` / `"no_answer"`, `"none"`. |
| `call_forwarding_destination` | string | _(none)_ | Forwarding destination (SIP URI or extension). |
| `call_forwarding_timeout` | integer | _(none)_ | Forwarding timeout in seconds. |

### Database Backend

Load users from the SQL database.

```toml
[[proxy.user_backends]]
type = "database"
# All fields below are optional overrides
url = "sqlite://rustpbx.sqlite3"     # Override main database_url
table_name = "users"
id_column = "id"
username_column = "username"
password_column = "password"
realm_column = "realm"
enabled_column = "is_active"
```

### HTTP Backend

Delegate authentication to an external HTTP service.

```toml
[[proxy.user_backends]]
type = "http"
url = "http://auth-service/verify"
method = "POST"                       # "GET" or "POST" (default: GET)
username_field = "username"           # Query/form parameter name (default: "username")
realm_field = "realm"                 # Query/form parameter name (default: "realm")
headers = { "X-Api-Key" = "secret" } # Custom headers sent with requests
sip_headers = ["X-Custom-Header"]    # SIP headers to forward to the HTTP service
```

### Plain Text Backend

Load users from a simple text file.

```toml
[[proxy.user_backends]]
type = "plain"
path = "./users.txt"
```

### Extension Backend

Database-managed extensions with optional caching.

```toml
[[proxy.user_backends]]
type = "extension"
database_url = "sqlite://rustpbx.sqlite3"  # Optional override
ttl = 3600                                  # Cache TTL in seconds
```

---

## Locator (`[proxy.locator]`)

Configures where SIP registration location data is stored.

### Memory (Default)

```toml
[proxy.locator]
type = "memory"
```

Fast but lost on restart.

### Database

```toml
[proxy.locator]
type = "database"
url = "sqlite://rustpbx.sqlite3"
```

### HTTP

```toml
[proxy.locator]
type = "http"
url = "http://registry-service/lookup"
method = "GET"                         # Optional
username_field = "username"            # Optional
expires_field = "expires"              # Optional
realm_field = "realm"                  # Optional
headers = { "X-Api-Key" = "secret" }  # Optional
```

---

## Locator Webhook (`[proxy.locator_webhook]`)

Triggers HTTP notifications on registration status changes.

```toml
[proxy.locator_webhook]
url = "http://your-app/sip-events"
events = ["registered", "unregistered", "offline"]
timeout_ms = 5000
headers = { "X-API-Key" = "my-secret-key" }
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `url` | string | **required** | Webhook endpoint URL. |
| `events` | list of strings | `[]` | Events to subscribe to: `"registered"`, `"unregistered"`, `"offline"`. |
| `headers` | map | _(none)_ | Custom headers to send with webhook requests. |
| `timeout_ms` | integer | _(none)_ | Request timeout in milliseconds. |

---

## HTTP Router (`[proxy.http_router]`)

Delegate call routing decisions to an external HTTP service.

```toml
[proxy.http_router]
url = "http://route-engine/decision"
timeout_ms = 500
fallback_to_static = true
headers = { "X-Api-Key" = "secret" }
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `url` | string | **required** | HTTP routing service endpoint. |
| `timeout_ms` | integer | _(none)_ | Request timeout in milliseconds. |
| `fallback_to_static` | bool | `false` | Fall back to static routes if the HTTP service fails. |
| `headers` | map | _(none)_ | Custom headers to send with routing requests. |

---

## Routes (`[[proxy.routes]]`)

Call routing rules. Evaluated by `priority` (higher value = higher priority), then by definition order.

```toml
[[proxy.routes]]
name = "Outbound US"
description = "Route US numbers to Telnyx"
priority = 10
direction = "outbound"           # "any", "inbound", "outbound"
disabled = false                 # Set true to disable without removing
source_trunks = ["telnyx"]       # Only match calls from these trunks
source_trunk_ids = [1, 2]        # Only match calls from trunks with these DB IDs
codecs = ["pcmu", "pcma"]        # Override codec list for this route

# Match conditions (all are optional; all specified conditions must match)
[proxy.routes.match]
"from.user" = "^1001$"           # Regex on From user part
"from.host" = "internal.net"     # Regex on From host part
"to.user" = "^1[2-9][0-9]{9}$"  # Regex on To user part
"to.host" = "example.com"        # Regex on To host part
"request_uri.user" = "^9"        # Regex on Request-URI user part
"request_uri.host" = "sip.com"   # Regex on Request-URI host part
"header.X-Custom" = "value"      # Regex on custom SIP header

# Rewrite rules (applied before routing)
[proxy.routes.rewrite]
"from.user" = "anonymous"        # Static replacement
"to.user" = "+1{1}"              # Capture group reference: {1} = first capture from match
"to.host" = "sip.provider.com"
"from.host" = "external.com"
"header.X-Route-Info" = "routed"

# Action (uses serde flatten, so fields are at route level)
dest = "telnyx"                  # Trunk name (string) or list of trunk names
# dest = ["trunk-a", "trunk-b"] # Multiple trunks for load balancing
select = "rr"                    # Selection: "rr" (round-robin), "random", "hash"
hash_key = "from.user"           # Hash key for "hash" selection: "from.user", "to.user", "call-id"
action = "forward"               # "forward" (default), "reject", "busy", "queue"

# For reject action:
# action = "reject"
# [proxy.routes.reject]
# code = 403
# reason = "Forbidden"

# For queue action:
# action = "queue"
# queue = "support-queue"        # Queue name or file path
```

### Match Condition Fields

| Field | Serde Key | Description |
|-------|-----------|-------------|
| `from_user` | `"from.user"` | From header user part (regex). |
| `from_host` | `"from.host"` | From header host part (regex). |
| `to_user` | `"to.user"` | To header user part (regex). |
| `to_host` | `"to.host"` | To header host part (regex). |
| `request_uri_user` | `"request_uri.user"` | Request-URI user part (regex). |
| `request_uri_host` | `"request_uri.host"` | Request-URI host part (regex). |
| `caller` | `"caller"` | Full caller string `user@host` (regex). |
| `callee` | `"callee"` | Full callee string `user@host` (regex). |
| `header.<Name>` | `"header.<Name>"` | Any SIP header value (regex). |

### Rewrite Fields

| Field | Serde Key | Description |
|-------|-----------|-------------|
| `from_user` | `"from.user"` | Rewrite From user part. Supports `{N}` capture group references. |
| `from_host` | `"from.host"` | Rewrite From host part. |
| `to_user` | `"to.user"` | Rewrite To user part. |
| `to_host` | `"to.host"` | Rewrite To host part. |
| `to_port` | `"to.port"` | Rewrite To port. |
| `request_uri_user` | `"request_uri.user"` | Rewrite Request-URI user part. |
| `request_uri_host` | `"request_uri.host"` | Rewrite Request-URI host part. |
| `request_uri_port` | `"request_uri.port"` | Rewrite Request-URI port. |
| `header.<Name>` | `"header.<Name>"` | Add or modify a SIP header. |

### External Route Files

Route rules can also be defined in separate files loaded via `routes_files` glob patterns:

```toml
# In config/routes/telnyx.toml:
[[routes]]
name = "Telnyx Outbound"
priority = 10

[routes.match]
"to.user" = "^\\+?1[2-9][0-9]{9}$"

dest = "telnyx"
```

**Important**: In external route files, use `[[routes]]` (not `[[proxy.routes]]`).

---

## Trunks (`[proxy.trunks.<name>]`)

SIP trunk (gateway) configurations for connecting to external SIP providers.

```toml
[proxy.trunks.telnyx]
dest = "sip:sip.telnyx.com:5060"
backup_dest = "sip:backup.telnyx.com:5060"
transport = "udp"                  # "udp", "tcp", "tls", "ws", "wss"
username = "myuser"
password = "mypassword"
direction = "bidirectional"        # "inbound", "outbound", "bidirectional"
disabled = false
max_calls = 50
max_cps = 5                        # Maximum calls per second
weight = 100                       # Load balancing weight
codec = ["PCMU", "PCMA"]           # Preferred codecs for this trunk
inbound_hosts = ["203.0.113.50"]   # IP whitelist for inbound calls
country = "US"                     # Country code for policy checks
register = true                    # Enable upstream REGISTER for inbound delivery
register_expires = 3600            # Registration expiry in seconds
incoming_from_user_prefix = "+"    # Match prefix on inbound From user
incoming_to_user_prefix = "1707"   # Match prefix on inbound To user
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `dest` | string | **required** | SIP URI of the trunk destination (e.g., `"sip:sip.provider.com:5060"`). |
| `backup_dest` | string | _(none)_ | Failover SIP URI. |
| `username` | string | _(none)_ | Authentication username. |
| `password` | string | _(none)_ | Authentication password. |
| `transport` | string | _(none)_ | Transport protocol: `"udp"`, `"tcp"`, `"tls"`, `"ws"`, `"wss"`. |
| `direction` | string | _(none)_ | Traffic direction: `"inbound"`, `"outbound"`, `"bidirectional"`. |
| `disabled` | bool | _(none)_ | Disable this trunk without removing it. |
| `max_calls` | integer | _(none)_ | Maximum concurrent calls through this trunk. |
| `max_cps` | integer | _(none)_ | Maximum calls per second. |
| `weight` | integer | _(none)_ | Load balancing weight (higher = more traffic). |
| `codec` | list of strings | `[]` | Preferred codec list. |
| `inbound_hosts` | list of strings | `[]` | IP addresses/CIDRs allowed for inbound calls. |
| `country` | string | _(none)_ | Country code for policy enforcement. |
| `register` | bool | _(none)_ | Enable upstream REGISTER to receive inbound calls. |
| `register_expires` | integer | _(none)_ | Registration refresh interval in seconds (default: 3600). |
| `incoming_from_user_prefix` | string | _(none)_ | Match inbound calls by From user prefix (plain string or regex). |
| `incoming_to_user_prefix` | string | _(none)_ | Match inbound calls by To user prefix (plain string or regex). |
| `recording` | table | _(none)_ | Per-trunk recording policy (same structure as `[recording]`). |
| `policy` | table | _(none)_ | Trunk-level policy spec for rate limiting and call restrictions. |

### External Trunk Files

Trunks can be defined in separate files loaded via `trunks_files`:

```toml
# In config/trunks/telnyx.toml:
[trunks.telnyx]
dest = "sip:sip.telnyx.com:5060"
username = "myuser"
password = "mypassword"
```

---

## Queues (`[proxy.queues.<name>]`)

Call queue / ACD (Automatic Call Distribution) configuration.

```toml
[proxy.queues.support]
name = "General Support"
accept_immediately = true
passthrough_ringback = false

[proxy.queues.support.hold]
audio_file = "sounds/hold_music.wav"
loop_playback = true

[proxy.queues.support.strategy]
mode = "sequential"               # "sequential" or "parallel"
wait_timeout_secs = 20

[[proxy.queues.support.strategy.targets]]
uri = "sip:1001@local"
label = "Alice"

[[proxy.queues.support.strategy.targets]]
uri = "sip:1002@local"
label = "Bob"

[proxy.queues.support.fallback]
redirect = "sip:voicemail@local"
# Or: failure_code = 480
# Or: failure_reason = "No agents available"
# Or: failure_prompt = "sounds/all_agents_busy.wav"
# Or: queue_ref = "overflow_queue"
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `name` | string | _(none)_ | Human-readable queue name. |
| `accept_immediately` | bool | `false` | Send 200 OK immediately before dialing agents. |
| `passthrough_ringback` | bool | `false` | Forward agent's ringback tone to caller (requires `accept_immediately`). |

**Hold config** (`hold`):

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `audio_file` | string | _(none)_ | Path to hold music audio file. |
| `loop_playback` | bool | `true` | Loop the hold music. |

**Strategy** (`strategy`):

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `mode` | string | `"sequential"` | Dial mode: `"sequential"` (hunt group) or `"parallel"` (ring all). |
| `wait_timeout_secs` | integer | _(none)_ | Ring timeout per agent in seconds. |
| `targets` | list of targets | `[]` | Agent targets. Each has `uri` (SIP URI) and optional `label`. |

**Fallback** (`fallback`):

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `redirect` | string | _(none)_ | SIP URI to redirect to on failure. |
| `failure_code` | integer | _(none)_ | SIP status code to return (100-699). |
| `failure_reason` | string | _(none)_ | Reason phrase for the failure response. |
| `failure_prompt` | string | _(none)_ | Audio file to play before hanging up. |
| `queue_ref` | string | _(none)_ | Name of another queue to overflow to. |

---

## Transcript (`[proxy.transcript]`)

Configuration for the post-call transcription tool.

```toml
[proxy.transcript]
command = "groq-sensevoice-wrapper"     # Path or name of the transcription CLI tool
models_path = "/opt/models"             # Directory containing ASR model files
timeout_secs = 120                      # Timeout for the transcription process
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `command` | string | _(none)_ | Path or name of the transcription command. |
| `models_path` | string | _(none)_ | Directory containing ASR model files. |
| `timeout_secs` | integer | _(none)_ | Timeout in seconds for each transcription run. |

---

## Recording (`[recording]`)

Global recording policy. Controls when and how calls are recorded. Can also be set per-trunk under `[proxy.trunks.<name>.recording]`.

```toml
[recording]
enabled = true
directions = ["inbound", "outbound", "internal"]
auto_start = true
path = "./config/recorders"
filename_pattern = "{call_id}_{timestamp}"
samplerate = 8000
ptime = 20
input_gain = 1.0
output_gain = 1.0
caller_allow = ["1001", "1002"]
caller_deny = []
callee_allow = []
callee_deny = ["911"]
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `false` | Enable call recording. |
| `directions` | list of strings | `[]` | Recording directions: `"inbound"`, `"outbound"`, `"internal"`. Empty means record all directions. |
| `auto_start` | bool | `true` | Automatically start recording when a call is answered. |
| `path` | string | `"./config/recorders"` | Directory to store recording files. |
| `filename_pattern` | string | _(none)_ | Pattern for recording filenames. |
| `samplerate` | integer | _(none)_ | Audio sample rate in Hz (e.g., `8000`, `16000`). |
| `ptime` | integer | _(none)_ | Packetization time in milliseconds. |
| `input_gain` | float | _(none)_ | Gain multiplier for the input (caller) audio channel. |
| `output_gain` | float | _(none)_ | Gain multiplier for the output (callee) audio channel. |
| `caller_allow` | list of strings | `[]` | Only record calls from these caller patterns. |
| `caller_deny` | list of strings | `[]` | Never record calls from these caller patterns. |
| `callee_allow` | list of strings | `[]` | Only record calls to these callee patterns. |
| `callee_deny` | list of strings | `[]` | Never record calls to these callee patterns. |

---

## Call Records (`[callrecord]`)

Configures where CDR (Call Detail Records) and associated media files are stored.

### Local Storage

```toml
[callrecord]
type = "local"
root = "./config/cdr"
```

### S3 Compatible Storage

```toml
[callrecord]
type = "s3"
vendor = "aws"                   # "aws", "gcp", "azure", "aliyun", "tencent", "minio", "digitalocean"
bucket = "my-recordings"
region = "us-east-1"
access_key = "ACCESS_KEY"
secret_key = "SECRET_KEY"
endpoint = "http://minio:9000"   # Required for non-AWS
root = "/daily-records"
with_media = true                # Upload media files along with CDR
keep_media_copy = false          # Keep a local copy of media after upload
```

### HTTP Webhook

```toml
[callrecord]
type = "http"
url = "http://my-crm/cdr-hook"
headers = { "X-Api-Key" = "secret" }
with_media = true                # Include media in webhook payload
keep_media_copy = true           # Keep local copy after upload
```

| Option | Type | Description |
|--------|------|-------------|
| `type` | string | Backend type: `"local"`, `"s3"`, `"http"`. |
| `root` | string | Root directory/path for CDR storage (local and S3). |
| `vendor` | string | S3 vendor: `"aws"`, `"gcp"`, `"azure"`, `"aliyun"`, `"tencent"`, `"minio"`, `"digitalocean"`. |
| `bucket` | string | S3 bucket name. |
| `region` | string | S3 region. |
| `access_key` | string | S3 access key. |
| `secret_key` | string | S3 secret key. |
| `endpoint` | string | S3 endpoint URL (required for non-AWS vendors). |
| `with_media` | bool | Include media files with CDR (S3 and HTTP). |
| `keep_media_copy` | bool | Retain local media copy after uploading (S3 and HTTP). |
| `url` | string | Webhook URL (HTTP type). |
| `headers` | map | Custom headers for webhook requests (HTTP type). |

---

## SIP Flow (`[sipflow]`)

SipFlow is an advanced unified SIP+RTP recording system providing superior I/O performance compared to traditional call recording.

### Local Backend

```toml
[sipflow]
type = "local"
root = "./config/sipflow"
subdirs = "daily"                # "none", "daily", "hourly"
flush_count = 1000               # Flush after this many items
flush_interval_secs = 5          # Flush interval in seconds
id_cache_size = 8192             # LRU cache size for call IDs
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `root` | string | **required** | Root directory for SIP flow data. |
| `subdirs` | string | `"daily"` | Subdirectory organization: `"none"`, `"daily"` (YYYYMMDD), `"hourly"` (YYYYMMDD/HH). |
| `flush_count` | integer | `1000` | Number of items to buffer before flushing to disk. |
| `flush_interval_secs` | integer | `5` | Flush interval in seconds. |
| `id_cache_size` | integer | `8192` | LRU cache size for call ID lookups. |

### Remote Backend

```toml
[sipflow]
type = "remote"
udp_addr = "10.0.0.10:9000"
http_addr = "http://10.0.0.10:9001"
timeout_secs = 10
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `udp_addr` | string | **required** | UDP address for SIP flow data ingest. |
| `http_addr` | string | **required** | HTTP address for SIP flow queries. |
| `timeout_secs` | integer | `10` | Request timeout in seconds. |

---

## Quality Monitoring (`[quality]`)

Call quality monitoring with watchdog, MOS estimation, and media state tracking.

```toml
[quality]
enabled = true
watchdog_interval_secs = 2
loss_warning_pct = 1.0
loss_critical_pct = 5.0
jitter_warning_ms = 30.0
jitter_critical_ms = 50.0
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `true` | Enable quality monitoring. |
| `watchdog_interval_secs` | integer | `2` | Interval in seconds between quality checks. |
| `loss_warning_pct` | float | _(none)_ | Packet loss warning threshold (percentage). |
| `loss_critical_pct` | float | _(none)_ | Packet loss critical threshold (percentage). |
| `jitter_warning_ms` | float | _(none)_ | Jitter warning threshold (milliseconds). |
| `jitter_critical_ms` | float | _(none)_ | Jitter critical threshold (milliseconds). |

---

## Voicemail (`[voicemail]`)

Voicemail system configuration.

```toml
[voicemail]
enabled = true
max_message_duration_secs = 120
max_messages_per_mailbox = 50
greeting_path = "./config/voicemail/greetings"
storage_path = "./config/voicemail/messages"
auto_transcribe = true
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `true` | Enable voicemail system. |
| `max_message_duration_secs` | integer | `120` | Maximum voicemail message duration in seconds. |
| `max_messages_per_mailbox` | integer | `50` | Maximum messages per user mailbox. |
| `greeting_path` | string | `"./config/voicemail/greetings"` | Directory for custom greeting audio files. |
| `storage_path` | string | `"./config/voicemail/messages"` | Directory for voicemail message storage. |
| `auto_transcribe` | bool | `true` | Automatically transcribe voicemail messages (requires transcript addon). |

---

## Console (`[console]`)

Web-based management console configuration. Requires the `console` feature (enabled by default).

```toml
[console]
base_path = "/console"
session_secret = "change-me-random-string-32-chars-min"
allow_registration = false
secure_cookie = false
# CDN overrides (optional)
alpine_js = "https://cdn.example.com/alpine.min.js"
tailwind_js = "https://cdn.example.com/tailwind.min.js"
chart_js = "https://cdn.example.com/chart.min.js"
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `base_path` | string | `"/console"` | URL path prefix for the console. |
| `session_secret` | string | random 32-char string | Secret key for session cookies. **Change this in production.** |
| `allow_registration` | bool | `false` | Allow self-service admin account registration after the first account. |
| `secure_cookie` | bool | `false` | Force the Secure attribute on session cookies. Set to `true` when running behind HTTPS. |
| `alpine_js` | string | _(none)_ | Custom URL for Alpine.js (CDN override). |
| `tailwind_js` | string | _(none)_ | Custom URL for Tailwind CSS (CDN override). |
| `chart_js` | string | _(none)_ | Custom URL for Chart.js (CDN override). |

---

## AMI (`[ami]`)

Asterisk Manager Interface compatibility layer for legacy integrations.

```toml
[ami]
allows = ["127.0.0.1", "10.0.1.10"]
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `allows` | list of strings | _(none)_ | IP whitelist for AMI connections. Use `["*"]` to allow all. If unset, only `127.0.0.1`, `::1`, and `localhost` are allowed. |

---

## Archive (`[archive]`)

Automatic archiving of old call data.

```toml
[archive]
enabled = true
archive_time = "03:00:00"
timezone = "America/New_York"
retention_days = 90
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | **required** | Enable automatic archiving. |
| `archive_time` | string | **required** | Time of day to run archiving (HH:MM:SS). |
| `timezone` | string | _(none)_ | Timezone for the archive schedule (e.g., `"UTC"`, `"America/New_York"`). |
| `retention_days` | integer | **required** | Number of days to retain data before archiving. |

---

## Storage (`[storage]`)

Generic object storage used by addons (transcripts, wholesale exports, etc.). Separate from call recording storage.

### Local Storage

```toml
[storage]
type = "local"
path = "storage/blobs"
```

### S3 Storage

```toml
[storage]
type = "s3"
vendor = "aws"                   # "aws", "gcp", "azure", "aliyun", "tencent", "minio", "digitalocean"
bucket = "app-assets"
region = "us-west-2"
access_key = "ACCESS_KEY"
secret_key = "SECRET_KEY"
endpoint = "http://minio:9000"   # Optional, required for non-AWS
prefix = "rustpbx/"              # Optional path prefix
```

---

## ICE Servers (`[[ice_servers]]`)

STUN/TURN server configuration for WebRTC clients. Provided to browser-based softphones during session setup.

```toml
[[ice_servers]]
urls = ["stun:stun.l.google.com:19302"]

[[ice_servers]]
urls = ["turn:turn.example.com:3478?transport=TCP"]
username = "turnuser"
credential = "turnpassword"
```

| Option | Type | Description |
|--------|------|-------------|
| `urls` | list of strings | STUN/TURN server URLs. |
| `username` | string | TURN authentication username. |
| `credential` | string | TURN authentication password/credential. |

---

## Addons (`[addons]`)

Key-value configuration maps for addon modules. Each addon is a sub-table under `[addons]`.

```toml
[proxy]
addons = ["transcript", "wholesale", "queue"]

[addons.wholesale]
license = "key-123"
billing_cycle = "monthly"
currency = "USD"

[addons.custom_addon]
api_key = "abc123"
```

Available built-in addons (feature-gated):

| Addon | Feature Flag | Description |
|-------|-------------|-------------|
| `transcript` | `addon-transcript` | Post-call audio transcription. |
| `acme` | `addon-acme` | Let's Encrypt SSL certificate management. |
| `archive` | `addon-archive` | Automatic CDR/recording archival. |
| `wholesale` | `addon-wholesale` | Wholesale billing and rate management. |
| `queue` | built-in | Call queue / ACD management UI. |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUSTPBX_DEMO_MODE` | Set to `"true"` or `"1"` to enable demo mode. |
| `RUST_LOG` | Standard Rust log filter (e.g., `"rustpbx=debug,sqlx=info"`). |
| `TOKIO_CONSOLE` | Enable tokio-console integration (any value). |
| `TOKIO_CONSOLE_BIND` | Address for tokio-console server. |

---

## CLI Arguments

```
rustpbx [OPTIONS] [COMMAND]

Options:
  --conf <PATH>                Path to the configuration file (TOML format)
  --tokio-console <ADDR>       Tokio console server address (e.g., 127.0.0.1:5556)
  --super-username <NAME>      Create/update a console super user
  --super-password <PASS>      Password for the console super user
  --super-email <EMAIL>        Email for the super user (defaults to username@localhost)

Commands:
  check-config                 Validate configuration and exit
  fixtures                     Initialize with demo fixture data
```

---

## Example Configurations

### Minimal Development Setup

```toml
http_addr = "0.0.0.0:8080"
database_url = "sqlite://rustpbx.sqlite3"

[console]
base_path = "/console"

[proxy]
addr = "0.0.0.0"
udp_port = 5060
modules = ["auth", "registrar", "call"]

[[proxy.user_backends]]
type = "memory"
users = [
    { username = "1001", password = "password" },
    { username = "1002", password = "password" },
]
```

### Production Setup with HTTPS and Telnyx Trunk

```toml
http_addr = "0.0.0.0:8080"
https_addr = "0.0.0.0:8443"
ssl_certificate = "/etc/letsencrypt/live/pbx.example.com/fullchain.pem"
ssl_private_key = "/etc/letsencrypt/live/pbx.example.com/privkey.pem"
log_level = "info"
log_file = "/var/log/rustpbx.log"
database_url = "sqlite:///var/lib/rustpbx/rustpbx.sqlite3"
external_ip = "203.0.113.10"
rtp_start_port = 20000
rtp_end_port = 20100

[[ice_servers]]
urls = ["stun:stun.l.google.com:19302"]

[console]
base_path = "/console"
session_secret = "CHANGE-ME-LONG-RANDOM-STRING-HERE-32-CHARS"
secure_cookie = true

[ami]
allows = ["127.0.0.1"]

[proxy]
modules = ["acl", "auth", "presence", "registrar", "call"]
addr = "0.0.0.0"
udp_port = 5060
ws_handler = "/ws"
media_proxy = "auto"
registrar_expires = 120
realms = ["203.0.113.10", "pbx.example.com"]
generated_dir = "./config"
routes_files = ["config/routes/*.toml"]
trunks_files = ["config/trunks/*.toml"]
ensure_user = true
nat_fix = true

acl_rules = [
    "allow all",
    "deny all",
]

addons = ["transcript"]

[proxy.transcript]
command = "groq-sensevoice-wrapper"
timeout_secs = 120

[[proxy.user_backends]]
type = "extension"

[proxy.trunks.telnyx]
dest = "sip:sip.telnyx.com:5060"
transport = "udp"
username = "your-telnyx-username"
password = "your-telnyx-password"
direction = "bidirectional"
max_calls = 50
max_cps = 5

[recording]
enabled = true
auto_start = true
directions = ["inbound", "outbound"]
path = "./config/recorders"

[sipflow]
type = "local"
root = "./config/sipflow"
subdirs = "daily"

[callrecord]
type = "local"
root = "./config/cdr"

[quality]
enabled = true
watchdog_interval_secs = 2

[voicemail]
enabled = true
auto_transcribe = true

[archive]
enabled = true
archive_time = "03:00:00"
retention_days = 90
```

### Docker Deployment (Minimal)

```toml
http_addr = "0.0.0.0:8080"
database_url = "sqlite://rustpbx.sqlite3"
external_ip = "YOUR-SERVER-PUBLIC-IP"
rtp_start_port = 20000
rtp_end_port = 20100

[console]
base_path = "/console"
allow_registration = false

[proxy]
addr = "0.0.0.0"
udp_port = 5060
modules = ["auth", "registrar", "call"]
realms = ["YOUR-SERVER-PUBLIC-IP"]

[[proxy.user_backends]]
type = "extension"

[sipflow]
type = "local"
root = "./config/cdr"
subdirs = "hourly"
```
