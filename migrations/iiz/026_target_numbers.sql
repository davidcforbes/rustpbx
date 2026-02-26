CREATE TABLE iiz.target_numbers (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number          TEXT        NOT NULL,
    name            TEXT        NOT NULL,
    description     TEXT,
    target_type     TEXT        NOT NULL DEFAULT 'Phone Match',
    priority        INTEGER     NOT NULL DEFAULT 0,
    concurrency_cap INTEGER,
    weight          INTEGER     NOT NULL DEFAULT 1,
    status          TEXT        NOT NULL DEFAULT 'Active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_tgn_account ON iiz.target_numbers (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('target_numbers');
SELECT iiz.add_notify_trigger('target_numbers');
