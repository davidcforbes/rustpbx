CREATE TABLE iiz.schedules (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    timezone        TEXT        NOT NULL DEFAULT 'America/New_York',
    monday_open     TIME,
    monday_close    TIME,
    tuesday_open    TIME,
    tuesday_close   TIME,
    wednesday_open  TIME,
    wednesday_close TIME,
    thursday_open   TIME,
    thursday_close  TIME,
    friday_open     TIME,
    friday_close    TIME,
    saturday_open   TIME,
    saturday_close  TIME,
    sunday_open     TIME,
    sunday_close    TIME,
    closed_destination_type TEXT,
    closed_destination_id   UUID,
    closed_destination_number TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_sched_account ON iiz.schedules (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('schedules');
SELECT iiz.add_notify_trigger('schedules');

CREATE TABLE iiz.schedule_holidays (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    schedule_id     UUID        NOT NULL REFERENCES iiz.schedules(id) ON DELETE CASCADE,
    date            DATE        NOT NULL,
    name            TEXT        NOT NULL,
    is_closed       BOOLEAN     NOT NULL DEFAULT true,
    custom_open     TIME,
    custom_close    TIME,
    override_destination_type   TEXT,
    override_destination_id     UUID,
    override_destination_number TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_schedule_holidays UNIQUE (schedule_id, date)
);
CREATE INDEX idx_sh_schedule ON iiz.schedule_holidays (schedule_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_sh_account ON iiz.schedule_holidays (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('schedule_holidays');
SELECT iiz.add_notify_trigger('schedule_holidays');
