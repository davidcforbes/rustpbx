CREATE TABLE iiz.receiving_numbers (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number          TEXT        NOT NULL,
    description     TEXT,
    tracking_count  INTEGER     NOT NULL DEFAULT 0,
    total_calls     INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_receiving_numbers_account UNIQUE (account_id, number)
);
CREATE INDEX idx_rn_account ON iiz.receiving_numbers (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('receiving_numbers');
SELECT iiz.add_notify_trigger('receiving_numbers');
