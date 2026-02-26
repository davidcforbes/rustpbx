CREATE TABLE iiz.tracking_sources (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    source_type     TEXT,
    position        INTEGER     NOT NULL DEFAULT 0,
    last_touch      BOOLEAN     NOT NULL DEFAULT false,
    number_count    INTEGER     NOT NULL DEFAULT 0,
    call_count      INTEGER     NOT NULL DEFAULT 0,
    status          TEXT        NOT NULL DEFAULT 'Active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_ts_account ON iiz.tracking_sources (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('tracking_sources');
SELECT iiz.add_notify_trigger('tracking_sources');
