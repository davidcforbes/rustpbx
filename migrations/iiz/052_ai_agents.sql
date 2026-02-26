CREATE TABLE iiz.voice_ai_agents (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                    TEXT        NOT NULL,
    welcome_message         TEXT,
    instructions            TEXT,
    voice                   TEXT        DEFAULT 'alloy',
    language                TEXT        DEFAULT 'en-US',
    knowledge_bank_id       UUID        REFERENCES iiz.knowledge_banks(id) ON DELETE SET NULL,
    max_turns               INTEGER     NOT NULL DEFAULT 10,
    handoff_threshold       TEXT        DEFAULT 'Medium',
    handoff_destination_type TEXT,
    handoff_destination_id  UUID,
    is_active               BOOLEAN     NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_vai_account ON iiz.voice_ai_agents (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('voice_ai_agents');
SELECT iiz.add_notify_trigger('voice_ai_agents');

CREATE TABLE iiz.chat_ai_agents (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    instructions        TEXT,
    knowledge_bank_id   UUID        REFERENCES iiz.knowledge_banks(id) ON DELETE SET NULL,
    welcome_message     TEXT,
    max_turns           INTEGER     NOT NULL DEFAULT 10,
    handoff_threshold   TEXT        DEFAULT 'Medium',
    handoff_queue_id    UUID,
    is_active           BOOLEAN     NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_caa_account ON iiz.chat_ai_agents (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('chat_ai_agents');
SELECT iiz.add_notify_trigger('chat_ai_agents');

CREATE TABLE iiz.chat_ai_configs (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                    TEXT        NOT NULL,
    knowledge_bank_id       UUID        REFERENCES iiz.knowledge_banks(id) ON DELETE SET NULL,
    instructions            TEXT,
    max_turns               INTEGER     NOT NULL DEFAULT 10,
    handoff_threshold       TEXT        DEFAULT 'Medium',
    crm_integration_enabled BOOLEAN     NOT NULL DEFAULT false,
    crm_type                TEXT,
    crm_config              JSONB,
    is_active               BOOLEAN     NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_cac_account ON iiz.chat_ai_configs (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('chat_ai_configs');
SELECT iiz.add_notify_trigger('chat_ai_configs');

CREATE TABLE iiz.dialogflow_configs (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    project_id          TEXT,
    service_account_json TEXT,
    language            TEXT        DEFAULT 'en',
    default_intent      TEXT,
    fallback_message    TEXT,
    connection_status   TEXT        NOT NULL DEFAULT 'Untested',
    last_tested_at      TIMESTAMPTZ,
    is_active           BOOLEAN     NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_dc_account ON iiz.dialogflow_configs (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('dialogflow_configs');
SELECT iiz.add_notify_trigger('dialogflow_configs');
