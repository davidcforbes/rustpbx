-- Call Visitor Sessions (NOT partitioned — 1:1 with call_records)
CREATE TABLE iiz.call_visitor_sessions (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL,
    call_id             UUID        NOT NULL UNIQUE,
    ip_address          TEXT,
    device              TEXT,
    browser             TEXT,
    os                  TEXT,
    referrer            TEXT,
    landing_page        TEXT,
    keywords            TEXT,
    campaign            TEXT,
    utm_source          TEXT,
    utm_medium          TEXT,
    utm_content         TEXT,
    utm_term            TEXT,
    visit_duration_secs INTEGER,
    pages_viewed        INTEGER,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_cvs_account ON iiz.call_visitor_sessions (account_id);

-- Call Transcription Segments (partitioned event log — immutable speech-to-text output)
CREATE TABLE iiz.call_transcription_segments (
    id                      UUID            NOT NULL,
    account_id              UUID            NOT NULL,
    call_id                 UUID            NOT NULL,
    segment_index           INTEGER         NOT NULL,
    timestamp_offset_secs   REAL,
    speaker                 iiz.speaker_type NOT NULL,
    content                 TEXT            NOT NULL,
    confidence              REAL,
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

CREATE INDEX idx_cts_call_segment    ON iiz.call_transcription_segments (call_id, segment_index);
CREATE INDEX idx_cts_account_created ON iiz.call_transcription_segments (account_id, created_at DESC);

CREATE TRIGGER create_partition_call_transcription_segments
    BEFORE INSERT ON iiz.call_transcription_segments
    FOR EACH ROW EXECUTE FUNCTION iiz.create_monthly_partition('created_at');

-- Call AI Summaries (partitioned event log — immutable AI-generated summaries)
CREATE TABLE iiz.call_ai_summaries (
    id              UUID            NOT NULL,
    account_id      UUID            NOT NULL,
    call_id         UUID            NOT NULL,
    summary_type    iiz.summary_type NOT NULL,
    content         TEXT            NOT NULL,
    model           TEXT,
    generated_at    TIMESTAMPTZ     NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    PRIMARY KEY (id, generated_at)
) PARTITION BY RANGE (generated_at);

CREATE INDEX idx_cas_call_type       ON iiz.call_ai_summaries (call_id, summary_type);
CREATE INDEX idx_cas_account_generated ON iiz.call_ai_summaries (account_id, generated_at DESC);

CREATE TRIGGER create_partition_call_ai_summaries
    BEFORE INSERT ON iiz.call_ai_summaries
    FOR EACH ROW EXECUTE FUNCTION iiz.create_monthly_partition('generated_at');

-- Call Keyword Hits (NOT partitioned — sparse, low volume)
CREATE TABLE iiz.call_keyword_hits (
    id                      UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID        NOT NULL,
    call_id                 UUID        NOT NULL,
    keyword_id              UUID,
    timestamp_offset_secs   REAL,
    speaker                 iiz.speaker_type,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);

CREATE INDEX idx_ckh_call    ON iiz.call_keyword_hits (call_id);
CREATE INDEX idx_ckh_keyword ON iiz.call_keyword_hits (keyword_id, account_id);
