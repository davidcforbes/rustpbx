-- Call Records (partitioned event log — immutable CDR)
CREATE TABLE iiz.call_records (
    id                      UUID            NOT NULL,
    account_id              UUID            NOT NULL,
    call_id                 TEXT            NOT NULL,
    caller_phone            TEXT,
    callee_phone            TEXT,
    direction               iiz.call_direction NOT NULL,
    status                  iiz.call_status NOT NULL,
    source_id               UUID,
    source_number_id        UUID,
    agent_id                UUID,
    queue_id                UUID,
    started_at              TIMESTAMPTZ     NOT NULL,
    answered_at             TIMESTAMPTZ,
    ended_at                TIMESTAMPTZ,
    duration_secs           INTEGER         NOT NULL DEFAULT 0,
    ring_duration_secs      INTEGER         NOT NULL DEFAULT 0,
    hold_duration_secs      INTEGER         NOT NULL DEFAULT 0,
    recording_url           TEXT,
    has_audio               BOOLEAN         NOT NULL DEFAULT false,
    is_first_time_caller    BOOLEAN         NOT NULL DEFAULT false,
    location                TEXT,
    automation_id           UUID,
    source_name             TEXT,
    agent_name              TEXT,
    queue_name              TEXT,
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,
    PRIMARY KEY (id, started_at)
) PARTITION BY RANGE (started_at);

CREATE INDEX idx_cr_account_started ON iiz.call_records (account_id, started_at DESC);
CREATE INDEX idx_cr_caller_phone    ON iiz.call_records (caller_phone);
CREATE INDEX idx_cr_callee_phone    ON iiz.call_records (callee_phone);
CREATE INDEX idx_cr_source          ON iiz.call_records (source_id) WHERE source_id IS NOT NULL;
CREATE INDEX idx_cr_agent           ON iiz.call_records (agent_id) WHERE agent_id IS NOT NULL;
CREATE INDEX idx_cr_queue           ON iiz.call_records (queue_id) WHERE queue_id IS NOT NULL;
CREATE INDEX idx_cr_account_status  ON iiz.call_records (account_id, status);
CREATE INDEX idx_cr_call_id         ON iiz.call_records (call_id);

CREATE TRIGGER create_partition_call_records
    BEFORE INSERT ON iiz.call_records
    FOR EACH ROW EXECUTE FUNCTION iiz.create_monthly_partition('started_at');

-- Call Annotations (NOT partitioned — mutable 1:1 overlay for call_records)
CREATE TABLE iiz.call_annotations (
    call_id             UUID        NOT NULL PRIMARY KEY,
    account_id          UUID        NOT NULL,
    score               INTEGER,
    converted           BOOLEAN,
    outcome             TEXT,
    reporting_tag       TEXT,
    category            TEXT,
    appointment_set     BOOLEAN,
    notes               TEXT,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by_id       UUID,
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_ca_account  ON iiz.call_annotations (account_id);
CREATE INDEX idx_ca_outcome  ON iiz.call_annotations (outcome) WHERE outcome IS NOT NULL;

SELECT iiz.add_updated_at_trigger('call_annotations');
