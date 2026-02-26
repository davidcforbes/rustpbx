CREATE TABLE iiz.chat_widgets (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    website_url         TEXT,
    tracking_number_id  UUID,
    routing_type        TEXT,
    queue_id            UUID,
    agent_count         INTEGER     NOT NULL DEFAULT 0,
    custom_fields_count INTEGER     NOT NULL DEFAULT 0,
    status              TEXT        NOT NULL DEFAULT 'Draft',
    config_json         JSONB,
    chat_count          INTEGER     NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_cw_account ON iiz.chat_widgets (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('chat_widgets');
SELECT iiz.add_notify_trigger('chat_widgets');
