-- Form Records (NOT partitioned — low volume)
CREATE TABLE iiz.form_records (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID            NOT NULL,
    contact_name        TEXT,
    contact_phone       TEXT,
    contact_email       TEXT,
    form_name           TEXT,
    source              TEXT,
    tracking_number     TEXT,
    form_data           JSONB,
    status              TEXT            NOT NULL DEFAULT 'New',
    submitted_at        TIMESTAMPTZ     NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_fr_account ON iiz.form_records (account_id);

-- Chat Records (NOT partitioned — low volume)
CREATE TABLE iiz.chat_records (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID            NOT NULL,
    visitor_name        TEXT,
    visitor_detail      TEXT,
    channel             iiz.channel_type,
    message_count       INTEGER         NOT NULL DEFAULT 0,
    agent_id            UUID,
    widget_id           UUID,
    status              TEXT            NOT NULL DEFAULT 'Active',
    duration_secs       INTEGER         NOT NULL DEFAULT 0,
    started_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    ended_at            TIMESTAMPTZ,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_chr_account ON iiz.chat_records (account_id);

-- Fax Records (NOT partitioned — low volume)
CREATE TABLE iiz.fax_records (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID            NOT NULL,
    from_number         TEXT,
    to_number           TEXT,
    direction           iiz.call_direction NOT NULL,
    pages               INTEGER         NOT NULL DEFAULT 0,
    status              TEXT            NOT NULL DEFAULT 'Received',
    document_url        TEXT,
    sent_at             TIMESTAMPTZ     NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_fxr_account ON iiz.fax_records (account_id);

-- Video Records (NOT partitioned — low volume)
CREATE TABLE iiz.video_records (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID            NOT NULL,
    participant_name    TEXT,
    participant_email   TEXT,
    host_agent_id       UUID,
    platform            TEXT,
    has_recording       BOOLEAN         NOT NULL DEFAULT false,
    recording_url       TEXT,
    duration_secs       INTEGER         NOT NULL DEFAULT 0,
    started_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    ended_at            TIMESTAMPTZ,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_video_records_account ON iiz.video_records (account_id);
