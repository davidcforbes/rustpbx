CREATE TABLE iiz.agent_scripts (
    id          UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id  UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    description TEXT,
    content     TEXT        NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ
);
CREATE INDEX idx_as_account ON iiz.agent_scripts (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('agent_scripts');
SELECT iiz.add_notify_trigger('agent_scripts');
