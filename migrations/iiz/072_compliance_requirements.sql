-- Compliance Requirements Checklist
CREATE TABLE iiz.compliance_requirements (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    country                 TEXT        NOT NULL DEFAULT 'US',
    requirement_name        TEXT        NOT NULL,
    requirement_description TEXT,
    status                  iiz.compliance_status NOT NULL DEFAULT 'not_started',
    documentation_url       TEXT,
    due_date                DATE,
    completed_at            TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_cr_account ON iiz.compliance_requirements (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('compliance_requirements');
SELECT iiz.add_notify_trigger('compliance_requirements');

-- Compliance Applications
CREATE TABLE iiz.compliance_applications (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    application_name        TEXT        NOT NULL,
    application_type        TEXT,
    country                 TEXT        NOT NULL DEFAULT 'US',
    status                  iiz.compliance_status NOT NULL DEFAULT 'draft',
    submitted_at            TIMESTAMPTZ,
    reviewed_at             TIMESTAMPTZ,
    expires_at              TIMESTAMPTZ,
    rejection_reason        TEXT,
    external_reference_id   TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_ca_account ON iiz.compliance_applications (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('compliance_applications');
SELECT iiz.add_notify_trigger('compliance_applications');

-- Compliance Addresses
CREATE TABLE iiz.compliance_addresses (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    label               TEXT,
    address_line1       TEXT        NOT NULL,
    address_line2       TEXT,
    city                TEXT        NOT NULL,
    state               TEXT        NOT NULL,
    zip                 TEXT        NOT NULL,
    country             TEXT        NOT NULL DEFAULT 'US',
    is_verified         BOOLEAN     NOT NULL DEFAULT false,
    verification_method TEXT,
    verified_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_caddr_account ON iiz.compliance_addresses (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('compliance_addresses');
SELECT iiz.add_notify_trigger('compliance_addresses');

-- Port Requests (LNP lifecycle)
CREATE TABLE iiz.port_requests (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    numbers_to_port         JSONB       NOT NULL,
    first_name              TEXT,
    last_name               TEXT,
    email                   TEXT,
    phone                   TEXT,
    billing_address_line1   TEXT,
    billing_address_line2   TEXT,
    city                    TEXT,
    state                   TEXT,
    zip                     TEXT,
    authorized_signature    TEXT,
    status                  iiz.compliance_status NOT NULL DEFAULT 'draft',
    submitted_at            TIMESTAMPTZ,
    completed_at            TIMESTAMPTZ,
    rejection_reason        TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_pr_account ON iiz.port_requests (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('port_requests');
SELECT iiz.add_notify_trigger('port_requests');
