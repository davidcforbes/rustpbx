CREATE TABLE iiz.number_pools (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    description     TEXT,
    source_id       UUID        REFERENCES iiz.tracking_sources(id) ON DELETE SET NULL,
    auto_manage     BOOLEAN     NOT NULL DEFAULT false,
    target_accuracy INTEGER     NOT NULL DEFAULT 95,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_np_account ON iiz.number_pools (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_np_source ON iiz.number_pools (source_id) WHERE source_id IS NOT NULL;
SELECT iiz.add_updated_at_trigger('number_pools');
SELECT iiz.add_notify_trigger('number_pools');

CREATE TABLE iiz.number_pool_members (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    pool_id             UUID        NOT NULL REFERENCES iiz.number_pools(id) ON DELETE CASCADE,
    tracking_number_id  UUID        NOT NULL UNIQUE,
    status              TEXT        NOT NULL DEFAULT 'Active',
    call_count          INTEGER     NOT NULL DEFAULT 0,
    added_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_npm_pool ON iiz.number_pool_members (pool_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_npm_account ON iiz.number_pool_members (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('number_pool_members');
SELECT iiz.add_notify_trigger('number_pool_members');
