CREATE TABLE iiz.text_numbers (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number          TEXT        NOT NULL,
    name            TEXT,
    is_assigned     BOOLEAN     NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_text_numbers_account UNIQUE (account_id, number)
);
CREATE INDEX idx_txn_account ON iiz.text_numbers (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('text_numbers');
SELECT iiz.add_notify_trigger('text_numbers');
