CREATE TABLE iiz.keyword_spotting_configs (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                    TEXT        NOT NULL,
    sensitivity             TEXT        NOT NULL DEFAULT 'Medium',
    apply_to_all_numbers    BOOLEAN     NOT NULL DEFAULT true,
    is_active               BOOLEAN     NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_ksc_account ON iiz.keyword_spotting_configs (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('keyword_spotting_configs');
SELECT iiz.add_notify_trigger('keyword_spotting_configs');

CREATE TABLE iiz.keyword_spotting_keywords (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    config_id       UUID        NOT NULL REFERENCES iiz.keyword_spotting_configs(id) ON DELETE CASCADE,
    keyword         TEXT        NOT NULL,
    category        TEXT        NOT NULL DEFAULT 'Neutral',
    score_weight    REAL        NOT NULL DEFAULT 1.0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_ksk_account ON iiz.keyword_spotting_keywords (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_ksk_config ON iiz.keyword_spotting_keywords (config_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('keyword_spotting_keywords');
SELECT iiz.add_notify_trigger('keyword_spotting_keywords');

CREATE TABLE iiz.keyword_spotting_numbers (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    config_id           UUID        NOT NULL REFERENCES iiz.keyword_spotting_configs(id) ON DELETE CASCADE,
    tracking_number_id  UUID        NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,
    CONSTRAINT uq_ks_numbers UNIQUE (config_id, tracking_number_id)
);
CREATE INDEX idx_ksn_account ON iiz.keyword_spotting_numbers (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_ksn_config ON iiz.keyword_spotting_numbers (config_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('keyword_spotting_numbers');
SELECT iiz.add_notify_trigger('keyword_spotting_numbers');
