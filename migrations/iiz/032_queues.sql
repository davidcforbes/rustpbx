CREATE TABLE iiz.queues (
    id                          UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id                  UUID            NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                        TEXT            NOT NULL,
    description                 TEXT,
    strategy                    iiz.queue_strategy NOT NULL DEFAULT 'ring_all',
    schedule_id                 UUID            REFERENCES iiz.schedules(id) ON DELETE SET NULL,
    repeat_callers              BOOLEAN         NOT NULL DEFAULT false,
    caller_id_display           TEXT,
    max_wait_secs               INTEGER         NOT NULL DEFAULT 300,
    no_answer_destination_type  TEXT,
    no_answer_destination_id    UUID,
    no_answer_destination_number TEXT,
    moh_audio_url               TEXT,
    wrap_up_secs                INTEGER         NOT NULL DEFAULT 30,
    is_active                   BOOLEAN         NOT NULL DEFAULT true,
    created_at                  TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at                  TIMESTAMPTZ,
    CONSTRAINT uq_queues_account_name UNIQUE (account_id, name)
);
CREATE INDEX idx_q_account ON iiz.queues (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('queues');
SELECT iiz.add_notify_trigger('queues');

CREATE TABLE iiz.queue_agents (
    id          UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id  UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    queue_id    UUID        NOT NULL REFERENCES iiz.queues(id) ON DELETE CASCADE,
    agent_id    UUID        NOT NULL REFERENCES iiz.users(id) ON DELETE CASCADE,
    priority    INTEGER     NOT NULL DEFAULT 0,
    is_active   BOOLEAN     NOT NULL DEFAULT true,
    added_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ,
    CONSTRAINT uq_queue_agents UNIQUE (queue_id, agent_id)
);
CREATE INDEX idx_qa_queue ON iiz.queue_agents (queue_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_qa_agent ON iiz.queue_agents (agent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_qa_account ON iiz.queue_agents (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('queue_agents');
SELECT iiz.add_notify_trigger('queue_agents');
