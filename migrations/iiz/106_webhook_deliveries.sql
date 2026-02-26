-- Webhook Deliveries (partitioned event log — immutable delivery attempt records)
CREATE TABLE iiz.webhook_deliveries (
    id                  UUID        NOT NULL,
    account_id          UUID        NOT NULL,
    webhook_id          UUID        NOT NULL,
    event_type          TEXT        NOT NULL,
    payload             JSONB,
    http_status_code    INTEGER,
    response_body       TEXT,
    status              TEXT        NOT NULL DEFAULT 'Pending',
    attempt_number      INTEGER     NOT NULL DEFAULT 1,
    delivered_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,
    PRIMARY KEY (id, delivered_at)
) PARTITION BY RANGE (delivered_at);

CREATE INDEX idx_wd_webhook_delivered  ON iiz.webhook_deliveries (webhook_id, delivered_at DESC);
CREATE INDEX idx_wd_account_delivered  ON iiz.webhook_deliveries (account_id, delivered_at DESC);

CREATE TRIGGER create_partition_webhook_deliveries
    BEFORE INSERT ON iiz.webhook_deliveries
    FOR EACH ROW EXECUTE FUNCTION iiz.create_monthly_partition('delivered_at');
