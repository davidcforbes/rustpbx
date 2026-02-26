CREATE TABLE iiz.tracking_numbers (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number                  TEXT        NOT NULL,
    source_id               UUID        REFERENCES iiz.tracking_sources(id) ON DELETE SET NULL,
    routing_description     TEXT,
    routing_type            TEXT,
    routing_target_type     TEXT,
    routing_target_id       UUID,
    text_enabled            BOOLEAN     NOT NULL DEFAULT false,
    receiving_number_id     UUID        REFERENCES iiz.receiving_numbers(id) ON DELETE SET NULL,
    number_type             iiz.number_type NOT NULL DEFAULT 'offsite_static',
    number_class            iiz.number_class NOT NULL DEFAULT 'local',
    pool_id                 UUID        REFERENCES iiz.number_pools(id) ON DELETE SET NULL,
    billing_date            INTEGER     DEFAULT 1,
    is_active               BOOLEAN     NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,
    CONSTRAINT uq_tracking_numbers_number UNIQUE (number)
);
CREATE INDEX idx_tn_account ON iiz.tracking_numbers (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_tn_source ON iiz.tracking_numbers (source_id) WHERE source_id IS NOT NULL;
CREATE INDEX idx_tn_pool ON iiz.tracking_numbers (pool_id) WHERE pool_id IS NOT NULL;
SELECT iiz.add_updated_at_trigger('tracking_numbers');
SELECT iiz.add_notify_trigger('tracking_numbers');
