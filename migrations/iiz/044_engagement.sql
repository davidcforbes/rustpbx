-- Bulk Messages
CREATE TABLE iiz.bulk_messages (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    label               TEXT,
    sender_number_id    UUID,
    sender_phone        TEXT,
    message_body        TEXT        NOT NULL,
    msg_type            TEXT        NOT NULL DEFAULT 'SMS',
    contact_list_id     UUID        REFERENCES iiz.contact_lists(id) ON DELETE SET NULL,
    recipient_count     INTEGER     NOT NULL DEFAULT 0,
    sent_count          INTEGER     NOT NULL DEFAULT 0,
    delivered_count     INTEGER     NOT NULL DEFAULT 0,
    failed_count        INTEGER     NOT NULL DEFAULT 0,
    status              TEXT        NOT NULL DEFAULT 'Draft',
    scheduled_at        TIMESTAMPTZ,
    started_at          TIMESTAMPTZ,
    completed_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_bm_account ON iiz.bulk_messages (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('bulk_messages');
SELECT iiz.add_notify_trigger('bulk_messages');

-- Lead Reactor Configs
CREATE TABLE iiz.lead_reactor_configs (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    trigger_event       TEXT        NOT NULL,
    delay_minutes       INTEGER     NOT NULL DEFAULT 0,
    is_active           BOOLEAN     NOT NULL DEFAULT true,
    working_hours_only  BOOLEAN     NOT NULL DEFAULT false,
    max_retries         INTEGER     NOT NULL DEFAULT 3,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_lrc_account ON iiz.lead_reactor_configs (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('lead_reactor_configs');
SELECT iiz.add_notify_trigger('lead_reactor_configs');

-- Lead Reactor Actions
CREATE TABLE iiz.lead_reactor_actions (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    config_id           UUID        NOT NULL REFERENCES iiz.lead_reactor_configs(id) ON DELETE CASCADE,
    sort_order          INTEGER     NOT NULL DEFAULT 0,
    action_type         TEXT        NOT NULL,
    template_content    TEXT,
    action_config       JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_lra_account ON iiz.lead_reactor_actions (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_lra_config ON iiz.lead_reactor_actions (config_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('lead_reactor_actions');
SELECT iiz.add_notify_trigger('lead_reactor_actions');

-- Smart Dialer Configs
CREATE TABLE iiz.smart_dialer_configs (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                    TEXT        NOT NULL,
    mode                    TEXT        NOT NULL DEFAULT 'Preview',
    max_concurrent          INTEGER     NOT NULL DEFAULT 1,
    ring_timeout_secs       INTEGER     NOT NULL DEFAULT 30,
    retry_attempts          INTEGER     NOT NULL DEFAULT 2,
    retry_interval_minutes  INTEGER     NOT NULL DEFAULT 60,
    outbound_number         TEXT,
    outbound_cnam           TEXT,
    start_time              TIME,
    end_time                TIME,
    timezone                TEXT        DEFAULT 'America/New_York',
    active_days             INTEGER     NOT NULL DEFAULT 31,
    contact_list_id         UUID        REFERENCES iiz.contact_lists(id) ON DELETE SET NULL,
    agent_script_id         UUID        REFERENCES iiz.agent_scripts(id) ON DELETE SET NULL,
    is_active               BOOLEAN     NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_sdc_account ON iiz.smart_dialer_configs (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('smart_dialer_configs');
SELECT iiz.add_notify_trigger('smart_dialer_configs');

-- Form Reactor Entries
CREATE TABLE iiz.form_reactor_entries (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    form_fields         TEXT,
    tracking_number_id  UUID,
    call_count          INTEGER     NOT NULL DEFAULT 0,
    status              TEXT        NOT NULL DEFAULT 'Active',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_fre_account ON iiz.form_reactor_entries (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('form_reactor_entries');
SELECT iiz.add_notify_trigger('form_reactor_entries');

-- Reminders
CREATE TABLE iiz.reminders (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT,
    timezone            TEXT        DEFAULT 'America/New_York',
    remind_at           TIMESTAMPTZ,
    is_recurring        BOOLEAN     NOT NULL DEFAULT false,
    recurrence_rule     TEXT,
    contact_source      TEXT,
    contact_phone       TEXT,
    contact_list_id     UUID        REFERENCES iiz.contact_lists(id) ON DELETE SET NULL,
    delivery_method     TEXT        NOT NULL DEFAULT 'Call',
    recipient           TEXT,
    message             TEXT,
    status              TEXT        NOT NULL DEFAULT 'Scheduled',
    call_id             UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_rem_account ON iiz.reminders (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('reminders');
SELECT iiz.add_notify_trigger('reminders');
