-- Contact Lists
CREATE TABLE iiz.contact_lists (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    description     TEXT,
    member_count    INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_contact_lists_account_name UNIQUE (account_id, name)
);
CREATE INDEX idx_contact_lists_account ON iiz.contact_lists (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('contact_lists');
SELECT iiz.add_notify_trigger('contact_lists');

-- Contact List Members
CREATE TABLE iiz.contact_list_members (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    list_id         UUID        NOT NULL REFERENCES iiz.contact_lists(id) ON DELETE CASCADE,
    phone           TEXT        NOT NULL,
    contact_name    TEXT,
    added_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_contact_list_members_phone UNIQUE (list_id, phone)
);
CREATE INDEX idx_clm_list ON iiz.contact_list_members (list_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_clm_account ON iiz.contact_list_members (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('contact_list_members');

-- Blocked Numbers
CREATE TABLE iiz.blocked_numbers (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number          TEXT        NOT NULL,
    cnam            TEXT,
    calls_blocked   INTEGER     NOT NULL DEFAULT 0,
    last_blocked_at TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_blocked_numbers_account UNIQUE (account_id, number)
);
CREATE INDEX idx_blocked_account ON iiz.blocked_numbers (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('blocked_numbers');

-- DNC (Do Not Call) Entries
CREATE TABLE iiz.dnc_entries (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number          TEXT        NOT NULL,
    added_by_id     UUID        REFERENCES iiz.users(id) ON DELETE SET NULL,
    reason          TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_dnc_account UNIQUE (account_id, number)
);
CREATE INDEX idx_dnc_account ON iiz.dnc_entries (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('dnc_entries');

-- DNT (Do Not Text) Entries
CREATE TABLE iiz.dnt_entries (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number          TEXT        NOT NULL,
    e164            TEXT        NOT NULL,
    rejected_count  INTEGER     NOT NULL DEFAULT 0,
    last_rejected_at TIMESTAMPTZ,
    added_by_id     UUID        REFERENCES iiz.users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_dnt_account UNIQUE (account_id, e164)
);
CREATE INDEX idx_dnt_account ON iiz.dnt_entries (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('dnt_entries');
