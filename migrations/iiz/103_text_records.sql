-- Text Records (NOT partitioned — conversation-level summary)
CREATE TABLE iiz.text_records (
    id                      UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id              UUID            NOT NULL,
    contact_phone           TEXT,
    tracking_number_id      UUID,
    direction               iiz.call_direction NOT NULL,
    preview                 TEXT,
    status                  TEXT            NOT NULL DEFAULT 'Pending',
    sent_at                 TIMESTAMPTZ     NOT NULL DEFAULT now(),
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ
);

CREATE INDEX idx_tr_account_sent ON iiz.text_records (account_id, sent_at DESC);
CREATE INDEX idx_tr_contact      ON iiz.text_records (contact_phone);

-- Text Messages (partitioned event log — immutable per-message records)
CREATE TABLE iiz.text_messages (
    id                      UUID            NOT NULL,
    account_id              UUID            NOT NULL,
    contact_phone           TEXT,
    tracking_number_id      UUID,
    call_id                 UUID,
    direction               iiz.call_direction NOT NULL,
    body                    TEXT            NOT NULL,
    status                  TEXT            NOT NULL DEFAULT 'Pending',
    sent_at                 TIMESTAMPTZ     NOT NULL DEFAULT now(),
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,
    PRIMARY KEY (id, sent_at)
) PARTITION BY RANGE (sent_at);

CREATE INDEX idx_tm_account_sent ON iiz.text_messages (account_id, sent_at DESC);
CREATE INDEX idx_tm_contact      ON iiz.text_messages (contact_phone);

CREATE TRIGGER create_partition_text_messages
    BEFORE INSERT ON iiz.text_messages
    FOR EACH ROW EXECUTE FUNCTION iiz.create_monthly_partition('sent_at');
