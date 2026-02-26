CREATE TABLE iiz.knowledge_banks (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    category        TEXT        NOT NULL DEFAULT 'General',
    document_count  INTEGER     NOT NULL DEFAULT 0,
    total_size_bytes BIGINT     NOT NULL DEFAULT 0,
    status          TEXT        NOT NULL DEFAULT 'Ready',
    last_import_at  TIMESTAMPTZ,
    used_by         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_kb_account ON iiz.knowledge_banks (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('knowledge_banks');
SELECT iiz.add_notify_trigger('knowledge_banks');

CREATE TABLE iiz.knowledge_bank_documents (
    id               UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id       UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    bank_id          UUID        NOT NULL REFERENCES iiz.knowledge_banks(id) ON DELETE CASCADE,
    filename         TEXT        NOT NULL,
    file_type        TEXT        NOT NULL,
    source_url       TEXT,
    file_ref         TEXT,
    content_hash     TEXT,
    file_size_bytes  BIGINT      NOT NULL DEFAULT 0,
    page_count       INTEGER,
    chunk_count      INTEGER     NOT NULL DEFAULT 0,
    embedding_status TEXT        NOT NULL DEFAULT 'Pending',
    embedding_model  TEXT,
    error_message    TEXT,
    indexed_at       TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at       TIMESTAMPTZ
);
CREATE INDEX idx_kbd_account ON iiz.knowledge_bank_documents (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_kbd_bank ON iiz.knowledge_bank_documents (bank_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('knowledge_bank_documents');
SELECT iiz.add_notify_trigger('knowledge_bank_documents');
