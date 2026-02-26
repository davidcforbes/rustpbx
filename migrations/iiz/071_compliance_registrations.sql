-- A2P 10DLC Campaigns
CREATE TABLE iiz.a2p_campaigns (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    campaign_name       TEXT        NOT NULL,
    brand_name          TEXT,
    use_case            TEXT,
    description         TEXT,
    sample_messages     TEXT,
    opt_in_description  TEXT,
    opt_out_description TEXT,
    assigned_numbers    INTEGER     NOT NULL DEFAULT 0,
    max_numbers         INTEGER,
    monthly_cost        NUMERIC(10,2),
    carrier             TEXT,
    status              iiz.compliance_status NOT NULL DEFAULT 'draft',
    rejection_reason    TEXT,
    dlc_campaign_id     TEXT,
    submitted_at        TIMESTAMPTZ,
    approved_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_a2p_account ON iiz.a2p_campaigns (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('a2p_campaigns');
SELECT iiz.add_notify_trigger('a2p_campaigns');

-- Toll-Free Registrations
CREATE TABLE iiz.toll_free_registrations (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    business_name           TEXT,
    contact_name            TEXT,
    contact_phone           TEXT,
    contact_email           TEXT,
    use_case                TEXT,
    use_case_description    TEXT,
    monthly_volume          TEXT,
    toll_free_numbers       JSONB,
    status                  iiz.compliance_status NOT NULL DEFAULT 'not_registered',
    rejection_reason        TEXT,
    submitted_at            TIMESTAMPTZ,
    approved_at             TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_tfr_account ON iiz.toll_free_registrations (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('toll_free_registrations');
SELECT iiz.add_notify_trigger('toll_free_registrations');

-- Voice/STIR-SHAKEN Registrations
CREATE TABLE iiz.voice_registrations (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    business_name           TEXT,
    ein                     TEXT,
    address_line1           TEXT,
    address_line2           TEXT,
    city                    TEXT,
    state                   TEXT,
    zip                     TEXT,
    status                  iiz.compliance_status NOT NULL DEFAULT 'not_registered',
    attestation_level       iiz.attestation_level,
    last_verified_at        TIMESTAMPTZ,
    next_verification_due   DATE,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);
CREATE INDEX idx_vr_account ON iiz.voice_registrations (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('voice_registrations');
SELECT iiz.add_notify_trigger('voice_registrations');

-- Voice Registration History (append-only audit trail)
CREATE TABLE iiz.voice_registration_history (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    registration_id     UUID        NOT NULL REFERENCES iiz.voice_registrations(id) ON DELETE CASCADE,
    event_date          DATE        NOT NULL,
    event_type          TEXT        NOT NULL,
    old_status          TEXT,
    new_status          TEXT,
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_vrh_registration ON iiz.voice_registration_history (registration_id);
CREATE INDEX idx_vrh_account ON iiz.voice_registration_history (account_id) WHERE deleted_at IS NULL;
-- No updated_at trigger (append-only)
-- No notify trigger (event log)

-- Caller ID CNAM
CREATE TABLE iiz.caller_id_cnam (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number              TEXT        NOT NULL,
    tracking_number_id  UUID,
    current_cnam        TEXT,
    requested_cnam      TEXT,
    status              TEXT        NOT NULL DEFAULT 'Not Configured',
    last_updated_at     TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_cnam_account ON iiz.caller_id_cnam (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('caller_id_cnam');
SELECT iiz.add_notify_trigger('caller_id_cnam');
