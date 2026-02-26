CREATE TABLE iiz.voicemail_boxes (
    id                          UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id                  UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                        TEXT        NOT NULL,
    max_message_length_secs     INTEGER     NOT NULL DEFAULT 120,
    greeting_type               iiz.greeting_type NOT NULL DEFAULT 'default',
    greeting_audio_url          TEXT,
    transcription_enabled       BOOLEAN     NOT NULL DEFAULT false,
    email_notification_enabled  BOOLEAN     NOT NULL DEFAULT false,
    notification_email          TEXT,
    max_messages                INTEGER     NOT NULL DEFAULT 50,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at                  TIMESTAMPTZ
);
CREATE INDEX idx_vb_account ON iiz.voicemail_boxes (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('voicemail_boxes');
SELECT iiz.add_notify_trigger('voicemail_boxes');

-- voicemail_messages: Event Log category (no notify trigger)
CREATE TABLE iiz.voicemail_messages (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    mailbox_id      UUID        NOT NULL REFERENCES iiz.voicemail_boxes(id) ON DELETE CASCADE,
    call_id         UUID,
    caller_number   TEXT,
    caller_name     TEXT,
    duration_secs   INTEGER     NOT NULL DEFAULT 0,
    audio_url       TEXT,
    transcription   TEXT,
    is_read         BOOLEAN     NOT NULL DEFAULT false,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_vmsg_mailbox ON iiz.voicemail_messages (mailbox_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_vmsg_account ON iiz.voicemail_messages (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('voicemail_messages');
