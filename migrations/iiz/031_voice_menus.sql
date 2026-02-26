CREATE TABLE iiz.voice_menus (
    id                          UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id                  UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                        TEXT        NOT NULL,
    greeting_type               iiz.greeting_type NOT NULL DEFAULT 'tts',
    greeting_audio_url          TEXT,
    greeting_text               TEXT,
    speech_recognition          BOOLEAN     NOT NULL DEFAULT false,
    speech_language             TEXT        DEFAULT 'en-US',
    timeout_secs                INTEGER     NOT NULL DEFAULT 10,
    max_retries                 INTEGER     NOT NULL DEFAULT 3,
    no_input_destination_type   TEXT,
    no_input_destination_id     UUID,
    no_input_destination_number TEXT,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at                  TIMESTAMPTZ
);
CREATE INDEX idx_vm_account ON iiz.voice_menus (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('voice_menus');
SELECT iiz.add_notify_trigger('voice_menus');

CREATE TABLE iiz.voice_menu_options (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    menu_id             UUID        NOT NULL REFERENCES iiz.voice_menus(id) ON DELETE CASCADE,
    dtmf_digit          TEXT        NOT NULL,
    description         TEXT,
    destination_type    TEXT,
    destination_id      UUID,
    destination_number  TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,
    CONSTRAINT uq_voice_menu_options UNIQUE (menu_id, dtmf_digit)
);
CREATE INDEX idx_vmo_menu ON iiz.voice_menu_options (menu_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_vmo_account ON iiz.voice_menu_options (account_id) WHERE deleted_at IS NULL;
SELECT iiz.add_updated_at_trigger('voice_menu_options');
SELECT iiz.add_notify_trigger('voice_menu_options');
