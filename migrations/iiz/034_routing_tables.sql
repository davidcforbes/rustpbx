CREATE TABLE iiz.routing_tables (
    id          UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id  UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    is_active   BOOLEAN     NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ
);
CREATE INDEX idx_rt_account ON iiz.routing_tables (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('routing_tables');
SELECT iiz.add_notify_trigger('routing_tables');

CREATE TABLE iiz.routing_table_routes (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    table_id            UUID        NOT NULL REFERENCES iiz.routing_tables(id) ON DELETE CASCADE,
    priority            INTEGER     NOT NULL DEFAULT 0,
    match_pattern       TEXT,
    destination_type    TEXT,
    destination_id      UUID,
    destination_number  TEXT,
    weight              INTEGER     NOT NULL DEFAULT 1,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_rtr_table ON iiz.routing_table_routes (table_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_rtr_account ON iiz.routing_table_routes (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('routing_table_routes');
SELECT iiz.add_notify_trigger('routing_table_routes');
