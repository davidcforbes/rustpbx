-- Export Records (NOT partitioned — low volume)
CREATE TABLE iiz.export_records (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL,
    name                TEXT,
    export_type         TEXT,
    format              iiz.export_format NOT NULL DEFAULT 'csv',
    date_range          TEXT,
    record_count        INTEGER     NOT NULL DEFAULT 0,
    status              TEXT        NOT NULL DEFAULT 'Processing',
    download_url        TEXT,
    requested_by_id     UUID,
    filters_applied     JSONB,
    completed_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_er_account ON iiz.export_records (account_id);

-- Notifications (NOT partitioned — low volume, mutable is_read flag)
CREATE TABLE iiz.notifications (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL,
    user_id         UUID        NOT NULL,
    event_type      TEXT        NOT NULL,
    title           TEXT        NOT NULL,
    body            TEXT,
    entity_type     TEXT,
    entity_id       UUID,
    is_read         BOOLEAN     NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_notif_user_read ON iiz.notifications (user_id, is_read);
CREATE INDEX idx_notif_account   ON iiz.notifications (account_id);

SELECT iiz.add_updated_at_trigger('notifications');

-- Monitoring Events (NOT partitioned — low volume)
CREATE TABLE iiz.monitoring_events (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL,
    session_id          TEXT,
    call_id             UUID,
    monitor_user_id     UUID        NOT NULL,
    monitored_agent_id  UUID,
    event_type          TEXT        NOT NULL,
    monitor_mode        iiz.monitor_mode NOT NULL,
    started_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at            TIMESTAMPTZ,
    duration_secs       INTEGER,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_me_account ON iiz.monitoring_events (account_id);
