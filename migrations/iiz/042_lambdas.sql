CREATE TABLE iiz.lambdas (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    runtime             TEXT        NOT NULL DEFAULT 'Node.js 18',
    code                TEXT        NOT NULL DEFAULT '',
    handler             TEXT        NOT NULL DEFAULT 'handler',
    timeout_ms          INTEGER     NOT NULL DEFAULT 30000,
    memory_mb           INTEGER     NOT NULL DEFAULT 128,
    last_invoked_at     TIMESTAMPTZ,
    invocation_count    INTEGER     NOT NULL DEFAULT 0,
    error_count         INTEGER     NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_lam_account ON iiz.lambdas (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('lambdas');
SELECT iiz.add_notify_trigger('lambdas');

CREATE TABLE iiz.lambda_env_vars (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    lambda_id       UUID        NOT NULL REFERENCES iiz.lambdas(id) ON DELETE CASCADE,
    key             TEXT        NOT NULL,
    value           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_lambda_env_vars UNIQUE (lambda_id, key)
);
CREATE INDEX idx_lev_account ON iiz.lambda_env_vars (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_lev_lambda ON iiz.lambda_env_vars (lambda_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('lambda_env_vars');
SELECT iiz.add_notify_trigger('lambda_env_vars');
