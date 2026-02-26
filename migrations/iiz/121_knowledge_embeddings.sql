-- pgvector extension for RAG retrieval
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE iiz.knowledge_bank_embeddings (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL,
    document_id     UUID        NOT NULL,
    chunk_index     INTEGER     NOT NULL,
    chunk_text      TEXT        NOT NULL,
    embedding       vector(1536),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_kbe_chunk UNIQUE (document_id, chunk_index)
);
CREATE INDEX idx_kbe_account ON iiz.knowledge_bank_embeddings (account_id);
CREATE INDEX idx_kbe_document ON iiz.knowledge_bank_embeddings (document_id);
CREATE INDEX idx_kbe_embedding ON iiz.knowledge_bank_embeddings
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);
