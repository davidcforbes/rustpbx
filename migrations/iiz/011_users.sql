CREATE TABLE iiz.users (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    username        TEXT        NOT NULL,
    email           TEXT        NOT NULL,
    password_hash   TEXT        NOT NULL,
    display_name    TEXT,
    initials        TEXT,
    avatar_color    TEXT        DEFAULT '#00bcd4',
    role            iiz.user_role NOT NULL DEFAULT 'agent',
    phone           TEXT,
    is_active       BOOLEAN     NOT NULL DEFAULT true,
    reset_token     TEXT,
    reset_token_expires TIMESTAMPTZ,
    last_login_at   TIMESTAMPTZ,
    last_login_ip   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_users_account_username UNIQUE (account_id, username),
    CONSTRAINT uq_users_email UNIQUE (email)
);

CREATE INDEX idx_users_account ON iiz.users (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_role ON iiz.users (account_id, role) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('users');
SELECT iiz.add_notify_trigger('users');
