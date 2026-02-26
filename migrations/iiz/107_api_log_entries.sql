-- API Log Entries (partitioned event log — immutable API request/response audit trail)
CREATE TABLE iiz.api_log_entries (
    id                      UUID        NOT NULL,
    account_id              UUID        NOT NULL,
    source                  TEXT,
    method                  TEXT        NOT NULL,
    endpoint                TEXT        NOT NULL,
    request_headers         JSONB,
    request_body            JSONB,
    response_code           INTEGER,
    response_body           JSONB,
    response_size_bytes     INTEGER,
    duration_ms             INTEGER,
    activity_description    TEXT,
    error_message           TEXT,
    timestamp               TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,
    PRIMARY KEY (id, timestamp)
) PARTITION BY RANGE (timestamp);

CREATE INDEX idx_ale_account_timestamp ON iiz.api_log_entries (account_id, timestamp DESC);
CREATE INDEX idx_ale_endpoint          ON iiz.api_log_entries (endpoint);

CREATE TRIGGER create_partition_api_log_entries
    BEFORE INSERT ON iiz.api_log_entries
    FOR EACH ROW EXECUTE FUNCTION iiz.create_monthly_partition('timestamp');
