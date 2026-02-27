-- Tags
CREATE TABLE iiz.tags (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    color           TEXT        DEFAULT '#00bcd4',
    description     TEXT,
    usage_count     INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_tags_account_name UNIQUE (account_id, name)
);
CREATE INDEX idx_tags_account ON iiz.tags (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('tags');
SELECT iiz.add_notify_trigger('tags');

-- Call Tags (analytical — insert/delete only)
-- NOTE: call_id has no FK constraint because call_records is a partitioned table
-- and PostgreSQL does not support foreign keys referencing partitioned tables.
CREATE TABLE iiz.call_tags (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    call_id         UUID        NOT NULL,
    tag_id          UUID        NOT NULL REFERENCES iiz.tags(id) ON DELETE CASCADE,
    applied_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    applied_by_type TEXT        NOT NULL DEFAULT 'Manual',
    applied_by_id   UUID        REFERENCES iiz.users(id) ON DELETE SET NULL,
    trigger_id      UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_call_tags UNIQUE (call_id, tag_id)
);
CREATE INDEX idx_ct_call ON iiz.call_tags (call_id);
CREATE INDEX idx_ct_tag ON iiz.call_tags (tag_id);
CREATE INDEX idx_ct_account ON iiz.call_tags (account_id) WHERE deleted_at IS NULL;
-- No updated_at trigger (insert/delete only table)
-- No notify trigger (analytical data, not config)

-- Scoring Configs (config singleton per account)
CREATE TABLE iiz.scoring_configs (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    answer_rate_weight  INTEGER     NOT NULL DEFAULT 34,
    talk_time_weight    INTEGER     NOT NULL DEFAULT 33,
    conversion_weight   INTEGER     NOT NULL DEFAULT 33,
    min_talk_time_secs  INTEGER     NOT NULL DEFAULT 60,
    target_answer_rate  INTEGER     NOT NULL DEFAULT 90,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,
    CONSTRAINT uq_scoring_configs_account UNIQUE (account_id)
);
CREATE INDEX idx_scoring_configs_account ON iiz.scoring_configs (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('scoring_configs');
SELECT iiz.add_notify_trigger('scoring_configs');
