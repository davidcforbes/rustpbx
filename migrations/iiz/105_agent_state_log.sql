-- Agent State Log (partitioned event log — immutable agent status transitions)
CREATE TABLE iiz.agent_state_log (
    id              UUID            NOT NULL,
    account_id      UUID            NOT NULL,
    agent_id        UUID            NOT NULL,
    status          iiz.agent_status NOT NULL,
    changed_at      TIMESTAMPTZ     NOT NULL DEFAULT now(),
    duration_secs   INTEGER,
    reason          TEXT,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    PRIMARY KEY (id, changed_at)
) PARTITION BY RANGE (changed_at);

CREATE INDEX idx_asl_agent_changed   ON iiz.agent_state_log (agent_id, changed_at DESC);
CREATE INDEX idx_asl_account_changed ON iiz.agent_state_log (account_id, changed_at DESC);

CREATE TRIGGER create_partition_agent_state_log
    BEFORE INSERT ON iiz.agent_state_log
    FOR EACH ROW EXECUTE FUNCTION iiz.create_monthly_partition('changed_at');
