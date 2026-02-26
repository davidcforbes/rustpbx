-- Appointments (analytical)
-- NOTE: call_id has no FK constraint because call_records is a partitioned table
-- and PostgreSQL does not support foreign keys referencing partitioned tables.
CREATE TABLE iiz.appointments (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID            NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    call_id             UUID,
    scheduled_at        TIMESTAMPTZ     NOT NULL,
    caller_name         TEXT,
    caller_phone        TEXT,
    source_id           UUID,
    agent_id            UUID            REFERENCES iiz.users(id) ON DELETE SET NULL,
    appointment_type    TEXT            NOT NULL DEFAULT 'New',
    status              TEXT            NOT NULL DEFAULT 'Pending',
    revenue             NUMERIC(12,2),
    notes               TEXT,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_appt_account_scheduled ON iiz.appointments (account_id, scheduled_at);
CREATE INDEX idx_appt_account ON iiz.appointments (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('appointments');

-- Custom Reports (config)
CREATE TABLE iiz.custom_reports (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    report_type         TEXT,
    columns             JSONB,
    filters             JSONB,
    date_range_type     TEXT        NOT NULL DEFAULT 'Last 30 Days',
    custom_start_date   DATE,
    custom_end_date     DATE,
    sort_column         TEXT,
    sort_direction      TEXT        DEFAULT 'DESC',
    schedule            TEXT,
    schedule_recipients JSONB,
    last_run_at         TIMESTAMPTZ,
    created_by_id       UUID        REFERENCES iiz.users(id) ON DELETE SET NULL,
    is_shared           BOOLEAN     NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_cr_account ON iiz.custom_reports (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('custom_reports');
SELECT iiz.add_notify_trigger('custom_reports');

-- Notification Rules (config)
CREATE TABLE iiz.notification_rules (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    metric              TEXT        NOT NULL,
    condition_operator  TEXT        NOT NULL,
    threshold_value     NUMERIC     NOT NULL,
    time_window_minutes INTEGER     NOT NULL DEFAULT 60,
    notification_method TEXT        NOT NULL DEFAULT 'In-App',
    recipients          JSONB,
    cooldown_minutes    INTEGER     NOT NULL DEFAULT 60,
    is_active           BOOLEAN     NOT NULL DEFAULT true,
    last_triggered_at   TIMESTAMPTZ,
    trigger_count       INTEGER     NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_nr_account ON iiz.notification_rules (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('notification_rules');
SELECT iiz.add_notify_trigger('notification_rules');
