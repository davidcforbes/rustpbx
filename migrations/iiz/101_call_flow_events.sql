-- Call Flow Events (partitioned event log — immutable per-call event trace)
CREATE TABLE iiz.call_flow_events (
    id              UUID            NOT NULL,
    account_id      UUID            NOT NULL,
    call_id         UUID            NOT NULL,
    event_type      TEXT            NOT NULL,
    occurred_at     TIMESTAMPTZ     NOT NULL,
    detail          TEXT,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    PRIMARY KEY (id, occurred_at)
) PARTITION BY RANGE (occurred_at);

CREATE INDEX idx_cfe_call_occurred   ON iiz.call_flow_events (call_id, occurred_at);
CREATE INDEX idx_cfe_account_occurred ON iiz.call_flow_events (account_id, occurred_at DESC);

CREATE TRIGGER create_partition_call_flow_events
    BEFORE INSERT ON iiz.call_flow_events
    FOR EACH ROW EXECUTE FUNCTION iiz.create_monthly_partition('occurred_at');
