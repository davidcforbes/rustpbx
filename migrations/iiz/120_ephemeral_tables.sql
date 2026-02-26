-- UNLOGGED tables skip WAL for ~2x write perf. Data survives normal restarts but lost on crash.

CREATE UNLOGGED TABLE iiz.active_calls (
    id                  UUID            NOT NULL PRIMARY KEY,
    account_id          UUID            NOT NULL,
    call_id             TEXT            NOT NULL UNIQUE,
    caller_name         TEXT,
    caller_number       TEXT,
    callee_number       TEXT,
    agent_id            UUID,
    queue_id            UUID,
    source_id           UUID,
    tracking_number_id  UUID,
    direction           iiz.call_direction NOT NULL,
    status              iiz.active_call_status NOT NULL DEFAULT 'ringing',
    started_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    answered_at         TIMESTAMPTZ,
    is_monitored        BOOLEAN         NOT NULL DEFAULT false,
    monitor_mode        iiz.monitor_mode,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_ac_account ON iiz.active_calls (account_id);
CREATE INDEX idx_ac_agent ON iiz.active_calls (agent_id) WHERE agent_id IS NOT NULL;
SELECT iiz.add_updated_at_trigger('active_calls');

CREATE UNLOGGED TABLE iiz.presence (
    identity            TEXT            NOT NULL PRIMARY KEY,
    account_id          UUID,
    user_id             UUID,
    status              iiz.agent_status NOT NULL DEFAULT 'offline',
    note                TEXT,
    activity            TEXT,
    current_call_id     UUID,
    last_updated        TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_presence_account ON iiz.presence (account_id) WHERE account_id IS NOT NULL;

CREATE UNLOGGED TABLE iiz.locations (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID,
    aor                 TEXT            NOT NULL,
    username            TEXT,
    realm               TEXT,
    destination         TEXT            NOT NULL,
    expires             TIMESTAMPTZ     NOT NULL,
    user_agent          TEXT,
    supports_webrtc     BOOLEAN         NOT NULL DEFAULT false,
    source_ip           TEXT,
    source_port         INTEGER,
    transport           iiz.sip_transport,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_loc_aor ON iiz.locations (aor);
CREATE INDEX idx_loc_expires ON iiz.locations (expires);
SELECT iiz.add_updated_at_trigger('locations');

-- frequency_limits: regular table (needs crash durability for rate limiting)
CREATE TABLE iiz.frequency_limits (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID            NOT NULL,
    policy_id           TEXT            NOT NULL,
    scope               TEXT            NOT NULL,
    limit_type          TEXT            NOT NULL,
    max_count           INTEGER         NOT NULL,
    current_count       INTEGER         NOT NULL DEFAULT 0,
    window_start        TIMESTAMPTZ,
    window_end          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_fl_policy ON iiz.frequency_limits (policy_id, scope);
CREATE INDEX idx_fl_window ON iiz.frequency_limits (window_end) WHERE window_end IS NOT NULL;
SELECT iiz.add_updated_at_trigger('frequency_limits');

-- account_variables: regular config table
CREATE TABLE iiz.account_variables (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID            NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT            NOT NULL,
    value               TEXT,
    description         TEXT,
    is_secret           BOOLEAN         NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,
    CONSTRAINT uq_account_variables UNIQUE (account_id, name)
);
CREATE INDEX idx_av_account ON iiz.account_variables (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('account_variables');
SELECT iiz.add_notify_trigger('account_variables');
