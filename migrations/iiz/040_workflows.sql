CREATE TABLE iiz.workflows (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    canvas_json     JSONB,
    status          TEXT        NOT NULL DEFAULT 'Draft',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_wf_account ON iiz.workflows (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('workflows');
SELECT iiz.add_notify_trigger('workflows');

CREATE TABLE iiz.workflow_nodes (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    workflow_id     UUID        NOT NULL REFERENCES iiz.workflows(id) ON DELETE CASCADE,
    node_type       iiz.workflow_node_type NOT NULL,
    event_type      TEXT,
    action_type     TEXT,
    condition_type  TEXT,
    config_json     JSONB,
    label           TEXT,
    position_x      REAL,
    position_y      REAL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_wfn_account ON iiz.workflow_nodes (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_wfn_workflow ON iiz.workflow_nodes (workflow_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('workflow_nodes');
SELECT iiz.add_notify_trigger('workflow_nodes');

CREATE TABLE iiz.workflow_edges (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    workflow_id     UUID        NOT NULL REFERENCES iiz.workflows(id) ON DELETE CASCADE,
    from_node_id    UUID        NOT NULL,
    to_node_id      UUID        NOT NULL,
    label           TEXT,
    sort_order      INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_workflow_edges UNIQUE (workflow_id, from_node_id, to_node_id)
);
CREATE INDEX idx_wfe_account ON iiz.workflow_edges (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_wfe_workflow ON iiz.workflow_edges (workflow_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('workflow_edges');
SELECT iiz.add_notify_trigger('workflow_edges');
