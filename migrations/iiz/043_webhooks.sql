CREATE TABLE iiz.webhooks (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    trigger_event       TEXT,
    callback_url        TEXT        NOT NULL,
    method              TEXT        NOT NULL DEFAULT 'POST',
    body_type           TEXT        NOT NULL DEFAULT 'JSON',
    headers             JSONB,
    secret              TEXT,
    retry_count         INTEGER     NOT NULL DEFAULT 3,
    retry_delay_secs    INTEGER     NOT NULL DEFAULT 60,
    status              TEXT        NOT NULL DEFAULT 'Active',
    last_triggered_at   TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_wh_account ON iiz.webhooks (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('webhooks');
SELECT iiz.add_notify_trigger('webhooks');

CREATE TABLE iiz.webhook_subscriptions (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    webhook_id      UUID        NOT NULL REFERENCES iiz.webhooks(id) ON DELETE CASCADE,
    event_type      TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_webhook_subs UNIQUE (webhook_id, event_type)
);
CREATE INDEX idx_ws_account ON iiz.webhook_subscriptions (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_ws_webhook ON iiz.webhook_subscriptions (webhook_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('webhook_subscriptions');
SELECT iiz.add_notify_trigger('webhook_subscriptions');
