CREATE TABLE iiz.smart_routers (
    id                          UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id                  UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                        TEXT        NOT NULL,
    priority                    INTEGER     NOT NULL DEFAULT 0,
    fallback_destination_type   TEXT,
    fallback_destination_id     UUID,
    fallback_destination_number TEXT,
    is_active                   BOOLEAN     NOT NULL DEFAULT true,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at                  TIMESTAMPTZ
);
CREATE INDEX idx_sr_account ON iiz.smart_routers (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('smart_routers');
SELECT iiz.add_notify_trigger('smart_routers');

CREATE TABLE iiz.smart_router_rules (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    router_id           UUID        NOT NULL REFERENCES iiz.smart_routers(id) ON DELETE CASCADE,
    sort_order          INTEGER     NOT NULL DEFAULT 0,
    condition_field     TEXT        NOT NULL,
    condition_operator  TEXT        NOT NULL,
    condition_value     TEXT        NOT NULL,
    destination_type    TEXT,
    destination_id      UUID,
    destination_number  TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_srr_router ON iiz.smart_router_rules (router_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_srr_account ON iiz.smart_router_rules (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('smart_router_rules');
SELECT iiz.add_notify_trigger('smart_router_rules');

CREATE TABLE iiz.geo_routers (
    id                          UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id                  UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                        TEXT        NOT NULL,
    default_destination_type    TEXT,
    default_destination_id      UUID,
    default_destination_number  TEXT,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at                  TIMESTAMPTZ
);
CREATE INDEX idx_gr_account ON iiz.geo_routers (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('geo_routers');
SELECT iiz.add_notify_trigger('geo_routers');

CREATE TABLE iiz.geo_router_rules (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    router_id           UUID        NOT NULL REFERENCES iiz.geo_routers(id) ON DELETE CASCADE,
    region              TEXT        NOT NULL,
    region_type         TEXT        NOT NULL,
    destination_type    TEXT,
    destination_id      UUID,
    destination_number  TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,
    CONSTRAINT uq_geo_router_rules UNIQUE (router_id, region)
);
CREATE INDEX idx_grr_router ON iiz.geo_router_rules (router_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_grr_account ON iiz.geo_router_rules (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('geo_router_rules');
SELECT iiz.add_notify_trigger('geo_router_rules');
