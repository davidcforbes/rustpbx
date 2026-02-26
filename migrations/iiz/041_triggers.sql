CREATE TABLE iiz.triggers (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    trigger_event   TEXT        NOT NULL,
    run_on          TEXT,
    runs_7d         INTEGER     NOT NULL DEFAULT 0,
    status          TEXT        NOT NULL DEFAULT 'Active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_trg_account ON iiz.triggers (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('triggers');
SELECT iiz.add_notify_trigger('triggers');

CREATE TABLE iiz.trigger_conditions (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    trigger_id      UUID        NOT NULL REFERENCES iiz.triggers(id) ON DELETE CASCADE,
    sort_order      INTEGER     NOT NULL DEFAULT 0,
    field           TEXT        NOT NULL,
    operator        TEXT        NOT NULL,
    value           TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_tc_account ON iiz.trigger_conditions (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_tc_trigger ON iiz.trigger_conditions (trigger_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('trigger_conditions');
SELECT iiz.add_notify_trigger('trigger_conditions');

CREATE TABLE iiz.trigger_actions (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    trigger_id      UUID        NOT NULL REFERENCES iiz.triggers(id) ON DELETE CASCADE,
    sort_order      INTEGER     NOT NULL DEFAULT 0,
    action_type     TEXT        NOT NULL,
    action_config   JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_ta_account ON iiz.trigger_actions (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_ta_trigger ON iiz.trigger_actions (trigger_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('trigger_actions');
SELECT iiz.add_notify_trigger('trigger_actions');
