CREATE TABLE iiz.call_settings (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                    TEXT        NOT NULL,
    is_default              BOOLEAN     NOT NULL DEFAULT false,
    greeting_enabled        BOOLEAN     NOT NULL DEFAULT false,
    whisper_enabled         BOOLEAN     NOT NULL DEFAULT false,
    inbound_recording       BOOLEAN     NOT NULL DEFAULT true,
    outbound_recording      BOOLEAN     NOT NULL DEFAULT false,
    transcription_enabled   BOOLEAN     NOT NULL DEFAULT false,
    caller_id_enabled       BOOLEAN     NOT NULL DEFAULT true,
    enhanced_caller_id      BOOLEAN     NOT NULL DEFAULT false,
    caller_id_override      BOOLEAN     NOT NULL DEFAULT false,
    spam_filter_enabled     BOOLEAN     NOT NULL DEFAULT false,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_cs_account ON iiz.call_settings (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('call_settings');
SELECT iiz.add_notify_trigger('call_settings');
