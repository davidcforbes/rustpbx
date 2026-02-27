CREATE TABLE iiz.ask_ai_configs (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    preset              TEXT        NOT NULL DEFAULT 'Custom Question',
    custom_prompt       TEXT,
    tracking_number_id  UUID,
    delay               TEXT        DEFAULT 'Immediately',
    output_action       TEXT,
    workflow_ids        JSONB,
    is_active           BOOLEAN     NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_aac_account ON iiz.ask_ai_configs (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('ask_ai_configs');
SELECT iiz.add_notify_trigger('ask_ai_configs');

CREATE TABLE iiz.summary_configs (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    phone_enabled           BOOLEAN     NOT NULL DEFAULT true,
    video_enabled           BOOLEAN     NOT NULL DEFAULT false,
    chat_enabled            BOOLEAN     NOT NULL DEFAULT false,
    enabled_summary_types   JSONB,
    transcribe_all          BOOLEAN     NOT NULL DEFAULT false,
    transcription_language  TEXT        DEFAULT 'en',
    pii_redaction_enabled   BOOLEAN     NOT NULL DEFAULT false,
    pii_redaction_rules     TEXT,
    default_model           TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,
    CONSTRAINT uq_summary_configs_account UNIQUE (account_id)
);
CREATE INDEX idx_summary_configs_account ON iiz.summary_configs (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('summary_configs');
SELECT iiz.add_notify_trigger('summary_configs');
