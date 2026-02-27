-- Business identity: singleton per account
CREATE TABLE iiz.business_info (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    legal_business_name TEXT,
    dba                 TEXT,
    ein                 TEXT,
    industry            TEXT,
    address_line1       TEXT,
    address_line2       TEXT,
    city                TEXT,
    state               TEXT,
    zip                 TEXT,
    country             TEXT        NOT NULL DEFAULT 'US',
    phone               TEXT,
    email               TEXT,
    website             TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,
    CONSTRAINT uq_business_info_account UNIQUE (account_id)
);
CREATE INDEX idx_bi_account ON iiz.business_info (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('business_info');
SELECT iiz.add_notify_trigger('business_info');

CREATE TABLE iiz.authorized_contacts (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    business_info_id    UUID        NOT NULL REFERENCES iiz.business_info(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    title               TEXT,
    phone               TEXT,
    email               TEXT,
    is_primary          BOOLEAN     NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_ac_business ON iiz.authorized_contacts (business_info_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_authorized_contacts_account ON iiz.authorized_contacts (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('authorized_contacts');
SELECT iiz.add_notify_trigger('authorized_contacts');
