-- Call Daily Summary (star schema fact table, computed by background job)
CREATE TABLE iiz.call_daily_summary (
    id                      UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID            NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    summary_date            DATE            NOT NULL,
    source_id               UUID,
    agent_id                UUID,
    queue_id                UUID,
    total_calls             INTEGER         NOT NULL DEFAULT 0,
    answered_calls          INTEGER         NOT NULL DEFAULT 0,
    missed_calls            INTEGER         NOT NULL DEFAULT 0,
    voicemail_calls         INTEGER         NOT NULL DEFAULT 0,
    total_duration_secs     INTEGER         NOT NULL DEFAULT 0,
    total_ring_duration_secs INTEGER        NOT NULL DEFAULT 0,
    total_hold_duration_secs INTEGER        NOT NULL DEFAULT 0,
    avg_duration_secs       NUMERIC(10,2),
    avg_ring_duration_secs  NUMERIC(10,2),
    unique_callers          INTEGER         NOT NULL DEFAULT 0,
    first_time_callers      INTEGER         NOT NULL DEFAULT 0,
    repeat_callers          INTEGER         NOT NULL DEFAULT 0,
    converted_calls         INTEGER         NOT NULL DEFAULT 0,
    appointments_set        INTEGER         NOT NULL DEFAULT 0,
    computed_at             TIMESTAMPTZ     NOT NULL DEFAULT now(),
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,
    CONSTRAINT uq_daily_summary UNIQUE (account_id, summary_date, source_id, agent_id, queue_id)
);
CREATE INDEX idx_cds_account_date ON iiz.call_daily_summary (account_id, summary_date);
CREATE INDEX idx_cds_account ON iiz.call_daily_summary (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('call_daily_summary');
-- No notify trigger (computed data, not config)
