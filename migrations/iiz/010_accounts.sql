CREATE TABLE iiz.accounts (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT        NOT NULL,
    account_type        iiz.account_type NOT NULL DEFAULT 'standard',
    parent_account_id   UUID        REFERENCES iiz.accounts(id) ON DELETE SET NULL,
    slug                TEXT        NOT NULL,
    timezone            TEXT        NOT NULL DEFAULT 'America/New_York',
    status              iiz.account_status NOT NULL DEFAULT 'active',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,
    CONSTRAINT uq_accounts_slug UNIQUE (slug)
);

CREATE INDEX idx_accounts_parent ON iiz.accounts (parent_account_id) WHERE parent_account_id IS NOT NULL;
CREATE INDEX idx_accounts_status ON iiz.accounts (status) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('accounts');
SELECT iiz.add_notify_trigger('accounts');
